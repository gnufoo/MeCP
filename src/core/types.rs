use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic JSON value type for flexible data representation
pub type JsonValue = serde_json::Value;

/// Resource metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetadata {
    pub name: String,
    pub description: String,
    pub mime_type: Option<String>,
    pub uri: String,
}

/// Resource content wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub content: JsonValue,
    pub metadata: Option<HashMap<String, String>>,
}

/// Tool parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: JsonValue,
    pub error: Option<String>,
}

/// Prompt metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
}

/// Prompt argument definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Prompt execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    pub messages: Vec<PromptMessage>,
}

/// Message in a prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: String,
}
