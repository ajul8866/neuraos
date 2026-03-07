//! HTTP client for NeuraOS API server.
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NeuraClient {
    pub(crate) client: Client,
    pub(crate) base_url: String,
    pub(crate) api_key: Option<String>,
}

impl NeuraClient {
    pub fn new(base_url: String, api_key: Option<String>) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(concat!("neura-cli/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { client, base_url, api_key })
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    fn inject_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key { req.header("X-API-Key", key) } else { req }
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<Value> {
        let req = self.inject_auth(self.client.get(self.url(path)));
        handle_response(req.send().await?).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> anyhow::Result<Value> {
        let req = self.inject_auth(self.client.post(self.url(path)).json(body));
        handle_response(req.send().await?).await
    }

    pub async fn delete(&self, path: &str) -> anyhow::Result<Value> {
        let req = self.inject_auth(self.client.delete(self.url(path)));
        handle_response(req.send().await?).await
    }

    pub async fn get_with_query(&self, path: &str, params: &[(&str, &str)]) -> anyhow::Result<Value> {
        let req = self.inject_auth(self.client.get(self.url(path)).query(params));
        handle_response(req.send().await?).await
    }
}

pub async fn handle_response(resp: reqwest::Response) -> anyhow::Result<Value> {
    let status = resp.status();
    let body = resp.text().await?;
    if status.is_success() {
        if body.trim().is_empty() { Ok(Value::Null) } else { Ok(serde_json::from_str(&body)?) }
    } else {
        let msg = serde_json::from_str::<Value>(&body).ok()
            .and_then(|v| v.get("error").or_else(|| v.get("message")).cloned())
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| body.clone());
        anyhow::bail!("HTTP {} — {}", status, msg)
    }
}
