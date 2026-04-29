#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OpenRouterError: {0}")]
    OpenRouter(#[from] openrouter_rs::error::OpenRouterError),

    #[error("Input/Output Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Model tried to call an unknown tool: {0}")]
    UnknownTool(String),
}

pub type Result<T> = std::result::Result<T, Error>;
