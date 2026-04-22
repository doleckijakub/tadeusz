pub use tool_derive::Tool;

pub mod registry;

use async_trait::async_trait;

pub type ToolResult<T> = std::result::Result<T, String>;

pub trait ToolSchema: Send + Sync + 'static {
    fn name() -> &'static str
    where
        Self: Sized;

    fn description() -> &'static str
    where
        Self: Sized;

    fn parameters() -> serde_json::Value
    where
        Self: Sized;
}

#[async_trait]
pub trait Tool: ToolSchema {
    async fn execute(&self) -> ToolResult<String>;
}
