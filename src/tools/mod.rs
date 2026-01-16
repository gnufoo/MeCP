pub mod mock;

use async_trait::async_trait;
use anyhow::Result;
use crate::core::types::{ToolParameter, ToolResult, JsonValue};

/// Tool metadata
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

/// Tool trait - defines the interface for all MCP tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get tool metadata
    async fn metadata(&self) -> Result<ToolMetadata>;
    
    /// Execute the tool with given parameters
    async fn execute(&self, params: JsonValue) -> Result<ToolResult>;
    
    /// Validate tool parameters
    async fn validate(&self, params: &JsonValue) -> Result<bool> {
        // Default implementation - can be overridden
        Ok(params.is_object())
    }
}
