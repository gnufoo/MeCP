pub mod mock;

use async_trait::async_trait;
use anyhow::Result;
use crate::core::types::{ResourceMetadata, ResourceContent};

/// Resource trait - defines the interface for all MCP resources
#[async_trait]
pub trait Resource: Send + Sync {
    /// Get resource metadata
    async fn metadata(&self) -> Result<ResourceMetadata>;
    
    /// Read resource content
    async fn read(&self) -> Result<ResourceContent>;
    
    /// Check if resource exists
    async fn exists(&self) -> bool {
        true
    }
    
    /// Get resource URI
    async fn uri(&self) -> String;
}
