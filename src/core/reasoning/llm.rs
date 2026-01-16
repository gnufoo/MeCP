use async_trait::async_trait;
use anyhow::Result;
use super::types::{CompletionRequest, CompletionResponse, CompletionChunk};

/// LLM Provider trait - abstraction for interacting with Large Language Models
/// Supports providers like OpenAI, Anthropic, Google, Cohere, local models, etc.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Initialize the LLM provider with configuration
    async fn initialize(&mut self, config: LlmConfig) -> Result<()>;
    
    /// Generate a completion
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    
    /// Generate a streaming completion
    async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<CompletionChunk>> + Unpin + Send>>;
    
    /// Get available models
    fn available_models(&self) -> Vec<LlmModel>;
    
    /// Get provider name
    fn provider_name(&self) -> &str;
    
    /// Check if provider is initialized
    fn is_initialized(&self) -> bool;
}

/// LLM Configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: LlmProviderType,
    pub api_key: Option<String>,
    pub model: String,
    pub endpoint: Option<String>,
    pub timeout_seconds: Option<u64>,
}

impl LlmConfig {
    pub fn new(provider: LlmProviderType, model: String) -> Self {
        Self {
            provider,
            api_key: None,
            model,
            endpoint: None,
            timeout_seconds: Some(30),
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_endpoint(mut self, endpoint: String) -> Self {
        self.endpoint = Some(endpoint);
        self
    }
}

/// LLM Provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProviderType {
    OpenAI,
    Anthropic,
    Google,
    Cohere,
    HuggingFace,
    Local,
    Custom,
}

impl std::fmt::Display for LlmProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProviderType::OpenAI => write!(f, "OpenAI"),
            LlmProviderType::Anthropic => write!(f, "Anthropic"),
            LlmProviderType::Google => write!(f, "Google"),
            LlmProviderType::Cohere => write!(f, "Cohere"),
            LlmProviderType::HuggingFace => write!(f, "HuggingFace"),
            LlmProviderType::Local => write!(f, "Local"),
            LlmProviderType::Custom => write!(f, "Custom"),
        }
    }
}

/// Model information
#[derive(Debug, Clone)]
pub struct LlmModel {
    pub id: String,
    pub name: String,
    pub context_window: u32,
    pub supports_streaming: bool,
}

/// Mock implementation for testing
pub struct MockLlmProvider {
    config: Option<LlmConfig>,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self { config: None }
    }
}

impl Default for MockLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn initialize(&mut self, config: LlmConfig) -> Result<()> {
        self.config = Some(config);
        Ok(())
    }

    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let last_message = request
            .messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        Ok(CompletionResponse {
            content: format!("Mock response to: {}", last_message),
            role: super::types::Role::Assistant,
            finish_reason: Some("stop".to_string()),
            usage: Some(super::types::Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
            }),
        })
    }

    async fn stream_complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<Box<dyn futures::Stream<Item = Result<CompletionChunk>> + Unpin + Send>> {
        use futures::stream;
        let chunks = vec![
            Ok(CompletionChunk {
                content: "Mock ".to_string(),
                finish_reason: None,
            }),
            Ok(CompletionChunk {
                content: "streaming ".to_string(),
                finish_reason: None,
            }),
            Ok(CompletionChunk {
                content: "response".to_string(),
                finish_reason: Some("stop".to_string()),
            }),
        ];
        Ok(Box::new(stream::iter(chunks)))
    }

    fn available_models(&self) -> Vec<LlmModel> {
        vec![LlmModel {
            id: "mock-model".to_string(),
            name: "Mock Model".to_string(),
            context_window: 4096,
            supports_streaming: true,
        }]
    }

    fn provider_name(&self) -> &str {
        "Mock Provider"
    }

    fn is_initialized(&self) -> bool {
        self.config.is_some()
    }
}
