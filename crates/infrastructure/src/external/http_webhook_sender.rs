use async_trait::async_trait;
use std::time::Duration;

use delivery_domain::ports::WebhookSender;

pub struct HttpWebhookSender {
    client: reqwest::Client,
}

impl HttpWebhookSender {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build reqwest client");
        Self { client }
    }
}

#[async_trait]
impl WebhookSender for HttpWebhookSender {
    async fn send(
        &self,
        url: &str,
        payload: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<(), String> {
        let resp = self
            .client
            .post(url)
            .timeout(Duration::from_secs(timeout_secs))
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }
}
