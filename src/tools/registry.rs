use std::{fmt::Debug, marker::PhantomData};

use async_trait::async_trait;
use openrouter_rs::types::{Tool, ToolCall};
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::tools::ToolType;

#[async_trait]
pub trait ToolDyn {
    fn name(&self) -> &'static str;

    fn description(&self) -> &'static str;

    fn parameters(&self) -> serde_json::Value;

    async fn execute_json(&self, args: &str) -> Result<String>;
}

pub struct ToolWrapper<T>(PhantomData<T>);

#[async_trait]
impl<T> ToolDyn for ToolWrapper<T>
where
    T: Debug + ToolType + DeserializeOwned + Send + Sync,
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

    async fn execute_json(&self, args: &str) -> Result<String> {
        let tool: T = serde_json::from_str(args)?;
        println!("[*] {:?}", tool);
        tool.execute().await
    }
}

pub struct ToolFactory(pub fn() -> Box<dyn ToolDyn + Send + Sync>);

inventory::collect!(ToolFactory);

macro_rules! register_tool {
    ($t:ty) => {
        inventory::submit! {
            ToolFactory(|| {
                Box::new(ToolWrapper::<$t>(std::marker::PhantomData))
            })
        }
    };
}

register_tool!(super::web_search::WebSearch);
register_tool!(super::web_fetch::WebFetch);

pub fn all_tools() -> Result<Vec<Tool>> {
    inventory::iter::<ToolFactory>()
        .map(|tf| {
            let t = (tf.0)();

            Tool::builder()
                .name(t.name())
                .description(t.description())
                .parameters(t.parameters())
                .build()
                .map_err(Into::into)
        })
        .collect()
}

pub async fn dispatch(tool_call: &ToolCall) -> Result<String> {
    let name = &tool_call.function.name;
    let args = &tool_call.function.arguments;

    for tf in inventory::iter::<ToolFactory>() {
        let tool = (tf.0)();

        if tool.name() == name {
            return tool.execute_json(args).await;
        }
    }

    Err(Error::UnknownTool(name.to_string()))
}
