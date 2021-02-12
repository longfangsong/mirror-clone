use crate::common::{Mission, SnapshotConfig};
use crate::error::Result;
use crate::traits::{SnapshotStorage, TargetStorage};
use async_trait::async_trait;
use reqwest::{redirect::Policy, Client, ClientBuilder};
use slog::{info, warn};

#[derive(Debug)]
pub struct MirrorIntelConfig {
    base: String,
}

pub struct MirrorIntel {
    config: MirrorIntelConfig,
    client: Client,
}

impl MirrorIntel {
    pub fn new(base: String) -> Self {
        Self {
            config: MirrorIntelConfig { base },
            client: ClientBuilder::new()
                .user_agent("mirror-clone / 0.1 (siyuan.internal.sjtug.org)")
                .redirect(Policy::none())
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl SnapshotStorage<String> for MirrorIntel {
    async fn snapshot(
        &mut self,
        mission: Mission,
        _config: &SnapshotConfig,
    ) -> Result<Vec<String>> {
        let logger = mission.logger;
        let progress = mission.progress;
        let client = mission.client;

        info!(logger, "checking intel connection...");
        client.head(&self.config.base).send().await?;
        progress.finish_with_message("done");

        // We always return empty file list, and diff transfer will transfer
        // all objects.
        Ok(vec![])
    }

    fn info(&self) -> String {
        format!("mirror_intel, {:?}", self.config)
    }
}

#[async_trait]
impl TargetStorage<String> for MirrorIntel {
    async fn put_object(&self, item: String, mission: &Mission) -> Result<()> {
        let target_url = format!("{}/{}", self.config.base, item);
        let response = self.client.head(&target_url).send().await?;
        let headers = response.headers().clone();
        drop(response);

        if let Some(location) = headers.get("Location") {
            if !location.to_str().unwrap().contains("jcloud") {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        if let Some(queue_length) = headers.get("X-Intel-Queue-Length") {
            let queue_length: u64 = queue_length.to_str().unwrap().parse().unwrap();
            if queue_length > 16384 {
                warn!(mission.logger, "queue full, length={}", queue_length);
                tokio::time::sleep(std::time::Duration::from_secs(queue_length - 16384)).await;
            }
        }
        Ok(())
    }
}
