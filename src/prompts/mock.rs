use async_trait::async_trait;
use anyhow::Result;

use crate::core::types::{PromptMetadata, PromptArgument, PromptResult, PromptMessage, JsonValue};
use crate::prompts::Prompt;

/// Mock prompt implementation for testing and demonstration
pub struct MockPrompt {
    name: String,
}

impl MockPrompt {
    pub fn new() -> Self {
        Self {
            name: "mock_prompt".to_string(),
        }
    }
}

#[async_trait]
impl Prompt for MockPrompt {
    async fn metadata(&self) -> Result<PromptMetadata> {
        Ok(PromptMetadata {
            name: self.name.clone(),
            description: "A mock prompt that generates a conversation starter".to_string(),
            arguments: vec![
                PromptArgument {
                    name: "topic".to_string(),
                    description: "Topic for the conversation".to_string(),
                    required: false,
                },
            ],
        })
    }

    async fn generate(&self, args: JsonValue) -> Result<PromptResult> {
        let topic = args
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("general discussion");

        Ok(PromptResult {
            messages: vec![
                PromptMessage {
                    role: "system".to_string(),
                    content: format!(
                        "You are a helpful assistant discussing {}. Provide clear and concise responses.",
                        topic
                    ),
                },
                PromptMessage {
                    role: "user".to_string(),
                    content: format!("Let's start a conversation about {}.", topic),
                },
            ],
        })
    }
}

impl Default for MockPrompt {
    fn default() -> Self {
        Self::new()
    }
}
