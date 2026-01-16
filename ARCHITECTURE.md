# MeCP Architecture Guide

## Overview

MeCP (Modular Context Protocol) is a Rust-based framework for building extensible MCP servers with comprehensive abstractions for databases and LLM reasoning.

## Core Design Principles

1. **Trait-Based Abstractions**: All major components use traits for maximum flexibility
2. **Async-First**: Built on `tokio` for efficient async/await operations
3. **Type Safety**: Leverage Rust's type system for compile-time guarantees
4. **Modularity**: Clear separation between resources, tools, prompts, and core infrastructure

## Architecture Layers

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│         (Your Custom Resources, Tools, Prompts)          │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────┐
│                   MCP Server Layer                       │
│         (Registration, Routing, Execution)               │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────┐
│                 Abstraction Layer                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Resources   │  │    Tools     │  │   Prompts    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                     │
┌────────────────────┴────────────────────────────────────┐
│              Infrastructure Layer                        │
│  ┌──────────────┐  ┌──────────────┐                     │
│  │  Databases   │  │  LLM/Reasoning│                     │
│  │  - Vector    │  │  - OpenAI    │                     │
│  │  - Graph     │  │  - Anthropic │                     │
│  │  - SQL       │  │  - Google    │                     │
│  └──────────────┘  └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Resources

**Purpose**: Provide access to data and content

**Trait Definition**:
```rust
#[async_trait]
pub trait Resource: Send + Sync {
    async fn metadata(&self) -> Result<ResourceMetadata>;
    async fn read(&self) -> Result<ResourceContent>;
    async fn exists(&self) -> bool;
    async fn uri(&self) -> String;
}
```

**Use Cases**:
- File system access
- Database queries
- API endpoints
- Configuration data
- External content

### 2. Tools

**Purpose**: Execute operations and transformations

**Trait Definition**:
```rust
#[async_trait]
pub trait Tool: Send + Sync {
    async fn metadata(&self) -> Result<ToolMetadata>;
    async fn execute(&self, params: JsonValue) -> Result<ToolResult>;
    async fn validate(&self, params: &JsonValue) -> Result<bool>;
}
```

**Use Cases**:
- Data processing
- API calls
- File operations
- Calculations
- System commands

### 3. Prompts

**Purpose**: Generate conversation contexts for LLMs

**Trait Definition**:
```rust
#[async_trait]
pub trait Prompt: Send + Sync {
    async fn metadata(&self) -> Result<PromptMetadata>;
    async fn generate(&self, args: JsonValue) -> Result<PromptResult>;
    async fn validate(&self, args: &JsonValue) -> Result<bool>;
}
```

**Use Cases**:
- System prompts
- Conversation templates
- Context generation
- Few-shot examples

## Database Abstractions

### Vector Database

**Purpose**: Similarity search and embedding storage

**Supported Databases**:
- Milvus
- Weaviate
- Milvus
- Qdrant
- ChromaDB
- FAISS

**Key Operations**:
```rust
async fn insert(&self, vector: Vector) -> Result<String>;
async fn search(&self, query_vector: Vec<f32>, top_k: usize, 
                filter: Option<JsonValue>) -> Result<Vec<VectorSearchResult>>;
async fn delete(&self, id: &str) -> Result<()>;
```

**Implementation Pattern**:
```rust
pub struct MilvusConnector {
    client: MilvusClient,
    index_name: String,
}

#[async_trait]
impl VectorDatabase for MilvusConnector {
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()> {
        // Initialize Milvus client
    }
    
    async fn search(&self, query_vector: Vec<f32>, top_k: usize, 
                    filter: Option<JsonValue>) -> Result<Vec<VectorSearchResult>> {
        // Call Milvus API
    }
}
```

### Graph Database

**Purpose**: Knowledge graphs and relationship queries

**Supported Databases**:
- Neo4j
- ArangoDB
- JanusGraph
- Amazon Neptune
- GraphQL endpoints

**Key Operations**:
```rust
async fn query(&self, query: &str) -> Result<GraphQueryResult>;
async fn create_node(&self, node: GraphNode) -> Result<String>;
async fn create_edge(&self, edge: GraphEdge) -> Result<String>;
async fn find_neighbors(&self, node_id: &str, direction: EdgeDirection, 
                        edge_label: Option<&str>) -> Result<Vec<GraphNode>>;
```

**Query Languages Supported**:
- Cypher (Neo4j)
- Gremlin (TinkerPop)
- GraphQL
- AQL (ArangoDB)

### SQL Database

**Purpose**: Relational data storage and queries

**Supported Databases**:
- MySQL
- PostgreSQL
- SQLite
- SQL Server
- Oracle

**Key Operations**:
```rust
async fn query(&self, sql: &str, params: Vec<JsonValue>) -> Result<SqlQueryResult>;
async fn execute(&self, sql: &str, params: Vec<JsonValue>) -> Result<u64>;
async fn begin_transaction(&self) -> Result<Box<dyn SqlTransaction>>;
```

**Transaction Support**:
```rust
#[async_trait]
pub trait SqlTransaction: Send + Sync {
    async fn commit(&mut self) -> Result<()>;
    async fn rollback(&mut self) -> Result<()>;
    async fn execute(&self, sql: &str, params: Vec<JsonValue>) -> Result<u64>;
}
```

## LLM Reasoning Interface

### Purpose

Provide unified interface to multiple LLM providers for:
- Completion generation
- Streaming responses
- Token usage tracking
- Provider abstraction

### Supported Providers

| Provider | Models | Streaming | Notes |
|----------|--------|-----------|-------|
| OpenAI | GPT-4, GPT-3.5 | ✓ | Full API support |
| Anthropic | Claude 3.x | ✓ | Latest models |
| Google | Gemini, PaLM | ✓ | Vertex AI support |
| Cohere | Command, Generate | ✓ | Enterprise features |
| HuggingFace | Various | Partial | Model hub integration |
| Local | Ollama, llama.cpp | ✓ | Self-hosted |

### Interface Design

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn initialize(&mut self, config: LlmConfig) -> Result<()>;
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream_complete(&self, request: CompletionRequest) 
        -> Result<Box<dyn Stream<Item = Result<CompletionChunk>> + Unpin + Send>>;
    fn available_models(&self) -> Vec<LlmModel>;
}
```

### Configuration

```rust
let config = LlmConfig::new(LlmProviderType::OpenAI, "gpt-4".to_string())
    .with_api_key(env::var("OPENAI_API_KEY")?)
    .with_endpoint("https://api.openai.com/v1".to_string());
```

### Request Parameters

```rust
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop: Option<Vec<String>>,
}
```

## Extension Patterns

### Adding a New Database Backend

1. Implement the appropriate trait (`VectorDatabase`, `GraphDatabase`, or `SqlDatabase`)
2. Handle connection management
3. Translate operations to native API calls
4. Handle errors and convert to `anyhow::Result`

Example:
```rust
pub struct MyVectorDB {
    client: MyClient,
}

#[async_trait]
impl VectorDatabase for MyVectorDB {
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()> {
        self.client = MyClient::new(&config.host, config.port)?;
        Ok(())
    }
    
    // Implement other methods...
}
```

### Adding a New LLM Provider

1. Implement the `LlmProvider` trait
2. Handle API authentication
3. Transform requests/responses
4. Implement streaming if supported

Example:
```rust
pub struct CustomLlmProvider {
    api_key: String,
    client: HttpClient,
}

#[async_trait]
impl LlmProvider for CustomLlmProvider {
    async fn initialize(&mut self, config: LlmConfig) -> Result<()> {
        self.api_key = config.api_key.ok_or(anyhow!("API key required"))?;
        Ok(())
    }
    
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Transform request and call API
    }
}
```

### Adding Custom Resources

1. Implement the `Resource` trait
2. Define metadata structure
3. Implement content reading logic
4. Register with server

Example:
```rust
pub struct DatabaseResource {
    connection: DatabaseConnection,
}

#[async_trait]
impl Resource for DatabaseResource {
    async fn metadata(&self) -> Result<ResourceMetadata> {
        Ok(ResourceMetadata {
            name: "database_query".to_string(),
            description: "Query database tables".to_string(),
            mime_type: Some("application/json".to_string()),
            uri: "db://localhost/mydb".to_string(),
        })
    }
    
    async fn read(&self) -> Result<ResourceContent> {
        let data = self.connection.query("SELECT * FROM users").await?;
        Ok(ResourceContent {
            uri: self.uri().await,
            content: serde_json::to_value(data)?,
            metadata: None,
        })
    }
}
```

## Concurrency and Safety

### Thread Safety

All traits require `Send + Sync`:
- Safe to send between threads
- Safe to share between threads
- All implementations must maintain these guarantees

### Async Runtime

- Built on `tokio` runtime
- All async operations are non-blocking
- Use `RwLock` for shared state (not `Mutex` to avoid blocking)

### Error Handling

- Use `anyhow::Result<T>` for flexible error handling
- Convert specific errors to `anyhow::Error` with context
- Propagate errors up the stack with `?`

Example:
```rust
async fn my_operation(&self) -> Result<Data> {
    let result = self.database.query("SELECT *")
        .await
        .context("Failed to query database")?;
    Ok(result)
}
```

## Performance Considerations

### Database Connections

- Use connection pooling for SQL databases
- Reuse clients for vector/graph databases
- Implement health checks and reconnection logic

### Caching

- Consider caching frequently accessed resources
- Use `Arc<RwLock<T>>` for shared cached state
- Implement TTL for cache entries

### Batching

- Batch vector insertions when possible
- Use prepared statements for SQL
- Group graph operations in transactions

## Testing Strategy

### Unit Tests

Test individual components in isolation:
```rust
#[tokio::test]
async fn test_vector_search() {
    let mut db = MockVectorDatabase::new();
    db.connect(test_config()).await.unwrap();
    
    let results = db.search(vec![0.1, 0.2], 5, None).await.unwrap();
    assert_eq!(results.len(), 5);
}
```

### Integration Tests

Test component interactions:
```rust
#[tokio::test]
async fn test_resource_with_database() {
    let resource = MyResource::with_database(test_db());
    let content = resource.read().await.unwrap();
    assert!(content.content.is_object());
}
```

### Mock Implementations

Use provided mock implementations for testing:
- `MockVectorDatabase`
- `MockGraphDatabase`
- `MockSqlDatabase`
- `MockLlmProvider`

## Security Considerations

### API Keys

- Never hardcode API keys
- Use environment variables or secure vaults
- Rotate keys regularly

### Input Validation

- Validate all user inputs
- Sanitize SQL queries (use parameterized queries)
- Validate vector dimensions
- Check prompt injection

### Rate Limiting

- Implement rate limiting for LLM calls
- Track token usage
- Set cost limits

## Deployment

### Configuration

Create a config file or use environment variables:
```bash
export DATABASE_URL="postgresql://localhost/mydb"
export VECTOR_DB_URL="http://localhost:6333"
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Docker

Example Dockerfile:
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/mecp /usr/local/bin/
CMD ["mecp"]
```

### Monitoring

- Log all database operations
- Track LLM token usage
- Monitor error rates
- Set up health checks

## Future Extensions

Potential areas for extension:

1. **Additional Databases**:
   - Redis for caching
   - MongoDB for document storage
   - Elasticsearch for full-text search

2. **Additional LLM Features**:
   - Function calling support
   - Vision models
   - Audio/speech models

3. **Protocol Support**:
   - WebSocket for real-time communication
   - gRPC for efficient RPC
   - Message queues (RabbitMQ, Kafka)

4. **Advanced Features**:
   - Distributed tracing
   - Circuit breakers
   - Automatic retries with exponential backoff
