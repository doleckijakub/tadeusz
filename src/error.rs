#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OpenRouterError: {0}")]
    OpenRouter(#[from] openrouter_rs::error::OpenRouterError),

    #[error("Input/Output Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Serialization Error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Model tried to call an unknown tool: {0}")]
    UnknownTool(String),

    #[error("Request Error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("DuckDuckGo Error: {0}")]
    DuckDuckGo(String),
}

pub type Result<T> = std::result::Result<T, Error>;
