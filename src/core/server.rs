use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::resources::Resource;
use crate::tools::Tool;
use crate::prompts::Prompt;
use crate::core::types::{ResourceMetadata, ToolResult, JsonValue};
use crate::tools::ToolMetadata;

/// Main MCP Server structure
pub struct McpServer {
    resources: Arc<RwLock<Vec<Box<dyn Resource>>>>,
    tools: Arc<RwLock<Vec<Box<dyn Tool>>>>,
    prompts: Arc<RwLock<Vec<Box<dyn Prompt>>>>,
}

impl McpServer {
    /// Create a new MCP server instance
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(Vec::new())),
            tools: Arc::new(RwLock::new(Vec::new())),
            prompts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a resource
    pub async fn register_resource(&self, resource: Box<dyn Resource>) {
        let mut resources = self.resources.write().await;
        resources.push(resource);
    }

    /// Register a tool
    pub async fn register_tool(&self, tool: Box<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.push(tool);
    }

    /// Register a prompt
    pub async fn register_prompt(&self, prompt: Box<dyn Prompt>) {
        let mut prompts = self.prompts.write().await;
        prompts.push(prompt);
    }

    /// Get count of registered resources
    pub async fn resource_count(&self) -> usize {
        self.resources.read().await.len()
    }

    /// Get count of registered tools
    pub async fn tool_count(&self) -> usize {
        self.tools.read().await.len()
    }

    /// Get count of registered prompts
    pub async fn prompt_count(&self) -> usize {
        self.prompts.read().await.len()
    }

    /// List all registered resources
    pub async fn list_resources(&self) -> Result<Vec<ResourceMetadata>> {
        let resources = self.resources.read().await;
        let mut metadatas = Vec::new();
        
        for resource in resources.iter() {
            metadatas.push(resource.metadata().await?);
        }
        
        Ok(metadatas)
    }

    /// Read a specific resource by URI
    pub async fn read_resource(&self, uri: &str) -> Result<crate::core::types::ResourceContent> {
        let resources = self.resources.read().await;
        
        for resource in resources.iter() {
            if resource.uri().await == uri {
                return resource.read().await;
            }
        }
        
        Err(anyhow!("Resource not found: {}", uri))
    }

    /// List all registered tools
    pub async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
        let tools = self.tools.read().await;
        let mut metadatas = Vec::new();
        
        for tool in tools.iter() {
            metadatas.push(tool.metadata().await?);
        }
        
        Ok(metadatas)
    }

    /// Call a specific tool by name
    pub async fn call_tool(&self, name: &str, params: JsonValue) -> Result<ToolResult> {
        let tools = self.tools.read().await;
        
        for tool in tools.iter() {
            let metadata = tool.metadata().await?;
            if metadata.name == name {
                return tool.execute(params).await;
            }
        }
        
        Err(anyhow!("Tool not found: {}", name))
    }

    /// List all registered prompts
    pub async fn list_prompts(&self) -> Result<Vec<crate::core::types::PromptMetadata>> {
        let prompts = self.prompts.read().await;
        let mut metadatas = Vec::new();
        
        for prompt in prompts.iter() {
            metadatas.push(prompt.metadata().await?);
        }
        
        Ok(metadatas)
    }

    /// Get a specific prompt by name
    pub async fn get_prompt(&self, name: &str, args: JsonValue) -> Result<crate::core::types::PromptResult> {
        let prompts = self.prompts.read().await;
        
        for prompt in prompts.iter() {
            let metadata = prompt.metadata().await?;
            if metadata.name == name {
                return prompt.generate(args).await;
            }
        }
        
        Err(anyhow!("Prompt not found: {}", name))
    }

    /// Run the MCP server
    pub async fn run(&self) -> Result<()> {
        println!("\nMCP Server running...");
        println!("Press Ctrl+C to stop\n");

        // Demo: Execute registered components
        self.demo_execution().await?;

        Ok(())
    }

    /// Demo execution of registered components
    async fn demo_execution(&self) -> Result<()> {
        println!("=== Demo Execution ===\n");

        // Execute resources
        let resources = self.resources.read().await;
        for resource in resources.iter() {
            let metadata = resource.metadata().await?;
            println!("Resource: {}", metadata.name);
            let content = resource.read().await?;
            println!("  Content: {:?}\n", content);
        }

        // Execute tools
        let tools = self.tools.read().await;
        for tool in tools.iter() {
            let metadata = tool.metadata().await?;
            println!("Tool: {}", metadata.name);
            let result = tool.execute(serde_json::json!({})).await?;
            println!("  Result: {:?}\n", result);
        }

        // Execute prompts
        let prompts = self.prompts.read().await;
        for prompt in prompts.iter() {
            let metadata = prompt.metadata().await?;
            println!("Prompt: {}", metadata.name);
            let result = prompt.generate(serde_json::json!({})).await?;
            println!("  Result: {:?}\n", result);
        }

        Ok(())
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}
