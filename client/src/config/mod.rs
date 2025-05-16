use std::path::PathBuf;
use std::env;
use std::fs;
use config::{Config, ConfigError, File};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// 获取客户端配置
pub fn get_config() -> &'static ClientConfig {
    &CONFIG
}

/// 加载配置
fn load_config() -> Result<ClientConfig, ConfigError> {
    // 设置默认配置源
    let mut builder = Config::builder();
    
    // 使用默认配置
    let default_config = default_config();
    builder = builder.add_source(config::Config::try_from(&default_config).unwrap());
    
    // 添加配置文件
    builder = builder.add_source(File::from(PathBuf::from("config/client.toml")).required(false))
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
pub fn save_config_to_file(config: &ClientConfig, path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
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
            eprintln!("Warning: Failed to save default config with client ID: {}", e);
            return new_id;
        }
        
        return new_id;
    } else {
        // 从默认配置文件加载
        match fs::read_to_string(&default_path) {
            Ok(content) => {
                match toml::from_str::<ClientConfig>(&content) {
                    Ok(config) => {
                        if let Some(id) = config.client_id {
                            return id;
                        }
                    },
                    Err(e) => {
                        eprintln!("Warning: Failed to parse default config: {}", e);
                    }
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
        
        return new_id;
    }
} 