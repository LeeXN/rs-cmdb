use common::entity::permission::{CommandAction, CommandOverride, CommandRules};
use config::{Config, ConfigError, File};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Primary IP auto-detection configuration for client
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrimaryIpConfig {
    /// CIDR subnet for auto-detection of primary IP from NICs (e.g., "10.0.0.0/8")
    pub subnet: String,
}

/// 客户端配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientConfig {
    /// 客户端唯一标识符，如果未指定则自动生成
    pub client_id: Option<String>,
    /// 客户端主机名
    pub hostname: Option<String>,
    /// 服务器配置
    pub server: ServerConfig,
    /// 报告配置
    pub report: ReportConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// Primary IP auto-detection configuration
    pub primary_ip: Option<PrimaryIpConfig>,
    /// 远程执行命令过滤规则（命令执行专用）
    #[serde(default = "default_command_rules")]
    pub execution_command_rules: CommandRules,
    /// 兼容旧配置字段
    #[serde(default)]
    pub command_rules: CommandRules,
    /// 允许远程执行的命令白名单（兼容旧配置）
    #[serde(default = "default_allowed_commands")]
    pub allowed_commands: Vec<String>,
}

/// 服务器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 服务器地址
    pub url: String,
    /// 是否启用TLS验证（用于自签名证书）
    pub verify_tls: bool,
}

/// 报告配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReportConfig {
    /// 是否启用服务模式
    pub service_mode: bool,
    /// 是否启用推送模式（向服务器主动推送数据）
    pub push_enabled: bool,
    /// 推送间隔（秒）
    pub push_interval: u64,
    /// 是否启用拉取模式（接收服务器请求）
    pub pull_enabled: bool,
    /// 收集的硬件组件
    pub components: Vec<String>,
}

/// 日志配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志文件路径（可选）
    pub file: Option<String>,
}

/// 静态配置实例
static CONFIG: Lazy<ClientConfig> = Lazy::new(|| {
    match load_config() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            // 使用默认配置
            default_config()
        }
    }
});

/// Configuration file used by the running service. Keeping this path lets a
/// successful registration persist the canonical server-side client ID when
/// migrating an old configuration that contained an empty ID.
static ACTIVE_CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

/// 获取客户端配置
pub fn get_config() -> &'static ClientConfig {
    &CONFIG
}

/// 加载配置
pub fn load_config() -> Result<ClientConfig, ConfigError> {
    // 设置默认配置源
    let mut builder = Config::builder();

    // 使用默认配置
    let default_config = default_config();
    builder = builder.add_source(config::Config::try_from(&default_config).unwrap());

    // 添加配置文件
    builder = builder
        .add_source(File::from(PathBuf::from("config/client.toml")).required(false))
        .add_source(File::from(PathBuf::from("/etc/rs-cmdb/client.toml")).required(false));

    // 添加用户主目录的配置
    if let Ok(home) = env::var("HOME") {
        let home_config = PathBuf::from(home).join(".config/rs-cmdb/client.toml");
        builder = builder.add_source(File::from(home_config).required(false));
    }

    // 添加环境变量，前缀为 "CMDB_CLIENT_"
    builder = builder.add_source(config::Environment::with_prefix("CMDB_CLIENT").separator("_"));

    // 构建配置
    let config: ClientConfig = builder.build()?.try_deserialize()?;

    Ok(config)
}

/// 从指定的文件加载配置
pub fn load_from_file(path: &str) -> Result<ClientConfig, Box<dyn std::error::Error>> {
    // 确保文件存在
    if !PathBuf::from(path).exists() {
        return Err(format!("Config file not found: {}", path).into());
    }

    // 读取文件内容
    let content = fs::read_to_string(path)?;

    // 解析 TOML
    let config: ClientConfig = toml::from_str(&content)?;

    Ok(config)
}

/// 默认允许的命令白名单
fn default_allowed_commands() -> Vec<String> {
    vec![
        "ping".to_string(),
        "traceroute".to_string(),
        "df".to_string(),
        "free".to_string(),
        "uptime".to_string(),
        "uname".to_string(),
        "ip".to_string(),
        "ss".to_string(),
        "lscpu".to_string(),
        "lsblk".to_string(),
        "dmidecode".to_string(),
        "cat".to_string(),
        "echo".to_string(),
        "ls".to_string(),
        "grep".to_string(),
        "wc".to_string(),
        "head".to_string(),
        "tail".to_string(),
        "systemctl".to_string(),
        "journalctl".to_string(),
        "hostname".to_string(),
    ]
}

fn default_command_rules() -> CommandRules {
    CommandRules::default()
}

/// 默认配置
pub fn default_config() -> ClientConfig {
    ClientConfig {
        client_id: None,
        hostname: None,
        server: ServerConfig {
            url: "http://localhost:8080/api/v1".to_string(),
            verify_tls: true,
        },
        report: ReportConfig {
            service_mode: false,
            push_enabled: true,
            push_interval: 300, // 5分钟
            pull_enabled: true,
            components: vec![
                "sys".to_string(),
                "os".to_string(),
                "cpu".to_string(),
                "ram".to_string(),
                "disk".to_string(),
                "nic".to_string(),
                "gpu".to_string(),
                "ipmi".to_string(),
            ],
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            file: None,
        },
        primary_ip: None,
        execution_command_rules: default_command_rules(),
        command_rules: default_command_rules(),
        allowed_commands: default_allowed_commands(),
    }
}

/// 获取默认配置文件路径
pub fn get_default_config_path() -> PathBuf {
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config/rs-cmdb/client.toml")
    } else {
        PathBuf::from("config/client.toml")
    }
}

/// 检查默认配置文件是否存在
pub fn default_config_exists() -> bool {
    get_default_config_path().exists()
}

/// 保存配置到文件
pub fn save_config_to_file(
    config: &ClientConfig,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // 确保目录存在
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 将配置序列化为TOML
    let toml_str = toml::to_string_pretty(config)?;

    // 写入文件
    fs::write(path, toml_str)?;

    Ok(())
}

/// 保存配置到指定路径
pub fn save_to_file(path: String, config: &ClientConfig) -> Result<(), Box<dyn std::error::Error>> {
    save_config_to_file(config, &PathBuf::from(path))
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_string();
        (!value.is_empty()).then_some(value)
    })
}

/// Normalize legacy empty-string identity fields and ensure that the client ID
/// is persisted in the same file systemd actually loads.
pub fn prepare_service_config(
    mut config: ClientConfig,
    path: &std::path::Path,
) -> anyhow::Result<ClientConfig> {
    let original_client_id = config.client_id.clone();
    let original_hostname = config.hostname.clone();
    config.hostname = non_empty(config.hostname);
    config.client_id = non_empty(config.client_id);

    if config.client_id.is_none() {
        // Older versions could accidentally save the generated identity in
        // the user's default config while systemd loaded /etc/rs-cmdb/client.toml.
        // Recover that identity before generating a new one.
        let default_path = get_default_config_path();
        if default_path != path {
            if let Ok(legacy) = load_from_file(default_path.to_string_lossy().as_ref()) {
                config.client_id = non_empty(legacy.client_id);
            }
        }
    }

    if config.client_id.is_none() {
        config.client_id = Some(Uuid::new_v4().to_string());
    }

    if config.client_id != original_client_id || config.hostname != original_hostname {
        save_config_to_file(&config, &path.to_path_buf())
            .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    }
    let _ = ACTIVE_CONFIG_PATH.set(path.to_path_buf());
    Ok(config)
}

/// Persist the canonical ID returned by the server. This is important when an
/// old empty-ID registration is reconciled to an existing serial-number record.
pub fn persist_active_client_id(client_id: &str) -> anyhow::Result<()> {
    let client_id = client_id.trim();
    if client_id.is_empty() {
        return Err(anyhow::anyhow!("server returned an empty client ID"));
    }
    let path = ACTIVE_CONFIG_PATH
        .get()
        .ok_or_else(|| anyhow::anyhow!("active client configuration path is unavailable"))?;
    let mut config = load_from_file(path.to_string_lossy().as_ref())
        .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    if config.client_id.as_deref().map(str::trim) == Some(client_id)
        && config
            .hostname
            .as_deref()
            .is_none_or(|hostname| !hostname.trim().is_empty())
    {
        return Ok(());
    }
    config.client_id = Some(client_id.to_string());
    config.hostname = non_empty(config.hostname);
    save_config_to_file(&config, path).map_err(|err| anyhow::anyhow!(err.to_string()))
}

/// 确保客户端有一个持久化的ID
pub fn ensure_client_id() -> String {
    let default_path = get_default_config_path();

    // 如果默认配置文件不存在，创建一个包含客户端ID的配置
    if !default_path.exists() {
        let mut config = default_config();
        let new_id = Uuid::new_v4().to_string();
        config.client_id = Some(new_id.clone());

        // 尝试保存配置
        if let Err(e) = save_config_to_file(&config, &default_path) {
            eprintln!(
                "Warning: Failed to save default config with client ID: {}",
                e
            );
            return new_id;
        }

        new_id
    } else {
        // 从默认配置文件加载
        match fs::read_to_string(&default_path) {
            Ok(content) => match toml::from_str::<ClientConfig>(&content) {
                Ok(config) => {
                    if let Some(id) = config.client_id {
                        return id;
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse default config: {}", e);
                }
            },
            Err(e) => {
                eprintln!("Warning: Failed to read default config: {}", e);
            }
        }

        // 如果配置存在但无法获取ID，生成一个新ID并更新配置
        let new_id = Uuid::new_v4().to_string();

        // 尝试加载现有配置
        if let Ok(mut config) = load_from_file(default_path.to_str().unwrap()) {
            config.client_id = Some(new_id.clone());

            // 保存更新后的配置
            if let Err(e) = save_config_to_file(&config, &default_path) {
                eprintln!("Warning: Failed to update config with client ID: {}", e);
            }
        }

        new_id
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn empty_identity_fields_are_normalized_and_persisted() {
        let test_dir =
            std::env::temp_dir().join(format!("rs-cmdb-client-config-{}", Uuid::new_v4()));
        let config_path = test_dir.join("client.toml");
        let mut config = default_config();
        config.client_id = Some("   ".to_string());
        config.hostname = Some(String::new());
        save_config_to_file(&config, &config_path).unwrap();

        let prepared = prepare_service_config(config, &config_path).unwrap();
        let persisted = load_from_file(config_path.to_string_lossy().as_ref()).unwrap();

        assert!(prepared
            .client_id
            .as_deref()
            .is_some_and(|id| !id.is_empty()));
        assert_eq!(prepared.hostname, None);
        assert_eq!(persisted.client_id, prepared.client_id);
        assert_eq!(persisted.hostname, None);

        std::fs::remove_file(&config_path).unwrap();
        std::fs::remove_dir(&test_dir).unwrap();
    }
}
