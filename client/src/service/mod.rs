mod command_executor;
mod pull_service;
mod push_service;
mod sandbox;
mod terminal_session_manager;

pub use command_executor::CommandExecutor;
pub use pull_service::PullService;
pub use push_service::PushService;
pub use terminal_session_manager::TerminalSessionManager;

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{debug, error, info, instrument, warn};

const AGENT_TOKEN_CAPABILITY_HEADER: &str = "x-rs-cmdb-agent-token-capable";

use crate::collector::linux_collector;
use crate::config::ClientConfig;
use common::entity::hardware::{Hardware, NICType, NIC};
use common::models::{Client, RegisterClientResponse};

/// 客户端服务，负责运行推送和拉取服务
pub struct ClientService {
    config: Arc<ClientConfig>,
    hardware_cache: Arc<Mutex<Option<Hardware>>>,
    client_id: String,
    scheduler: Arc<JobScheduler>,
}

impl ClientService {
    /// 创建新的客户端服务实例
    pub async fn new(config: Arc<ClientConfig>) -> Result<Self> {
        let client_id = config
            .client_id
            .as_deref()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .map(str::to_string)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "client ID is missing; refusing to start with an ephemeral identity"
                )
            })?;

        // 创建调度器
        let scheduler = JobScheduler::new()
            .await
            .context("Failed to create job scheduler")?;

        Ok(Self {
            config,
            hardware_cache: Arc::new(Mutex::new(None)),
            client_id,
            scheduler: Arc::new(scheduler),
        })
    }

    /// 启动客户端服务
    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting client service with client ID: {}", self.client_id);

        // 注册客户端
        let active_client_id = self.register_client().await?;

        // 启动推送服务
        if self.config.report.push_enabled {
            self.start_push_service(&active_client_id).await?;
        }

        // 启动拉取服务
        if self.config.report.pull_enabled {
            self.start_pull_service().await?;
        }

        // 启动调度器
        self.scheduler
            .start()
            .await
            .context("Failed to start scheduler")?;

        // 启动远程命令执行长轮询
        self.start_command_executor(&active_client_id).await;

        // 启动终端会话长轮询
        self.start_terminal_session_manager(&active_client_id).await;

        Ok(())
    }

    /// 收集硬件信息
    #[instrument(skip(self))]
    pub async fn collect_hardware(&self) -> Result<Hardware> {
        debug!("Collecting hardware information");

        // 使用收集器收集硬件信息
        let hardware = linux_collector::collect_hardware();

        // 缓存硬件信息
        {
            let mut cache = self.hardware_cache.lock().await;
            *cache = Some(hardware.clone());
        }

        Ok(hardware)
    }

    /// 注册客户端到服务器
    #[instrument(skip(self))]
    async fn register_client(&self) -> Result<String> {
        info!("Registering client to server");

        let os_info = linux_collector::collect_os_info();
        let system_info = linux_collector::collect_system_info();
        let hostname = self
            .config
            .hostname
            .clone()
            .filter(|hostname| !hostname.trim().is_empty())
            .unwrap_or_else(|| os_info.hostname.clone());

        // Auto-detect primary IP: explicit config takes priority, then infer from server URL
        let primary_ip = if let Some(cfg) = &self.config.primary_ip {
            detect_primary_ip_from_subnet(&cfg.subnet)
        } else {
            // Infer subnet from server URL (e.g., http://10.0.0.50:8080/api/v1 -> 10.0.0.0/24)
            let server_host = self
                .config
                .server
                .url
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split(':')
                .next()
                .unwrap_or("");
            if let Ok(ip) = server_host.parse::<std::net::Ipv4Addr>() {
                let inferred = format!(
                    "{}.{}.{}.0/24",
                    ip.octets()[0],
                    ip.octets()[1],
                    ip.octets()[2]
                );
                detect_primary_ip_from_subnet(&inferred)
            } else {
                None
            }
        };

        let registration = Client {
            id: self.client_id.clone(),
            hostname,
            serial_number: Some(system_info.serial_number.clone()),
            sys_vendor: Some(system_info.sys_vendor.clone()),
            product_name: Some(system_info.product_name.clone()),
            ip_address: os_info.ip_address,
            primary_ip,
            os: Some(format!(
                "{}-{}",
                os_info.name.clone(),
                os_info.version.clone()
            )),
            kernel_version: Some(os_info.kernel.clone()),
            last_seen: None,
            registered_at: None,
            comment: None,
            location: None,
            rack: None,
            unit_position: None,
            u_height: Some(1),
            project_id: None,
            owner_id: None,
            status: None,
            environment: None,
            asset_tag: None,
            tags: Vec::new(),
            warranty_expiration: None,
            supplier: None,
            power_consumption: None,
            agent_token: None,
            created_by: None,
        };

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!self.config.server.verify_tls)
            .build()?;

        let registration_url = format!("{}/clients/register", self.config.server.url);
        let token_candidates = load_agent_token_candidates_for(&self.client_id);
        let response = if token_candidates.is_empty() {
            warn!(
                "No persisted agent token found for client {}; an existing server record cannot be reclaimed without its token",
                self.client_id
            );
            client
                .post(&registration_url)
                .header(AGENT_TOKEN_CAPABILITY_HEADER, "1")
                .json(&registration)
                .send()
                .await?
        } else {
            let mut last_response = None;
            for (token, path) in token_candidates {
                debug!("Trying persisted agent token from {}", path.display());
                let response = client
                    .post(&registration_url)
                    .header(AGENT_TOKEN_CAPABILITY_HEADER, "1")
                    .header(
                        "Authorization",
                        format!("Bearer {}:{}", self.client_id, token),
                    )
                    .json(&registration)
                    .send()
                    .await?;
                let unauthorized = response.status() == reqwest::StatusCode::UNAUTHORIZED;
                last_response = Some(response);
                if !unauthorized {
                    break;
                }
                warn!(
                    "Persisted agent token from {} was rejected; trying the next migration candidate",
                    path.display()
                );
            }
            last_response.expect("token candidate list is non-empty")
        };

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Failed to register client: {:?}", error_text);
            return Err(anyhow::anyhow!("Server returned error: {:?}", error_text));
        }

        // Parse the registration response to extract and persist the agent token.
        let body = response.bytes().await?;
        let parsed: common::models::ApiResponse<RegisterClientResponse> =
            serde_json::from_slice(&body).context("registration response was invalid")?;
        let registered = parsed
            .data
            .ok_or_else(|| anyhow::anyhow!("registration response did not include client data"))?;
        let active_client_id = registered.client.id.trim().to_string();
        if active_client_id.is_empty() {
            anyhow::bail!("registration response returned an empty client ID");
        }

        if let Err(e) = save_agent_token_for(&active_client_id, &registered.agent_token) {
            warn!(
                "Failed to persist agent token: {}. Commands will not work until re-registered.",
                e
            );
        } else {
            info!("Agent token saved successfully.");
        }
        if active_client_id != self.client_id {
            info!(
                "Server reconciled client identity {} to {}; persisting canonical ID",
                self.client_id, active_client_id
            );
        }
        crate::config::persist_active_client_id(&active_client_id)
            .context("failed to persist canonical client ID")?;

        // Remove the malformed empty-ID migration token after it has been
        // replaced by a token bound to the canonical client identity.
        if self.client_id != active_client_id {
            let _ = remove_unknown_agent_token();
        }

        info!("Client registered successfully");
        Ok(active_client_id)
    }

    /// 启动推送服务
    #[instrument(skip(self))]
    async fn start_push_service(&self, client_id: &str) -> Result<()> {
        info!("Starting push service");

        let push_service = PushService::new(
            self.config.clone(),
            client_id.to_string(),
            self.hardware_cache.clone(),
        );

        // 设置定时任务，定期推送硬件信息
        let interval = self.config.report.push_interval;
        let push_svc = push_service.clone();

        let job = Job::new_async(
            format!("0 */{} * * * *", interval / 60).as_str(),
            move |_, _| {
                let push = push_svc.clone();
                Box::pin(async move {
                    if let Err(e) = push.push_hardware_info().await {
                        error!("Failed to push hardware info: {}", e);
                    }
                })
            },
        )
        .context("Failed to create push job")?;

        self.scheduler
            .add(job)
            .await
            .context("Failed to add push job to scheduler")?;

        // 立即执行一次推送
        let push_svc = push_service.clone();
        tokio::spawn(async move {
            if let Err(e) = push_svc.push_hardware_info().await {
                error!("Failed to push initial hardware info: {}", e);
            }
        });

        Ok(())
    }

    /// 启动拉取服务
    #[instrument(skip(self))]
    async fn start_pull_service(&self) -> Result<()> {
        info!("Starting pull service");

        // 创建拉取服务
        let _pull_service = PullService::new(
            self.config.clone(),
            self.client_id.clone(),
            self.hardware_cache.clone(),
        );

        // TODO: 实现长轮询或WebSocket连接以接收服务器请求
        // 当前版本简单化，使用定期检查服务器请求的方式

        Ok(())
    }

    /// 启动远程命令执行长轮询循环（后台 task）
    async fn start_command_executor(&self, client_id: &str) {
        let executor = CommandExecutor::new(self.config.clone(), client_id.to_string());
        tokio::spawn(async move {
            executor.run_poll_loop().await;
        });
        info!("CommandExecutor: background poll loop started");
    }

    async fn start_terminal_session_manager(&self, client_id: &str) {
        let manager = TerminalSessionManager::new(self.config.clone(), client_id.to_string());
        tokio::spawn(async move {
            manager.run_poll_loop().await;
        });
        info!("TerminalSessionManager: background poll loop started");
    }
}

/// Try to detect primary IP by matching NICs against a CIDR subnet.
/// Returns the IPv4 of the first matching NIC, preferring Ethernet.
fn detect_primary_ip_from_subnet(subnet: &str) -> Option<String> {
    let subnet: ipnet::IpNet = subnet.parse().ok()?;
    let nics = linux_collector::collect_nic_info().ok()?;
    let mut candidates: Vec<&NIC> = nics
        .iter()
        .filter(|nic| {
            nic.ipv4_address
                .parse::<std::net::IpAddr>()
                .ok()
                .is_some_and(|ip| subnet.contains(&ip))
        })
        .collect();
    candidates.sort_by_key(|nic| match nic.nic_type {
        NICType::Ethernet => 0,
        _ => 1,
    });
    candidates.first().map(|nic| nic.ipv4_address.clone())
}

// ── Agent token persistence ──────────────────────────────────────────────────

/// Base directory where agent tokens are stored.
fn agent_token_dir() -> std::path::PathBuf {
    // Prefer system-wide path when running as root/service, fallback to user config dir.
    if std::path::Path::new("/var/lib/rs-cmdb").exists() {
        std::path::PathBuf::from("/var/lib/rs-cmdb")
    } else if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".config/rs-cmdb")
    } else {
        std::path::PathBuf::from(".")
    }
}

/// All token directories used by current and older installations. Do not make
/// reads depend on whether `/var/lib/rs-cmdb` exists: package upgrades can
/// create it after an older agent saved its token under `$HOME`.
fn agent_token_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![std::path::PathBuf::from("/var/lib/rs-cmdb")];
    if let Ok(home) = std::env::var("HOME") {
        let user_dir = std::path::PathBuf::from(home).join(".config/rs-cmdb");
        if !dirs.contains(&user_dir) {
            dirs.push(user_dir);
        }
    }
    let current_dir = std::path::PathBuf::from(".");
    if !dirs.contains(&current_dir) {
        dirs.push(current_dir);
    }
    dirs
}

fn sanitize_client_id(client_id: &str) -> String {
    let sanitized: String = client_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .take(128)
        .collect();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn agent_token_path_for(client_id: &str) -> std::path::PathBuf {
    agent_token_dir().join(format!("agent_token_{}", sanitize_client_id(client_id)))
}

fn legacy_agent_token_path() -> std::path::PathBuf {
    agent_token_dir().join("agent_token")
}

fn token_candidate_paths(client_id: &str, dirs: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    let client_filename = format!("agent_token_{}", sanitize_client_id(client_id));
    // Search every client-specific file before considering any legacy file.
    // Otherwise a stale generic token in the preferred directory can mask the
    // correct client-specific token in an older directory.
    dirs.iter()
        .map(|dir| dir.join(&client_filename))
        .chain(dirs.iter().map(|dir| dir.join("agent_token")))
        .chain(dirs.iter().map(|dir| dir.join("agent_token_unknown")))
        .collect()
}

fn load_agent_token_from_dirs(
    client_id: &str,
    dirs: &[std::path::PathBuf],
) -> Option<(String, std::path::PathBuf)> {
    token_candidate_paths(client_id, dirs)
        .into_iter()
        .find_map(|path| {
            let token = std::fs::read_to_string(&path).ok()?;
            let token = token.trim();
            (!token.is_empty()).then(|| (token.to_string(), path))
        })
}

fn load_agent_token_candidates_from_dirs(
    client_id: &str,
    dirs: &[std::path::PathBuf],
) -> Vec<(String, std::path::PathBuf)> {
    let mut candidates = Vec::new();
    for path in token_candidate_paths(client_id, dirs) {
        let Ok(token) = std::fs::read_to_string(&path) else {
            continue;
        };
        let token = token.trim();
        if token.is_empty()
            || candidates
                .iter()
                .any(|(existing, _): &(String, std::path::PathBuf)| existing == token)
        {
            continue;
        }
        candidates.push((token.to_string(), path));
    }
    candidates
}

fn load_agent_token_candidates_for(client_id: &str) -> Vec<(String, std::path::PathBuf)> {
    load_agent_token_candidates_from_dirs(client_id, &agent_token_dirs())
}

/// Persist the plain-text agent token to a client-specific file (chmod 600).
pub fn save_agent_token_for(client_id: &str, token: &str) -> std::io::Result<()> {
    let path = agent_token_path_for(client_id);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, token)?;
    // Set file permissions to owner-read-only (0o600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Backwards-compatible legacy token writer. New code should use the
/// client-specific writer above.
pub fn save_agent_token(token: &str) -> std::io::Result<()> {
    let path = legacy_agent_token_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, token)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Load the persisted token for a specific client. The legacy single-token
/// file is a migration fallback for older installations.
pub fn load_agent_token_for(client_id: &str) -> Option<String> {
    load_agent_token_from_dirs(client_id, &agent_token_dirs()).map(|(token, path)| {
        debug!("Loaded persisted agent token from {}", path.display());
        token
    })
}

fn remove_unknown_agent_token() -> std::io::Result<()> {
    let mut first_error = None;
    for path in agent_token_dirs()
        .into_iter()
        .map(|dir| dir.join("agent_token_unknown"))
    {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) if first_error.is_none() => first_error = Some(err),
            Err(_) => {}
        }
    }
    first_error.map_or(Ok(()), Err)
}

/// Load the legacy single-client token.
pub fn load_agent_token() -> Option<String> {
    agent_token_dirs().into_iter().find_map(|dir| {
        let token = std::fs::read_to_string(dir.join("agent_token")).ok()?;
        let token = token.trim();
        (!token.is_empty()).then(|| token.to_string())
    })
}

#[cfg(test)]
mod token_persistence_tests {
    use super::{
        load_agent_token_candidates_from_dirs, load_agent_token_from_dirs, token_candidate_paths,
    };
    use std::path::PathBuf;

    #[test]
    fn searches_all_specific_tokens_before_legacy_tokens() {
        let preferred = PathBuf::from("/preferred");
        let legacy = PathBuf::from("/legacy");
        let paths = token_candidate_paths("client-1", &[preferred, legacy]);

        assert_eq!(paths[0], PathBuf::from("/preferred/agent_token_client-1"));
        assert_eq!(paths[1], PathBuf::from("/legacy/agent_token_client-1"));
        assert_eq!(paths[2], PathBuf::from("/preferred/agent_token"));
        assert_eq!(paths[4], PathBuf::from("/preferred/agent_token_unknown"));
    }

    #[test]
    fn discovers_specific_token_in_an_older_directory() {
        let root =
            std::env::temp_dir().join(format!("rs-cmdb-token-test-{}", uuid::Uuid::new_v4()));
        let preferred = root.join("preferred");
        let legacy = root.join("legacy");
        std::fs::create_dir_all(&preferred).unwrap();
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(preferred.join("agent_token"), "stale-generic").unwrap();
        std::fs::write(legacy.join("agent_token_client-1"), "correct-specific\n").unwrap();

        let loaded = load_agent_token_from_dirs("client-1", &[preferred, legacy.clone()]).unwrap();
        assert_eq!(loaded.0, "correct-specific");
        assert_eq!(loaded.1, legacy.join("agent_token_client-1"));

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn returns_distinct_migration_tokens_for_server_validation() {
        let root =
            std::env::temp_dir().join(format!("rs-cmdb-token-test-{}", uuid::Uuid::new_v4()));
        let preferred = root.join("preferred");
        let legacy = root.join("legacy");
        std::fs::create_dir_all(&preferred).unwrap();
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(preferred.join("agent_token_client-1"), "new-token").unwrap();
        std::fs::write(preferred.join("agent_token"), "new-token").unwrap();
        std::fs::write(legacy.join("agent_token"), "old-valid-token").unwrap();

        let candidates =
            load_agent_token_candidates_from_dirs("client-1", &[preferred, legacy.clone()]);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].0, "new-token");
        assert_eq!(candidates[1].0, "old-valid-token");
        assert_eq!(candidates[1].1, legacy.join("agent_token"));

        std::fs::remove_dir_all(root).unwrap();
    }
}
