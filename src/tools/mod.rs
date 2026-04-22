use openrouter_rs::{error::OpenRouterError, types::Tool};

pub mod web_search;
pub mod web_fetch;

pub trait ToolType {
    fn name() -> &'static str;

    fn description() -> &'static str;

    fn parameters() -> serde_json::Value;

    async fn execute(&self) -> Result<String, Box<dyn std::error::Error>>;
}

pub fn tool<T>() -> Result<Tool, OpenRouterError>
where
    T: ToolType,
{
    Tool::builder()
        .name(T::name())
        .description(T::description())
        .parameters(T::parameters())
        .build()
}
