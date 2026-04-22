use async_trait::async_trait;
use reqwest;
use serde::Deserialize;
use serde_json::json;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct WebFetch {
    pub url: String,
}

#[async_trait]
impl super::ToolType for WebFetch {
    fn name() -> &'static str {
        "web_fetch"
    }

    fn description() -> &'static str {
        "Fetch a URL and return the raw HTML text of the page"
    }

    fn parameters() -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The absolute URL to fetch"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self) -> Result<String> {
        let resp = reqwest::Client::new()
            .get(&self.url)
            .send()
            .await?
            .text()
            .await?;

        Ok(resp)
    }
}
