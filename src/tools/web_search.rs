use duckduckgo::browser::Browser;
use duckduckgo::response::Response;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
pub struct WebSearch {
    pub query: String,
}

impl super::ToolType for WebSearch {
    fn name() -> &'static str {
        "web_search"
    }

    fn description() -> &'static str {
        "Perform a web search using DuckDuckGo and return a summary of results"
    }

    fn parameters() -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to look up"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self) -> Result<String, Box<dyn std::error::Error>> {
        let browser = Browser::new();

        let path = format!("?q={}", urlencoding::encode(&self.query));
        let resp: Response = browser.get_api_response(&path, None).await?;

        let mut out = String::new();

        if let Some(heading) = &resp.heading {
            out.push_str(&format!("Heading: {}\n\n", heading));
        }

        if let Some(abstract_text) = &resp.abstract_text {
            out.push_str(&format!("Abstract: {}\n", abstract_text));
            if let Some(source) = &resp.abstract_source {
                out.push_str(&format!("Source: {}\n", source));
            }
            if let Some(url) = &resp.abstract_url {
                out.push_str(&format!("URL: {}\n", url));
            }
            out.push('\n');
        }

        if let Some(definition) = &resp.definition {
            out.push_str(&format!("Definition: {}\n", definition));
            if let Some(source) = &resp.definition_source {
                out.push_str(&format!("Source: {}\n", source));
            }
            if let Some(url) = &resp.definition_url {
                out.push_str(&format!("URL: {}\n", url));
            }
            out.push('\n');
        }

        if let Some(answer) = &resp.answer {
            out.push_str(&format!("Answer: {}\n", answer));
            if let Some(answer_type) = &resp.answer_type {
                out.push_str(&format!("Type: {}\n", answer_type));
            }
            out.push('\n');
        }

        if !resp.related_topics.is_empty() {
            out.push_str("Related Topics:\n");
            for (i, topic) in resp.related_topics.iter().enumerate().take(5) {
                if let Some(text) = &topic.text {
                    out.push_str(&format!("  {}. {}", i + 1, text));
                    if let Some(url) = &topic.first_url {
                        out.push_str(&format!(" — {}", url));
                    }
                    out.push('\n');
                }
            }
            out.push('\n');
        }

        if !resp.results.is_empty() {
            out.push_str("Results:\n");
            for (i, result) in resp.results.iter().enumerate().take(5) {
                if let Some(text) = &result.get("text") {
                    out.push_str(&format!("  {}. {}", i + 1, text));
                    if let Some(url) = &result.get("first_url") {
                        out.push_str(&format!(" — {}", url));
                    }
                    out.push('\n');
                }
            }
        }

        if out.trim().is_empty() {
            Ok("No results found.".to_string())
        } else {
            Ok(out.trim_end().to_string())
        }
    }
}
