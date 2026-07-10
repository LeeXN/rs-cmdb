//! Danger detection service for remote command execution.
//!
//! Classifies commands as `Safe`, `Warning`, or `Blocked` based on
//! the command name and individual argument tokens. Since the agent now
//! executes commands via `std::process::Command` (no shell), shell-based
//! injection patterns (pipes, redirects, subshells) are inherently blocked.
//!
//! This service provides a server-side pre-check for the UI.

use common::command::DangerLevel;

/// A single detection rule.
#[derive(Debug, Clone)]
pub struct DangerRule {
    pub name: &'static str,
    #[allow(dead_code)]
    pub level: DangerLevel,
    /// Returns true if the command+args match this rule.
    pub matcher: fn(&str, &[String]) -> bool,
}

/// Result of analysing a command.
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub level: DangerLevel,
    /// Name of the first rule that matched (if any).
    pub matched_rule: Option<String>,
    /// Human-readable summary.
    pub message: String,
}

/// Stateless service – all rules are compile-time constants.
pub struct DangerDetectionService;

// ─── BLOCKED rules ───────────────────────────────────────────────────────────

fn rule_rm_rf(cmd: &str, args: &[String]) -> bool {
    // rm -rf /, rm -rf /* etc. — check command name + recursive + root path
    if cmd != "rm" {
        return false;
    }
    let has_recursive = args.iter().any(|a| a == "-rf" || a == "-fr" || a == "--no-preserve-root")
        || (args.contains(&"-r".to_string()) && args.contains(&"-f".to_string()));
    let has_root = args.iter().any(|a| a == "/" || a == "/*");
    has_recursive && has_root
}

fn rule_mkfs(cmd: &str, _args: &[String]) -> bool {
    cmd == "mkfs" || cmd.starts_with("mkfs.")
}

fn rule_dd_dev(cmd: &str, args: &[String]) -> bool {
    cmd == "dd" && args.iter().any(|a| a.starts_with("of=/dev/"))
}

fn rule_shred_dev(cmd: &str, args: &[String]) -> bool {
    cmd == "shred" && args.iter().any(|a| a.starts_with("/dev/"))
}

fn rule_wipefs(cmd: &str, _args: &[String]) -> bool {
    cmd == "wipefs"
}

fn rule_shutdown_halt(cmd: &str, _args: &[String]) -> bool {
    matches!(cmd, "shutdown" | "halt" | "poweroff")
}

fn rule_reboot(cmd: &str, _args: &[String]) -> bool {
    cmd == "reboot"
}

fn rule_passwd_root(cmd: &str, args: &[String]) -> bool {
    cmd == "passwd" && args.iter().any(|a| a == "root")
}

fn rule_chmod_777_root(cmd: &str, args: &[String]) -> bool {
    cmd == "chmod" && args.contains(&"777".to_string()) && args.iter().any(|a| a == "/" || a.starts_with("/*") || a.starts_with("--"))
}

// ─── WARNING rules ────────────────────────────────────────────────────────────

fn rule_sudo(cmd: &str, _args: &[String]) -> bool {
    cmd == "sudo"
}

fn rule_su_root(cmd: &str, args: &[String]) -> bool {
    cmd == "su" && args.iter().any(|a| a == "root")
}

fn rule_systemctl_stop(cmd: &str, args: &[String]) -> bool {
    cmd == "systemctl" && (args.contains(&"stop".to_string()) || args.contains(&"disable".to_string()))
}

fn rule_service_stop(cmd: &str, args: &[String]) -> bool {
    cmd == "service" && args.contains(&"stop".to_string())
}

fn rule_kill_all(cmd: &str, _args: &[String]) -> bool {
    matches!(cmd, "killall" | "pkill")
}

fn rule_crontab_r(cmd: &str, args: &[String]) -> bool {
    cmd == "crontab" && args.contains(&"-r".to_string())
}

fn rule_chmod_suid(cmd: &str, args: &[String]) -> bool {
    cmd == "chmod" && args.iter().any(|a| a == "+s" || a == "4755" || a == "6755")
}

fn rule_chown_root(cmd: &str, args: &[String]) -> bool {
    cmd == "chown" && args.iter().any(|a| a == "root")
}

// ─── rule table ───────────────────────────────────────────────────────────────

static BLOCKED_RULES: &[DangerRule] = &[
    DangerRule { name: "rm-rf-root", level: DangerLevel::Blocked, matcher: rule_rm_rf },
    DangerRule { name: "mkfs", level: DangerLevel::Blocked, matcher: rule_mkfs },
    DangerRule { name: "dd-overwrite-device", level: DangerLevel::Blocked, matcher: rule_dd_dev },
    DangerRule { name: "shred-device", level: DangerLevel::Blocked, matcher: rule_shred_dev },
    DangerRule { name: "wipefs", level: DangerLevel::Blocked, matcher: rule_wipefs },
    DangerRule { name: "shutdown-halt", level: DangerLevel::Blocked, matcher: rule_shutdown_halt },
    DangerRule { name: "reboot", level: DangerLevel::Blocked, matcher: rule_reboot },
    DangerRule { name: "passwd-root", level: DangerLevel::Blocked, matcher: rule_passwd_root },
    DangerRule { name: "chmod-777-root", level: DangerLevel::Blocked, matcher: rule_chmod_777_root },
];

static WARNING_RULES: &[DangerRule] = &[
    DangerRule { name: "sudo", level: DangerLevel::Warning, matcher: rule_sudo },
    DangerRule { name: "su-root", level: DangerLevel::Warning, matcher: rule_su_root },
    DangerRule { name: "systemctl-stop", level: DangerLevel::Warning, matcher: rule_systemctl_stop },
    DangerRule { name: "service-stop", level: DangerLevel::Warning, matcher: rule_service_stop },
    DangerRule { name: "killall-pkill", level: DangerLevel::Warning, matcher: rule_kill_all },
    DangerRule { name: "crontab-remove", level: DangerLevel::Warning, matcher: rule_crontab_r },
    DangerRule { name: "chmod-suid", level: DangerLevel::Warning, matcher: rule_chmod_suid },
    DangerRule { name: "chown-root", level: DangerLevel::Warning, matcher: rule_chown_root },
];

// ─── public API ───────────────────────────────────────────────────────────────

impl DangerDetectionService {
    pub fn new() -> Self {
        Self
    }

    /// Analyse a command and its arguments.
    /// `command` is the executable name, `args` are the individual argument tokens.
    pub fn analyse(&self, command: &str, args: &[String]) -> DetectionResult {
        // Check BLOCKED rules first
        for rule in BLOCKED_RULES {
            if (rule.matcher)(command, args) {
                return DetectionResult {
                    level: DangerLevel::Blocked,
                    matched_rule: Some(rule.name.to_string()),
                    message: format!(
                        "Command blocked by rule '{}'. This operation could cause irreversible system damage.",
                        rule.name
                    ),
                };
            }
        }

        // Check WARNING rules
        let mut highest = DangerLevel::Safe;
        let mut matched_rule: Option<String> = None;

        for rule in WARNING_RULES {
            if (rule.matcher)(command, args) {
                highest = DangerLevel::Warning;
                if matched_rule.is_none() {
                    matched_rule = Some(rule.name.to_string());
                }
            }
        }

        match highest {
            DangerLevel::Warning => DetectionResult {
                level: DangerLevel::Warning,
                matched_rule: matched_rule.clone(),
                message: format!(
                    "Command requires confirmation (rule: '{}'). Review carefully before executing.",
                    matched_rule.as_deref().unwrap_or("unknown")
                ),
            },
            _ => DetectionResult {
                level: DangerLevel::Safe,
                matched_rule: None,
                message: "Command appears safe.".to_string(),
            },
        }
    }
}

impl Default for DangerDetectionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::command::DangerLevel;

    fn svc() -> DangerDetectionService {
        DangerDetectionService::new()
    }

    #[test]
    fn test_rm_rf_blocked() {
        let r = svc().analyse("rm", &["-rf".into(), "/".into()]);
        assert_eq!(r.level, DangerLevel::Blocked);
    }

    #[test]
    fn test_mkfs_blocked() {
        let r = svc().analyse("mkfs.ext4", &["/dev/sda1".into()]);
        assert_eq!(r.level, DangerLevel::Blocked);
    }

    #[test]
    fn test_dd_dev_blocked() {
        let r = svc().analyse("dd", &["if=/dev/zero".into(), "of=/dev/sda".into()]);
        assert_eq!(r.level, DangerLevel::Blocked);
    }

    #[test]
    fn test_sudo_warning() {
        let r = svc().analyse("sudo", &["apt-get".into(), "update".into()]);
        assert_eq!(r.level, DangerLevel::Warning);
    }

    #[test]
    fn test_ls_safe() {
        let r = svc().analyse("ls", &["-la".into(), "/tmp".into()]);
        assert_eq!(r.level, DangerLevel::Safe);
    }

    #[test]
    fn test_ping_safe() {
        let r = svc().analyse("ping", &["-c".into(), "4".into(), "8.8.8.8".into()]);
        assert_eq!(r.level, DangerLevel::Safe);
    }

    #[test]
    fn test_shutdown_blocked() {
        let r = svc().analyse("shutdown", &["-h".into(), "now".into()]);
        assert_eq!(r.level, DangerLevel::Blocked);
    }

    #[test]
    fn test_reboot_blocked() {
        let r = svc().analyse("reboot", &[] as &[String]);
        assert_eq!(r.level, DangerLevel::Blocked);
    }

    #[test]
    fn test_systemctl_warning() {
        let r = svc().analyse("systemctl", &["stop".into(), "nginx".into()]);
        assert_eq!(r.level, DangerLevel::Warning);
    }

    #[test]
    fn test_rm_file_safe() {
        let r = svc().analyse("rm", &["file.txt".into()]);
        assert_eq!(r.level, DangerLevel::Safe);
    }

    #[test]
    fn test_rm_rf_no_root_safe() {
        let r = svc().analyse("rm", &["-rf".into(), "/tmp/logs".into()]);
        assert_eq!(r.level, DangerLevel::Safe);
    }

    #[test]
    fn test_safe_commands_not_over_blocked() {
        assert_eq!(svc().analyse("df", &["-h".into()]).level, DangerLevel::Safe);
        assert_eq!(svc().analyse("free", &["-m".into()]).level, DangerLevel::Safe);
        assert_eq!(svc().analyse("ps", &["aux".into()]).level, DangerLevel::Safe);
        assert_eq!(svc().analyse("cat", &["/etc/os-release".into()]).level, DangerLevel::Safe);
        assert_eq!(svc().analyse("uptime", &[] as &[String]).level, DangerLevel::Safe);
    }
}
