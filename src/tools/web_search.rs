use async_trait::async_trait;
use duckduckgo::browser::Browser;
use serde::Deserialize;
use tool::{Tool, ToolResult};

#[derive(Default, Tool, Debug, Deserialize)]
#[tool(
    name = "web_search",
    description = "Perform a web search and return results"
)]
pub struct WebSearch {
    #[required]
    #[description("The search query to look up")]
    pub query: String,
}

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:124.0) Gecko/20100101 Firefox/124.0";

#[async_trait]
impl Tool for WebSearch {
    async fn execute(&self) -> ToolResult<String> {
        let browser = Browser::new();

        let results = browser
            .lite_search(&self.query, "wt-wt", Some(8), USER_AGENT)
            .await
            .map_err(|e| format!("DuckDuckGo Error: {e}"))?;

        if results.is_empty() {
            return Ok("No results found.".to_string());
        }

        let out = results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let snippet = r.snippet.trim();
                if snippet.is_empty() {
                    format!("{}. {}\n   {}", i + 1, r.title, r.url)
                } else {
                    format!("{}. {}\n   {}\n   {}", i + 1, r.title, snippet, r.url)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(out)
    }
}
