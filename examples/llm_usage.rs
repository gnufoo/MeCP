use mecp::core::reasoning::{LlmProvider, LlmConfig, LlmProviderType};
use mecp::core::reasoning::llm::MockLlmProvider;
use mecp::core::reasoning::types::{CompletionRequest, Message, Role};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== LLM Reasoning Interface Examples ===\n");

    // Basic completion example
    basic_completion_example().await?;
    
    // Streaming example
    streaming_example().await?;

    Ok(())
}

async fn basic_completion_example() -> Result<()> {
    println!("1. Basic Completion Example");
    println!("   (Supports: OpenAI, Anthropic, Google, Cohere, etc.)\n");

    let mut llm = MockLlmProvider::new();
    
    let config = LlmConfig::new(LlmProviderType::OpenAI, "gpt-4".to_string())
        .with_api_key("your-api-key".to_string());

    llm.initialize(config).await?;
    println!("   ✓ LLM provider initialized");

    let request = CompletionRequest::new(vec![
        Message {
            role: Role::System,
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: Role::User,
            content: "What is the capital of France?".to_string(),
        },
    ])
    .with_temperature(0.7)
    .with_max_tokens(100);

    let response = llm.complete(request).await?;
    println!("   ✓ Generated completion");
    println!("   Response: {}", response.content);
    
    if let Some(usage) = response.usage {
        println!("   Token usage: {} total ({} prompt + {} completion)",
            usage.total_tokens, usage.prompt_tokens, usage.completion_tokens);
    }
    
    println!();

    Ok(())
}

async fn streaming_example() -> Result<()> {
    use futures::StreamExt;

    println!("2. Streaming Completion Example\n");

    let mut llm = MockLlmProvider::new();
    
    let config = LlmConfig::new(LlmProviderType::Anthropic, "claude-3".to_string())
        .with_api_key("your-api-key".to_string());

    llm.initialize(config).await?;
    println!("   ✓ LLM provider initialized");

    let request = CompletionRequest::new(vec![
        Message {
            role: Role::User,
            content: "Tell me a short story".to_string(),
        },
    ]);

    let mut stream = llm.stream_complete(request).await?;
    print!("   Streaming response: ");
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        print!("{}", chunk.content);
        
        if chunk.finish_reason.is_some() {
            break;
        }
    }
    
    println!("\n   ✓ Streaming completed\n");

    Ok(())
}
