use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

use crate::core::types::{ResourceMetadata, ResourceContent};
use crate::resources::Resource;

/// Mock resource implementation for testing and demonstration
pub struct MockResource {
    name: String,
    uri: String,
}

impl MockResource {
    pub fn new() -> Self {
        Self {
            name: "mock_resource".to_string(),
            uri: "mock://example/resource".to_string(),
        }
    }
}

#[async_trait]
impl Resource for MockResource {
    async fn metadata(&self) -> Result<ResourceMetadata> {
        Ok(ResourceMetadata {
            name: self.name.clone(),
            description: "A mock resource for demonstration purposes".to_string(),
            mime_type: Some("application/json".to_string()),
            uri: self.uri.clone(),
        })
    }

    async fn read(&self) -> Result<ResourceContent> {
        Ok(ResourceContent {
            uri: self.uri.clone(),
            content: json!({
                "message": "This is mock resource data",
                "items": ["item1", "item2", "item3"],
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            metadata: None,
        })
    }

    async fn uri(&self) -> String {
        self.uri.clone()
    }
}

impl Default for MockResource {
    fn default() -> Self {
        Self::new()
    }
}
