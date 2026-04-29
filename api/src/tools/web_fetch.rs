use async_trait::async_trait;
use reqwest;
use serde::Deserialize;
use tool::{Tool, ToolResult};

#[derive(Default, Tool, Deserialize, Debug)]
#[tool(description = "Fetch a URL and return the contents")]
pub struct WebFetch {
    #[description("The fetched URL")]
    pub url: String,
}

#[async_trait]
impl Tool for WebFetch {
    async fn execute(&self) -> ToolResult<String> {
        let resp = reqwest::Client::new()
            .get(&self.url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?
            .text()
            .await
            .map_err(|e| format!("Parsing the response failed: {e}"))?;

        let md = html2md::rewrite_html(&resp, false);

        Ok(md)
    }
}
