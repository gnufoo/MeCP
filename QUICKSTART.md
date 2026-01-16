# Quick Start Guide

Get started with MeCP in 5 minutes!

## Installation

```bash
# Clone or create your project
git clone <your-repo>
cd MeCP

# Build the project
cargo build --release
```

## Run the Demo

```bash
# Run the main server
cargo run

# Run database examples
cargo run --example database_usage

# Run LLM examples
cargo run --example llm_usage
```

## Basic Usage

### 1. Create a Simple Resource

```rust
use mecp::resources::Resource;
use mecp::core::types::{ResourceMetadata, ResourceContent};
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct MyResource {}

#[async_trait]
impl Resource for MyResource {
    async fn metadata(&self) -> Result<ResourceMetadata> {
        Ok(ResourceMetadata {
            name: "my_resource".to_string(),
            description: "My custom resource".to_string(),
            mime_type: Some("application/json".to_string()),
            uri: "custom://my/resource".to_string(),
        })
    }

    async fn read(&self) -> Result<ResourceContent> {
        Ok(ResourceContent {
            uri: "custom://my/resource".to_string(),
            content: json!({"data": "Hello from my resource!"}),
            metadata: None,
        })
    }

    async fn uri(&self) -> String {
        "custom://my/resource".to_string()
    }
}
```

### 2. Create a Simple Tool

```rust
use mecp::tools::{Tool, ToolMetadata};
use mecp::core::types::{ToolResult, ToolParameter, JsonValue};
use async_trait::async_trait;
use anyhow::Result;
use serde_json::json;

pub struct CalculatorTool {}

#[async_trait]
impl Tool for CalculatorTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        Ok(ToolMetadata {
            name: "calculator".to_string(),
            description: "Performs basic calculations".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "operation".to_string(),
                    description: "Operation: add, subtract, multiply, divide".to_string(),
                    required: true,
                    param_type: "string".to_string(),
                },
                ToolParameter {
                    name: "a".to_string(),
                    description: "First number".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                },
                ToolParameter {
                    name: "b".to_string(),
                    description: "Second number".to_string(),
                    required: true,
                    param_type: "number".to_string(),
                },
            ],
        })
    }

    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        let operation = params["operation"].as_str().unwrap_or("add");
        let a = params["a"].as_f64().unwrap_or(0.0);
        let b = params["b"].as_f64().unwrap_or(0.0);

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b == 0.0 {
                    return Ok(ToolResult {
                        success: false,
                        output: json!(null),
                        error: Some("Division by zero".to_string()),
                    });
                }
                a / b
            }
            _ => {
                return Ok(ToolResult {
                    success: false,
                    output: json!(null),
                    error: Some("Unknown operation".to_string()),
                });
            }
        };

        Ok(ToolResult {
            success: true,
            output: json!({
                "operation": operation,
                "a": a,
                "b": b,
                "result": result
            }),
            error: None,
        })
    }
}
```

### 3. Register Components with Server

```rust
use mecp::core::server::McpServer;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let server = McpServer::new();
    
    // Register your custom components
    server.register_resource(Box::new(MyResource {})).await;
    server.register_tool(Box::new(CalculatorTool {})).await;
    
    // Run the server
    server.run().await?;
    
    Ok(())
}
```

## Working with Databases

### Connect to a Vector Database

```rust
use mecp::core::database::{VectorDatabase, DatabaseConfig};
use mecp::core::database::vector::MockVectorDatabase;

#[tokio::main]
async fn main() -> Result<()> {
    let mut db = MockVectorDatabase::new();
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 6333,
        database: "my_vectors".to_string(),
        username: None,
        password: None,
        options: std::collections::HashMap::new(),
    };
    
    db.connect(config).await?;
    
    // Insert vectors
    let vector = Vector {
        id: "vec1".to_string(),
        values: vec![0.1, 0.2, 0.3],
        metadata: None,
    };
    db.insert(vector).await?;
    
    // Search
    let results = db.search(vec![0.1, 0.2, 0.3], 10, None).await?;
    
    Ok(())
}
```

### Connect to a Graph Database

```rust
use mecp::core::database::{GraphDatabase, DatabaseConfig};
use mecp::core::database::graph::MockGraphDatabase;

#[tokio::main]
async fn main() -> Result<()> {
    let mut db = MockGraphDatabase::new();
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 7687,
        database: "neo4j".to_string(),
        username: Some("neo4j".to_string()),
        password: Some("password".to_string()),
        options: std::collections::HashMap::new(),
    };
    
    db.connect(config).await?;
    
    // Execute Cypher query
    let result = db.query("MATCH (n:Person) RETURN n LIMIT 10").await?;
    
    Ok(())
}
```

### Connect to SQL Database

```rust
use mecp::core::database::{SqlDatabase, DatabaseConfig};
use mecp::core::database::sql::{MockSqlDatabase, DatabaseType};

#[tokio::main]
async fn main() -> Result<()> {
    let mut db = MockSqlDatabase::new(DatabaseType::PostgreSQL);
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "mydb".to_string(),
        username: Some("user".to_string()),
        password: Some("password".to_string()),
        options: std::collections::HashMap::new(),
    };
    
    db.connect(config).await?;
    
    // Query
    let result = db.query("SELECT * FROM users WHERE active = $1", vec![json!(true)]).await?;
    
    // Execute with transaction
    let mut tx = db.begin_transaction().await?;
    tx.execute("UPDATE users SET last_login = NOW() WHERE id = $1", vec![json!(1)]).await?;
    tx.commit().await?;
    
    Ok(())
}
```

## Working with LLMs

### Basic Completion

```rust
use mecp::core::reasoning::{LlmProvider, LlmConfig, LlmProviderType};
use mecp::core::reasoning::llm::MockLlmProvider;
use mecp::core::reasoning::types::{CompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<()> {
    let mut llm = MockLlmProvider::new();
    
    let config = LlmConfig::new(LlmProviderType::OpenAI, "gpt-4".to_string())
        .with_api_key(std::env::var("OPENAI_API_KEY")?);
    
    llm.initialize(config).await?;
    
    let request = CompletionRequest::new(vec![
        Message {
            role: Role::System,
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: Role::User,
            content: "What is Rust?".to_string(),
        },
    ])
    .with_temperature(0.7)
    .with_max_tokens(500);
    
    let response = llm.complete(request).await?;
    println!("Response: {}", response.content);
    
    Ok(())
}
```

### Streaming Completion

```rust
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let mut llm = MockLlmProvider::new();
    // ... initialize ...
    
    let request = CompletionRequest::new(vec![
        Message {
            role: Role::User,
            content: "Write a short poem".to_string(),
        },
    ]);
    
    let mut stream = llm.stream_complete(request).await?;
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        print!("{}", chunk.content);
        
        if chunk.finish_reason.is_some() {
            break;
        }
    }
    
    Ok(())
}
```

## Environment Setup

Create a `.env` file for configuration:

```bash
# Database Configuration
DATABASE_URL=postgresql://localhost/mydb
VECTOR_DB_URL=http://localhost:6333
GRAPH_DB_URL=bolt://localhost:7687

# LLM API Keys
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
GOOGLE_API_KEY=...

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

Load in your code:

```rust
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    // Your code here
}
```

## Testing

Run all tests:

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Test a specific module
cargo test --lib core::database

# Run with output
cargo test -- --nocapture
```

## Project Structure Tour

```
MeCP/
├── src/
│   ├── main.rs              # Server entry point
│   ├── lib.rs               # Library exports
│   ├── core/                # Core abstractions
│   │   ├── server.rs        # MCP server
│   │   ├── database/        # Database traits
│   │   └── reasoning/       # LLM interfaces
│   ├── resources/           # Resource implementations
│   │   └── mock.rs          # Example resource
│   ├── tools/               # Tool implementations
│   │   └── mock.rs          # Example tool
│   └── prompts/             # Prompt implementations
│       └── mock.rs          # Example prompt
├── examples/                # Usage examples
│   ├── database_usage.rs
│   └── llm_usage.rs
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md          # Detailed architecture
└── QUICKSTART.md           # This file
```

## Next Steps

1. **Read the Architecture Guide**: See `ARCHITECTURE.md` for detailed design patterns
2. **Implement Real Connectors**: Replace mock implementations with real database clients
3. **Add LLM Providers**: Implement OpenAI, Anthropic, or other providers
4. **Create Custom Components**: Build resources, tools, and prompts for your use case
5. **Deploy**: Use Docker or your preferred deployment method

## Common Patterns

### Error Handling

```rust
use anyhow::{Result, Context};

async fn my_function() -> Result<Data> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;
    Ok(data)
}
```

### Async Initialization

```rust
impl MyComponent {
    pub async fn new(config: Config) -> Result<Self> {
        let client = connect(&config).await?;
        Ok(Self { client })
    }
}
```

### Shared State

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SharedResource {
    cache: Arc<RwLock<HashMap<String, Data>>>,
}
```

## Troubleshooting

### Build Errors

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for issues
cargo check
```

### Runtime Issues

- Check database connectivity
- Verify API keys are set
- Review logs for error messages
- Ensure async runtime is properly configured

## Resources

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [Serde JSON](https://docs.serde.rs/serde_json/)
- [Anyhow Error Handling](https://docs.rs/anyhow/)

## Getting Help

- Check the `ARCHITECTURE.md` for design patterns
- Review the `examples/` directory
- Look at the mock implementations for reference
- Read the trait documentation in source files

Happy coding with MeCP! 🚀
