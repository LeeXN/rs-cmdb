mod push_service;
mod pull_service;

pub use push_service::PushService;
pub use pull_service::PullService;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, debug, instrument};
use uuid::Uuid;
use tokio_cron_scheduler::{JobScheduler, Job};
use anyhow::{Result, Context};

use crate::config::ClientConfig;
use crate::collector::linux_collector;
use common::models::Client;
use common::entity::hardware::Hardware;

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
        // 使用配置中的客户端ID，此时应该已经确保存在
        let client_id = match &config.client_id {
            Some(id) => id.clone(),
            None => {
                // 不应该走到这里，因为我们在 load_client_config 中已确保配置中有 client_id
                // 但为了健壮性，如果走到这里，确保生成的ID被保存
                let id = Uuid::new_v4().to_string();
                error!("Warning: No client ID found in config, generating a new one");
                
                // 保存到默认配置
                let default_path = crate::config::get_default_config_path();
                let mut config_clone = (*config).clone();
                config_clone.client_id = Some(id.clone());
                if let Err(e) = crate::config::save_config_to_file(&config_clone, &default_path) {
                    error!("Warning: Failed to save client ID to config: {}", e);
                }
                
                id
            },
        };
        
        // 创建调度器
        let scheduler = JobScheduler::new().await.context("Failed to create job scheduler")?;
        
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
        self.register_client().await?;
        
        // 启动推送服务
        if self.config.report.push_enabled {
            self.start_push_service().await?;
        }
        
        // 启动拉取服务
        if self.config.report.pull_enabled {
            self.start_pull_service().await?;
        }
        
        // 启动调度器
        self.scheduler.start().await.context("Failed to start scheduler")?;
        
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
    async fn register_client(&self) -> Result<()> {
        info!("Registering client to server");
        
        let os_info = linux_collector::collect_os_info();
        let system_info = linux_collector::collect_system_info();
        let hostname = self.config.hostname.clone().unwrap_or_else(|| os_info.hostname.clone());
        
        let registration = Client {
            id: self.client_id.clone(),
            hostname,
            serial_number: Some(system_info.serial_number.clone()),
            sys_vendor: Some(system_info.sys_vendor.clone()),
            product_name: Some(system_info.product_name.clone()),
            ip_address: os_info.ip_address,
            os: Some(format!("{}-{}", os_info.name.clone(), os_info.version.clone())),
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
            warranty_expiration: None,
            supplier: None,
            power_consumption: None,
        };
        
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!self.config.server.verify_tls)
            .build()?;
        
        let response = client
            .post(format!("{}/clients/register", self.config.server.url))
            .json(&registration)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Failed to register client: {:?}", error_text);
            return Err(anyhow::anyhow!("Server returned error: {:?}", error_text));
        }
        
        info!("Client registered successfully");
        Ok(())
    }
    
    /// 启动推送服务
    #[instrument(skip(self))]
    async fn start_push_service(&self) -> Result<()> {
        info!("Starting push service");
        
        let push_service = PushService::new(
            self.config.clone(),
            self.client_id.clone(),
            self.hardware_cache.clone(),
        );
        
        // 设置定时任务，定期推送硬件信息
        let interval = self.config.report.push_interval;
        let push_svc = push_service.clone();
        
        let job = Job::new_async(format!("0 */{} * * * *", interval / 60).as_str(), move |_, _| {
            let push = push_svc.clone();
            Box::pin(async move {
                if let Err(e) = push.push_hardware_info().await {
                    error!("Failed to push hardware info: {}", e);
                }
            })
        }).context("Failed to create push job")?;
        
        self.scheduler.add(job).await.context("Failed to add push job to scheduler")?;
        
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
} 