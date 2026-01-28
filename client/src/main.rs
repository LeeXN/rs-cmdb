use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::signal;
use tracing::info;

use crate::config::ClientConfig;
use crate::display::{print_hardware_info, DisplayOptions, HardwareType};

pub mod collector;
pub mod config;
pub mod display;
pub mod service;
#[cfg(test)]
mod tests;

// 命令行参数
#[derive(Parser)]
#[command(name = "rs-cmdb-client")]
#[command(about = "Configuration Management Database Client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,
}

#[derive(Args)]
struct DetailFlag {
    /// 显示详细信息
    #[arg(short, long)]
    detail: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 以服务模式运行，定期上报硬件信息
    Service {
        /// 服务器URL
        #[arg(short, long)]
        server: Option<String>,

        /// 推送间隔（秒）
        #[arg(short, long)]
        interval: Option<u64>,

        /// 客户端ID（如果未指定则自动生成）
        #[arg(long)]
        client_id: Option<String>,
    },

    /// 生成配置文件模板
    GenerateConfig {
        /// 配置文件输出路径
        #[arg(short, long, default_value = "client.toml")]
        output: String,

        /// 服务器URL
        #[arg(short, long)]
        server: Option<String>,
    },

    /// 显示系统信息
    System(DetailFlag),

    /// 显示OS信息
    Os(DetailFlag),

    /// 显示CPU信息
    Cpu(DetailFlag),

    /// 显示磁盘信息
    Disk(DetailFlag),

    /// 显示网卡信息
    Nic(DetailFlag),

    /// 显示GPU信息
    Gpu(DetailFlag),

    /// 显示内存信息
    Ram(DetailFlag),

    /// 显示IPMI/BMC信息
    Ipmi(DetailFlag),

    /// 显示所有硬件信息
    All(DetailFlag),
}

/// 加载配置
fn load_client_config(config_path: Option<&str>) -> Arc<ClientConfig> {
    if let Some(path) = config_path {
        // 用户指定了配置文件，如果加载失败则使用默认配置
        match config::load_from_file(path) {
            Ok(cfg) => return Arc::new(cfg),
            Err(e) => {
                eprintln!("Warning: Failed to load config from {}: {}", path, e);
                // 继续使用默认配置
            }
        }
    }

    // 尝试从默认位置加载配置
    if config::default_config_exists() {
        let default_path = config::get_default_config_path();
        if let Ok(cfg) = config::load_from_file(default_path.to_str().unwrap()) {
            return Arc::new(cfg);
        }
    }

    // 确保我们有一个客户端ID
    let client_id = config::ensure_client_id();

    // 触发lazy静态配置的加载
    let mut config = config::get_config().clone();

    // 使用确保的客户端ID
    config.client_id = Some(client_id);

    Arc::new(config)
}

/// 配置日志
fn setup_logging(config: &ClientConfig) -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_level = config.logging.level.clone();
    let env_filter = tracing_subscriber::EnvFilter::new(&log_level);

    if let Some(log_file) = &config.logging.file {
        let path = std::path::Path::new(log_file);
        let directory = path.parent().unwrap_or(std::path::Path::new("."));
        let filename = path
            .file_name()
            .unwrap_or(std::ffi::OsStr::new("rs-cmdb-client.log"));

        let file_appender = tracing_appender::rolling::never(directory, filename);
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(non_blocking)
            .init();

        Some(guard)
    } else {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 加载配置
    let config = load_client_config(cli.config.as_deref());

    // 设置日志
    let _guard = setup_logging(&config);

    match cli.command {
        Some(Commands::Service {
            server,
            interval,
            client_id,
        }) => {
            // 服务模式
            info!("Starting client in service mode");

            // 应用命令行参数覆盖配置
            let mut config = (*config).clone();

            if let Some(server_url) = server {
                config.server.url = server_url;
            }

            if let Some(push_interval) = interval {
                config.report.push_interval = push_interval;
            }

            if let Some(id) = client_id {
                config.client_id = Some(id);
            }

            // 强制启用服务模式
            config.report.service_mode = true;

            // 启动客户端服务
            let client_service = service::ClientService::new(Arc::new(config))
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            client_service
                .start()
                .await
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            // 等待终止信号
            shutdown_signal().await;
            info!("Client service shutting down");
        }
        Some(Commands::GenerateConfig { output, server }) => {
            // 生成配置文件模板
            let mut config = config::default_config();

            if let Some(server_url) = server {
                config.server.url = server_url;
            }

            // 生成新的客户端ID
            config.client_id = Some(uuid::Uuid::new_v4().to_string());

            let output_path = output.clone();
            config::save_to_file(output, &config).map_err(|e| anyhow::anyhow!(e.to_string()))?;
            println!("Config template generated successfully: {}", output_path);
        }
        Some(command) => {
            // 直接显示硬件信息模式
            let (hardware_type, detail) = match command {
                Commands::System(flag) => (HardwareType::SYS, flag.detail),
                Commands::Os(flag) => (HardwareType::OS, flag.detail),
                Commands::Cpu(flag) => (HardwareType::CPU, flag.detail),
                Commands::Disk(flag) => (HardwareType::Disk, flag.detail),
                Commands::Nic(flag) => (HardwareType::NIC, flag.detail),
                Commands::Gpu(flag) => (HardwareType::GPU, flag.detail),
                Commands::Ram(flag) => (HardwareType::RAM, flag.detail),
                Commands::Ipmi(flag) => (HardwareType::IPMI, flag.detail),
                Commands::All(flag) => (HardwareType::All, flag.detail),
                _ => unreachable!(),
            };

            let mut options = DisplayOptions {
                hardware_types: HashSet::new(),
                show_detail: detail,
            };

            options.hardware_types.insert(hardware_type);
            print_hardware_info(&options);
        }
        None => {
            // 无子命令，默认显示所有信息
            let mut options = DisplayOptions {
                hardware_types: HashSet::new(),
                show_detail: false,
            };

            options.hardware_types.insert(HardwareType::All);
            print_hardware_info(&options);
        }
    }

    Ok(())
}

// 等待终止信号
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Received Ctrl+C, starting graceful shutdown..."); },
        _ = terminate => { info!("Received terminate signal, starting graceful shutdown..."); },
    }
}
