use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error, debug, instrument};
use anyhow::Result;

use crate::config::ClientConfig;
use common::entity::hardware::Hardware;
use common::models::{PullRequest, PullResponse};

#[derive(Clone)]
pub struct PullService {
    config: Arc<ClientConfig>,
    client_id: String,
    hardware_cache: Arc<Mutex<Option<Hardware>>>,
}

impl PullService {
    /// 创建新的拉取服务
    pub fn new(
        config: Arc<ClientConfig>,
        client_id: String,
        hardware_cache: Arc<Mutex<Option<Hardware>>>,
    ) -> Self {
        Self {
            config,
            client_id,
            hardware_cache,
        }
    }
    
    /// 处理服务器的拉取请求
    #[instrument(skip(self, request))]
    pub async fn handle_pull_request(&self, request: PullRequest) -> Result<()> {
        info!("Handling pull request: {}", request.request_id);
        
        // 根据请求收集指定的硬件信息
        let hardware = match request.components.is_empty() {
            true => {
                // 如果没有指定组件，使用缓存或收集全部
                let cache = self.hardware_cache.lock().await;
                match &*cache {
                    Some(hw) => hw.clone(),
                    None => {
                        drop(cache);
                        crate::collector::collect_all_hardware_info()
                    }
                }
            },
            false => {
                // 如果指定了组件，仅收集这些组件
                crate::collector::linux_collector::collect_hardware_info(&request.components)
            }
        };
        
        // 创建响应
        let response = PullResponse {
            request_id: request.request_id.clone(),
            hardware: Some(hardware),
            status: "success".to_string(),
            error: None,
        };
        
        // 发送响应回服务器
        debug!("Sending pull response");
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!self.config.server.verify_tls)
            .build()?;
            
        let response_url = format!("{}/clients/{}/hardware/pull/response", 
            self.config.server.url, self.client_id);
            
        let result = client
            .post(&response_url)
            .json(&response)
            .send()
            .await?;
            
        if !result.status().is_success() {
            let error_text = result.text().await?;
            error!("Failed to send pull response: {}", error_text);
            return Err(anyhow::anyhow!("Server returned error: {}", error_text));
        }
        
        info!("Pull response sent successfully");
        Ok(())
    }
    
    /// 检查是否有挂起的拉取请求
    #[instrument(skip(self))]
    pub async fn check_pending_requests(&self) -> Result<()> {
        debug!("Checking for pending pull requests");
        
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!self.config.server.verify_tls)
            .build()?;
            
        let pending_url = format!("{}/clients/{}/hardware/pull/pending", 
            self.config.server.url, self.client_id);
            
        let response = client
            .get(&pending_url)
            .send()
            .await?;
            
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                debug!("No pending pull requests");
                return Ok(());
            }
            
            let error_text = response.text().await?;
            error!("Failed to check pending pull requests: {}", error_text);
            return Err(anyhow::anyhow!("Server returned error: {}", error_text));
        }
        
        // 解析挂起的请求
        let requests: Vec<PullRequest> = response.json().await?;
        
        if requests.is_empty() {
            debug!("No pending pull requests");
            return Ok(());
        }
        
        info!("Found {} pending pull requests", requests.len());
        
        // 处理所有挂起的请求
        for request in requests {
            if let Err(e) = self.handle_pull_request(request).await {
                error!("Failed to handle pull request: {}", e);
            }
        }
        
        Ok(())
    }
} 