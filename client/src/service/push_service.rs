use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

use crate::config::ClientConfig;
use common::entity::hardware::Hardware;
use common::models::ClientHardwareInfo;

#[derive(Clone)]
pub struct PushService {
    config: Arc<ClientConfig>,
    client_id: String,
    hardware_cache: Arc<Mutex<Option<Hardware>>>,
}

impl PushService {
    /// 创建新的推送服务
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

    /// 推送硬件信息到服务器
    #[instrument(skip(self))]
    pub async fn push_hardware_info(&self) -> Result<()> {
        debug!("Preparing to push hardware information");

        // 获取缓存的硬件信息，如果没有则重新收集
        let hardware = {
            let cache = self.hardware_cache.lock().await;
            match &*cache {
                Some(hw) => hw.clone(),
                None => {
                    drop(cache); // 释放锁
                    crate::collector::collect_all_hardware_info()
                }
            }
        };
        debug!("Collected hardware information: {:?}", hardware);

        // 创建推送请求
        let now = Utc::now().to_rfc3339();
        let hardware_info = ClientHardwareInfo {
            client_id: self.client_id.clone(),
            hardware: Some(hardware),
            collected_at: now,
        };

        // 推送数据
        info!("Pushing hardware information to server");
        debug!("Hardware info: {:?}", hardware_info);
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(!self.config.server.verify_tls)
            .build()?;

        let response = client
            .post(format!(
                "{}/clients/{}/hardware",
                self.config.server.url, self.client_id
            ))
            .json(&hardware_info)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("Failed to push hardware information: {}", error_text);
            return Err(anyhow::anyhow!("Server returned error: {}", error_text));
        }

        info!("Hardware information pushed successfully");
        Ok(())
    }
}
