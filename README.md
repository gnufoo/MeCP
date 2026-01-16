# MeCP - Modular Context Protocol Server

A Rust-based Model Context Protocol (MCP) server with modular architecture, comprehensive abstraction layers, and integrated database service management.

## 🚀 Quick Start

```bash
# Build the project and CLI
cargo build --release

# Install and start all database services (MySQL, Neo4j)
./target/release/mecp-cli start

# Check status
./target/release/mecp-cli status

# Run the MCP server
cargo run
```

## 📚 Documentation

- **[SETUP_COMPLETE.md](SETUP_COMPLETE.md)** - 🎯 **START HERE** - Complete setup guide
- **[DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)** - 📊 **NEW!** Monitoring Dashboard
- **[QUICKREF_CLI.md](QUICKREF_CLI.md)** - Quick reference for CLI commands
- **[DATABASE_SETUP.md](DATABASE_SETUP.md)** - Detailed database setup guide
- **[DASHBOARD.md](DASHBOARD.md)** - Complete dashboard documentation
- **[INSTALLATION.md](INSTALLATION.md)** - Step-by-step installation
- **[CLI_USAGE.md](CLI_USAGE.md)** - Complete CLI reference
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture
- **[QUICKSTART.md](QUICKSTART.md)** - API usage guide

## Project Structure

```
MeCP/
├── src/
│   ├── main.rs                 # Entry point
│   ├── core/                   # Core abstractions
│   │   ├── mod.rs
│   │   ├── server.rs           # MCP server implementation
│   │   ├── types.rs            # Common types
│   │   ├── database/           # Database abstractions
│   │   │   ├── mod.rs
│   │   │   ├── types.rs        # Database types
│   │   │   ├── vector.rs       # Vector DB trait (Milvus, Weaviate, etc.)
│   │   │   ├── graph.rs        # Graph DB trait (Neo4j, GraphQL, etc.)
│   │   │   └── sql.rs          # SQL DB trait (MySQL, PostgreSQL, etc.)
│   │   └── reasoning/          # LLM reasoning abstractions
│   │       ├── mod.rs
│   │       ├── types.rs        # Reasoning types
│   │       └── llm.rs          # LLM provider trait
│   ├── resources/              # Resource implementations
│   │   ├── mod.rs
│   │   └── mock.rs             # Mock resource (get_mock_resource)
│   ├── tools/                  # Tool implementations
│   │   ├── mod.rs
│   │   └── mock.rs             # HelloWorld tool
│   └── prompts/                # Prompt implementations
│       ├── mod.rs
│       └── mock.rs             # Mock prompt
├── Cargo.toml
└── README.md
```

## Features

### 1. Modular MCP Components

#### Resources
- **Interface**: `Resource` trait in `src/resources/mod.rs`
- **Mock Implementation**: `MockResource` - Returns sample JSON data
- **Methods**: `metadata()`, `read()`, `exists()`, `uri()`

#### Tools
- **Interface**: `Tool` trait in `src/tools/mod.rs`
- **Mock Implementation**: `HelloWorldTool` - Simple greeting tool
- **Methods**: `metadata()`, `execute()`, `validate()`

#### Prompts
- **Interface**: `Prompt` trait in `src/prompts/mod.rs`
- **Mock Implementation**: `MockPrompt` - Generates conversation starters
- **Methods**: `metadata()`, `generate()`, `validate()`

### 2. Database Abstractions

#### Vector Database (`VectorDatabase` trait)
Supports vector similarity search databases:
- Milvus (primary, local deployment)
- Weaviate
- Qdrant
- Qdrant
- Others

**Key Methods**:
- `connect()`, `disconnect()`
- `insert()`, `batch_insert()`
- `search()` - Similarity search with filters
- `get()`, `delete()`, `update_metadata()`
- `create_index()`, `delete_index()`

#### Graph Database (`GraphDatabase` trait)
Supports graph databases and knowledge graphs:
- Neo4j
- ArangoDB
- JanusGraph
- Amazon Neptune
- GraphQL endpoints

**Key Methods**:
- `connect()`, `disconnect()`
- `query()` - Execute Cypher, Gremlin, or GraphQL
- `create_node()`, `create_edge()`
- `get_node()`, `update_node()`, `delete_node()`
- `find_neighbors()`, `shortest_path()`

#### SQL Database (`SqlDatabase` trait)
Supports relational databases:
- MySQL
- PostgreSQL
- SQLite
- SQL Server
- Oracle

**Key Methods**:
- `connect()`, `disconnect()`
- `query()`, `execute()`
- `begin_transaction()` - Returns `SqlTransaction` trait
- Transaction support: `commit()`, `rollback()`

### 3. LLM Reasoning Interface

#### LLM Provider (`LlmProvider` trait)
Abstraction for connecting to major LLM providers:
- OpenAI (GPT-4, GPT-3.5, etc.)
- Anthropic (Claude)
- Google (Gemini, PaLM)
- Cohere
- HuggingFace
- Local models (Ollama, llama.cpp, etc.)

**Key Methods**:
- `initialize()` - Setup with API keys and configuration
- `complete()` - Generate completions
- `stream_complete()` - Streaming responses
- `available_models()` - List supported models

**Features**:
- Configurable temperature, max_tokens, and sampling parameters
- Token usage tracking
- Streaming support
- Multiple provider support

## 📊 Monitoring Dashboard

MeCP includes a beautiful real-time monitoring dashboard for debugging and analytics:

### Features
- **Real-time Metrics**: Live tracking of all MCP interface calls
- **Performance Analytics**: Response times, success rates, and throughput
- **Error Monitoring**: Quick access to errors with detailed messages
- **History Logs**: Complete audit trail stored in MySQL
- **Auto-refresh**: Updates every 5 seconds
- **REST API**: Programmatic access to metrics

### Quick Start

```bash
# Initialize database
./scripts/init-mysql-db.sh

# Start server
cargo run --release

# Open dashboard
# http://127.0.0.1:3000/dashboard
```

See [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md) for complete guide.

### Testing the Dashboard

Test the complete monitoring flow with included test scripts:

```bash
# Quick test with bash script (20 requests)
./scripts/test-dashboard-flow.sh

# Comprehensive test with Rust client (50 requests)
cargo run --example test_client

# Custom number of requests
cargo run --example test_client -- 100
```

See [TEST_SCRIPTS_README.md](TEST_SCRIPTS_README.md) for detailed testing guide.

## Building and Running

### Database Services

```bash
# Build CLI tool
cargo build --release

# Start all database services (auto-installs if needed)
./target/release/mecp-cli start

# Check service status
./target/release/mecp-cli status

# Stop services
./target/release/mecp-cli stop

# Reset databases (for testing)
./target/release/mecp-cli reset
```

See [CLI_USAGE.md](CLI_USAGE.md) for complete CLI reference.

### MCP Server

```bash
# Build the project
cargo build

# Run the server
cargo run

# Run tests
cargo test

# Build in release mode
cargo build --release
```

## Usage Example

```rust
use mecp::core::server::McpServer;
use mecp::resources::mock::MockResource;
use mecp::tools::mock::HelloWorldTool;
use mecp::prompts::mock::MockPrompt;

#[tokio::main]
async fn main() -> Result<()> {
    // Create server
    let server = McpServer::new();
    
    // Register components
    server.register_resource(Box::new(MockResource::new())).await;
    server.register_tool(Box::new(HelloWorldTool::new())).await;
    server.register_prompt(Box::new(MockPrompt::new())).await;
    
    // Run server
    server.run().await?;
    
    Ok(())
}
```

## Extending the Framework

### Adding a New Resource

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::resources::Resource;

pub struct MyCustomResource {
    // Your fields
}

#[async_trait]
impl Resource for MyCustomResource {
    async fn metadata(&self) -> Result<ResourceMetadata> {
        // Implementation
    }
    
    async fn read(&self) -> Result<ResourceContent> {
        // Implementation
    }
    
    async fn uri(&self) -> String {
        // Implementation
    }
}
```

### Adding a New Tool

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::tools::Tool;

pub struct MyCustomTool {
    // Your fields
}

#[async_trait]
impl Tool for MyCustomTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        // Implementation
    }
    
    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        // Implementation
    }
}
```

### Implementing a Database Connector

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::core::database::VectorDatabase;

pub struct MilvusConnector {
    // Your fields
}

#[async_trait]
impl VectorDatabase for MilvusConnector {
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()> {
        // Connect to Milvus
    }
    
    async fn search(&self, query_vector: Vec<f32>, top_k: usize, filter: Option<JsonValue>) -> Result<Vec<VectorSearchResult>> {
        // Implement similarity search
    }
    
    // ... implement other methods
}
```

### Implementing an LLM Provider

```rust
use async_trait::async_trait;
use anyhow::Result;
use crate::core::reasoning::LlmProvider;

pub struct OpenAIProvider {
    // Your fields
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    async fn initialize(&mut self, config: LlmConfig) -> Result<()> {
        // Initialize with API key
    }
    
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Call OpenAI API
    }
    
    // ... implement other methods
}
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
