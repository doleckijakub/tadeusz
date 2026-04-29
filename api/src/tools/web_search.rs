use async_trait::async_trait;
use serde::Deserialize;
use tool::{Tool, ToolResult};

#[derive(Default, Tool, Debug, Deserialize)]
#[tool(description = "Perform a web search and return results")]
pub struct WebSearch {
    #[description("The search query to look up")]
    pub query: String,
}

#[derive(Deserialize)]
struct SearxngResponse {
    results: Vec<SearxngResult>,
}

#[derive(Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

#[async_trait]
impl Tool for WebSearch {
    async fn execute(&self) -> ToolResult<String> {
        let base_url = std::env::var("SEARXNG_URL").expect("SEARXNG_URL not set");

        let response = reqwest::Client::new()
            .get(format!("{base_url}/search"))
            .query(&[("q", &self.query), ("format", &"json".to_string())])
            .send()
            .await
            .map_err(|e| format!("SearXNG request error: {e}"))?
            .json::<SearxngResponse>()
            .await
            .map_err(|e| format!("SearXNG parse error: {e}"))?;

        if response.results.is_empty() {
            return Ok("No results found.".to_string());
        }

        let out = response
            .results
            .iter()
            .take(8)
            .enumerate()
            .map(|(i, r)| {
                let content = r.content.as_deref().unwrap_or("").trim();
                if content.is_empty() {
                    format!("{}. {}\n   {}", i + 1, r.title, r.url)
                } else {
                    format!("{}. {}\n   {}\n   {}", i + 1, r.title, content, r.url)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(out)
    }
}
