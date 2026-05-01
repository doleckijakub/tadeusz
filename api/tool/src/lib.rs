pub use tool_derive::Tool;
pub use tool_derive::ToolFieldSchema;

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

pub trait ToolFieldSchema {
    fn field_schema() -> serde_json::Value;
}

impl ToolFieldSchema for String {
    fn field_schema() -> serde_json::Value {
        serde_json::json!({"type": "string"})
    }
}

impl ToolFieldSchema for bool {
    fn field_schema() -> serde_json::Value {
        serde_json::json!({"type": "boolean"})
    }
}

macro_rules! impl_integer {
    ($($t:ty),+) => {
        $(impl ToolFieldSchema for $t {
            fn field_schema() -> serde_json::Value {
                serde_json::json!({"type": "integer"})
            }
        })+
    };
}

impl_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, isize, usize);

impl ToolFieldSchema for f32 {
    fn field_schema() -> serde_json::Value {
        serde_json::json!({"type": "number"})
    }
}

impl ToolFieldSchema for f64 {
    fn field_schema() -> serde_json::Value {
        serde_json::json!({"type": "number"})
    }
}

impl<T: ToolFieldSchema> ToolFieldSchema for Option<T> {
    fn field_schema() -> serde_json::Value {
        T::field_schema()
    }
}

impl<T: ToolFieldSchema> ToolFieldSchema for Vec<T> {
    fn field_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "array",
            "items": T::field_schema()
        })
    }
}
