use crate::error::{Error, Result};
use openrouter_rs::types::{Tool as ApiTool, ToolCall};

pub use tool::registry::PreparedAnyTool;

pub fn all_tools() -> Result<Vec<ApiTool>> {
    inventory::iter::<tool::registry::ToolRegistration>()
        .map(|r| {
            let t = (r.0)();
            ApiTool::builder()
                .name(t.name())
                .description(t.description())
                .parameters(t.parameters())
                .build()
                .map_err(Into::into)
        })
        .collect()
}

pub fn create(call: &ToolCall) -> Result<Box<dyn PreparedAnyTool>> {
    let name = &call.function.name;
    let args = &call.function.arguments;

    for r in inventory::iter::<tool::registry::ToolRegistration>() {
        let t = (r.0)();
        if t.name() == name {
            return (r.1)(args).map_err(Error::ToolExecution);
        }
    }

    Err(Error::UnknownTool(name.clone()))
}
