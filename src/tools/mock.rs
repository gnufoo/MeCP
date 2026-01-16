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

/// Search tool - required for ChatGPT Connectors and deep research
/// Searches for information and returns results with URLs
pub struct SearchTool {
    name: String,
}

impl SearchTool {
    pub fn new() -> Self {
        Self {
            name: "search".to_string(),
        }
    }
}

#[async_trait]
impl Tool for SearchTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        Ok(ToolMetadata {
            name: self.name.clone(),
            description: "Search for information and return relevant results with URLs. Required for ChatGPT Connectors and deep research.".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "query".to_string(),
                    description: "Search query string".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                },
                ToolParameter {
                    name: "max_results".to_string(),
                    description: "Maximum number of results to return (default: 10)".to_string(),
                    required: false,
                    param_type: "number".to_string(),
                },
            ],
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let query = params
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: query"))?;

        let max_results = params
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(10) as usize;

        // Mock search results
        let mock_results = vec![
            json!({
                "title": format!("Result 1: Information about {}", query),
                "url": "https://example.com/result1",
                "snippet": format!("This is a mock search result for '{}'. It contains relevant information about the topic.", query),
                "relevance_score": 0.95
            }),
            json!({
                "title": format!("Result 2: Deep dive into {}", query),
                "url": "https://example.com/result2",
                "snippet": format!("Another mock result for '{}' with detailed information and context.", query),
                "relevance_score": 0.87
            }),
            json!({
                "title": format!("Result 3: {} - Complete Guide", query),
                "url": "https://example.com/result3",
                "snippet": format!("A comprehensive guide covering all aspects of '{}' with examples and best practices.", query),
                "relevance_score": 0.82
            }),
            json!({
                "title": format!("Result 4: {} Explained", query),
                "url": "https://example.com/result4",
                "snippet": format!("An explanation of '{}' with step-by-step instructions and visual aids.", query),
                "relevance_score": 0.78
            }),
            json!({
                "title": format!("Result 5: Advanced {} Techniques", query),
                "url": "https://example.com/result5",
                "snippet": format!("Advanced techniques and strategies for working with '{}' in production environments.", query),
                "relevance_score": 0.75
            }),
        ];

        // Limit results to max_results
        let results: Vec<_> = mock_results.into_iter().take(max_results).collect();

        Ok(ToolResult {
            success: true,
            output: json!({
                "query": query,
                "total_results": results.len(),
                "results": results,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }),
            error: None,
        })
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

/// Fetch tool - required for ChatGPT Connectors and deep research
/// Fetches content from a URL
pub struct FetchTool {
    name: String,
}

impl FetchTool {
    pub fn new() -> Self {
        Self {
            name: "fetch".to_string(),
        }
    }
}

#[async_trait]
impl Tool for FetchTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        Ok(ToolMetadata {
            name: self.name.clone(),
            description: "Fetch content from a URL. Required for ChatGPT Connectors and deep research.".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "url".to_string(),
                    description: "URL to fetch content from".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                },
            ],
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: url"))?;

        // Mock fetched content based on URL
        let (title, content) = if url.contains("result1") {
            (
                "Result 1 Content".to_string(),
                "This is mock content fetched from result1. It contains detailed information about the topic, including key concepts, examples, and practical applications. The content is structured and easy to understand.".to_string(),
            )
        } else if url.contains("result2") {
            (
                "Result 2 Content".to_string(),
                "This is mock content from result2. It provides a deep dive into the subject matter with technical details, code examples, and best practices. The content is comprehensive and well-organized.".to_string(),
            )
        } else if url.contains("result3") {
            (
                "Result 3 Content".to_string(),
                "This is mock content from result3. It serves as a complete guide covering all aspects of the topic, from basics to advanced concepts. Includes step-by-step instructions and real-world scenarios.".to_string(),
            )
        } else if url.contains("result4") {
            (
                "Result 4 Content".to_string(),
                "This is mock content from result4. It explains the topic in detail with visual aids, diagrams, and examples. The content is designed to be accessible to both beginners and experts.".to_string(),
            )
        } else if url.contains("result5") {
            (
                "Result 5 Content".to_string(),
                "This is mock content from result5. It covers advanced techniques and strategies for production environments. Includes performance optimization tips and troubleshooting guides.".to_string(),
            )
        } else {
            (
                "Fetched Content".to_string(),
                format!("This is mock content fetched from {}. The content includes relevant information, examples, and context about the requested topic. In a real implementation, this would fetch actual content from the URL.", url),
            )
        };

        Ok(ToolResult {
            success: true,
            output: json!({
                "url": url,
                "title": title,
                "content": content,
                "content_type": "text/html",
                "content_length": content.len(),
                "fetched_at": chrono::Utc::now().to_rfc3339(),
                "status": "success"
            }),
            error: None,
        })
    }
}

impl Default for FetchTool {
    fn default() -> Self {
        Self::new()
    }
}
