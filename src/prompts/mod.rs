pub mod mock;

use async_trait::async_trait;
use anyhow::Result;
use crate::core::types::{PromptMetadata, PromptResult, JsonValue};

/// Prompt trait - defines the interface for all MCP prompts
#[async_trait]
pub trait Prompt: Send + Sync {
    /// Get prompt metadata
    async fn metadata(&self) -> Result<PromptMetadata>;
    
    /// Generate prompt with given arguments
    async fn generate(&self, args: JsonValue) -> Result<PromptResult>;
    
    /// Validate prompt arguments
    async fn validate(&self, args: &JsonValue) -> Result<bool> {
        // Default implementation - can be overridden
        Ok(args.is_object())
    }
}
