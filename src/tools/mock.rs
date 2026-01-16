use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

use crate::core::types::{ToolParameter, ToolResult, JsonValue};
use crate::tools::{Tool, ToolMetadata};

/// Hello World tool - a simple mock tool implementation
pub struct HelloWorldTool {
    name: String,
}

impl HelloWorldTool {
    pub fn new() -> Self {
        Self {
            name: "hello_world".to_string(),
        }
    }
}

#[async_trait]
impl Tool for HelloWorldTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        Ok(ToolMetadata {
            name: self.name.clone(),
            description: "A simple hello world tool that greets users".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "name".to_string(),
                    description: "Name to greet".to_string(),
                    required: false,
                    param_type: "string".to_string(),
                },
            ],
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("World");

        let message = format!("Hello, {}! Welcome to MeCP.", name);

        Ok(ToolResult {
            success: true,
            output: json!({
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            error: None,
        })
    }
}

impl Default for HelloWorldTool {
    fn default() -> Self {
        Self::new()
    }
}
