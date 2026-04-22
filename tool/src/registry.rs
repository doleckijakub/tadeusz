use std::fmt::Debug;
use std::marker::PhantomData;

use async_trait::async_trait;
use serde::de::DeserializeOwned;

use crate::{Tool, ToolResult};

#[async_trait]
pub trait AnyTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> serde_json::Value;
    async fn call(&self, args: &str) -> ToolResult<String>;
}

#[derive(Default)]
pub struct TypedTool<T>(PhantomData<fn() -> T>);

#[async_trait]
impl<T> AnyTool for TypedTool<T>
where
    T: Tool + DeserializeOwned + Debug + Send + Sync,
{
    fn name(&self) -> &'static str {
        T::name()
    }

    fn description(&self) -> &'static str {
        T::description()
    }

    fn parameters(&self) -> serde_json::Value {
        T::parameters()
    }

    async fn call(&self, args: &str) -> ToolResult<String> {
        let tool: T =
            serde_json::from_str(args).map_err(|e| format!("Serialization error: {e}"))?;
        eprintln!("[*] {:?}", tool);
        tool.execute().await
    }
}

pub struct ToolRegistration(pub fn() -> Box<dyn AnyTool>);

inventory::collect!(ToolRegistration);
