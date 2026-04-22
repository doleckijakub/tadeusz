use crate::error::{Error, Result};
use openrouter_rs::types::{Tool as ApiTool, ToolCall};

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

pub async fn dispatch(call: &ToolCall) -> Result<String> {
    let name = &call.function.name;
    let args = &call.function.arguments;

    for r in inventory::iter::<tool::registry::ToolRegistration>() {
        let t = (r.0)();
        if t.name() == name {
            return t
                .call(args)
                .await
                .map_err(|e| Error::ToolExecution(e.to_string()));
        }
    }

    Err(Error::UnknownTool(name.clone()))
}
