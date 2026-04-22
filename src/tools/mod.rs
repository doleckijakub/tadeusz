use async_trait::async_trait;

use crate::error::Result;

pub mod web_fetch;
pub mod web_search;

pub mod registry;

#[async_trait]
pub trait ToolType: Send + Sync {
    fn name() -> &'static str;

    fn description() -> &'static str;

    fn parameters() -> serde_json::Value;

    async fn execute(&self) -> Result<String>;
}
