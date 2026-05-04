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
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0")
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

        let resp = client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {e}"))?;

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if content_type.contains("text/html") || content_type.is_empty() {
            let text = resp
                .text()
                .await
                .map_err(|e| format!("Parsing the response failed: {e}"))?;
            Ok(html2md::rewrite_html(&text, false))
        } else if content_type.contains("application/pdf") {
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| format!("Reading response failed: {e}"))?;
            let text = pdf_extract::extract_text_from_mem(&bytes)
                .map_err(|e| format!("PDF extraction failed: {e}"))?;
            Ok(text)
        } else {
            Ok(format!(
                "Unsupported data type: Tadeusz cannot yet read {content_type} documents"
            ))
        }
    }
}
