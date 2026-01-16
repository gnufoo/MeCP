# MeCP - Model Context Protocol Server

<div align="center">

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)
![Status](https://img.shields.io/badge/status-production--ready-brightgreen?style=for-the-badge)

**Production-ready Model Context Protocol server in Rust**

Self-hosted AI context management with multi-database support, Web3 authentication, and real-time monitoring

[Features](#-features) • [Quick Start](#-quick-start) • [Documentation](#-documentation) • [Architecture](#-architecture) • [Contributing](#-contributing)

</div>

---

## 🚀 Features

### Core Capabilities
- **🦀 High-Performance Rust** - Blazing fast, memory-safe implementation
- **📋 JSON-RPC 2.0 API** - Standard MCP protocol compliance
- **🔌 Modular Architecture** - Extensible resource, tool, and prompt system
- **📡 HTTP Server** - RESTful API with CORS support

### Database Integration
- **🗄️ MySQL** - Relational data storage with connection pooling
- **🕸️ Neo4j** - Graph database for complex relationships
- **🔢 Milvus** - High-performance vector database for embeddings
- **🔄 Unified Abstraction Layer** - Switch databases seamlessly

### Monitoring & Security
- **📊 Real-time Dashboard** - Beautiful web UI for metrics and logs
- **💾 Persistent Metrics** - MySQL-backed analytics and history
- **🔐 Web3 Authentication** - Gasless EVM wallet signature auth
- **🔒 JWT Sessions** - Secure, stateless session management

### Developer Experience
- **⚡ CLI Management** - One-command database setup and control
- **🧪 Comprehensive Testing** - Unit and integration test suites
- **📚 Complete Documentation** - Guides for every component
- **🐳 Production Ready** - Battle-tested with enterprise features

---

## 🎯 Quick Start

### Prerequisites
- Rust 1.70+ ([Install](https://rustup.rs/))
- MySQL 8.0+ ([Install](https://dev.mysql.com/downloads/))
- (Optional) Neo4j, Milvus for additional features

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/mecp.git
cd mecp

# Build the project
cargo build --release

# Initialize databases
./scripts/init-mysql-db.sh

# Start the server
cargo run --release
```

### Access Dashboard

```
http://127.0.0.1:3000/dashboard
```

### Make Your First API Call

```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "resources/list"
  }'
```

---

## 📚 Documentation

### Getting Started
- **[Installation Guide](INSTALLATION.md)** - Detailed setup instructions
- **[Quick Start](QUICKSTART.md)** - API usage and examples
- **[CLI Usage](CLI_USAGE.md)** - Command-line interface reference

### Features
- **[Dashboard Guide](DASHBOARD.md)** - Monitoring and metrics
- **[Web3 Authentication](WEB3_AUTH_GUIDE.md)** - Secure wallet-based auth
- **[Database Setup](DATABASE_SETUP.md)** - Multi-database configuration

### Advanced
- **[Architecture](ARCHITECTURE.md)** - System design and components
- **[API Documentation](API_DOCUMENTATION.md)** - Complete API reference
- **[Testing Guide](TESTING.md)** - Running and writing tests

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                       MeCP Server                           │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │
│  │  Resources  │  │    Tools    │  │   Prompts   │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │           HTTP Server (Axum + Tower)                │   │
│  │  ┌──────────┐  ┌──────────┐  ┌────────────────┐   │   │
│  │  │  /mcp    │  │  /api/*  │  │  /dashboard    │   │   │
│  │  └──────────┘  └──────────┘  └────────────────┘   │   │
│  └────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌────────────────────────────────────────────────────┐   │
│  │         Database Abstraction Layer                  │   │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │   │
│  │  │  MySQL   │  │  Neo4j   │  │     Milvus       │ │   │
│  │  └──────────┘  └──────────┘  └──────────────────┘ │   │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

- **Core Server** - MCP protocol implementation with extensible handlers
- **HTTP Layer** - Axum-based web server with middleware support
- **Database Layer** - Trait-based abstraction for SQL, Graph, and Vector DBs
- **Metrics System** - Real-time collection with MySQL persistence
- **Auth Module** - EVM signature verification and JWT token management

---

## 🛠️ Technology Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2021 Edition |
| Web Framework | Axum 0.7 |
| Database | MySQL 8.0, Neo4j, Milvus |
| Authentication | ethers-rs, jsonwebtoken |
| Serialization | serde, serde_json |
| Async Runtime | tokio |
| Testing | tokio-test, reqwest |

---

## 📊 Dashboard

The integrated monitoring dashboard provides:

- **📈 Real-time Metrics** - API call rates, success rates, response times
- **📝 Request History** - Detailed logs with full request/response data
- **🐛 Error Tracking** - Dedicated error monitoring with stack traces
- **📊 Analytics** - Per-endpoint statistics and trends
- **🔄 Live Updates** - Auto-refresh every 5 seconds
- **🔐 Secure Access** - Optional Web3 wallet authentication

![Dashboard Screenshot](https://via.placeholder.com/800x400?text=Dashboard+Screenshot)

---

## 🔐 Web3 Authentication

Secure your dashboard with cryptographic wallet signatures:

```bash
# Quick setup
./scripts/setup-auth-example.sh

# Or manual configuration
[auth]
enabled = true
allowed_address = "0xYourWalletAddress"
jwt_secret = "your-secret-key"
session_duration = 86400
```

Features:
- ✅ **Gasless Authentication** - No blockchain transactions
- ✅ **EVM Compatible** - Works with MetaMask, WalletConnect, etc.
- ✅ **24-Hour Sessions** - JWT-based session management
- ✅ **Production Ready** - Used in live deployments

[Read the Web3 Auth Guide →](WEB3_AUTH_GUIDE.md)

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run with coverage
cargo tarpaulin --out Html

# Test dashboard flow
./scripts/test-dashboard-flow.sh
```

---

## 🚀 Deployment

### Production Checklist

- [ ] Configure strong JWT secret (`openssl rand -hex 64`)
- [ ] Set up HTTPS with reverse proxy (nginx/caddy)
- [ ] Enable rate limiting on auth endpoints
- [ ] Configure database connection pooling
- [ ] Set up monitoring and alerting
- [ ] Backup database regularly
- [ ] Review security settings in `config.toml`

### Docker Deployment (Coming Soon)

```bash
docker-compose up -d
```

---

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone and setup
git clone https://github.com/yourusername/mecp.git
cd mecp
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run linter
cargo clippy
```

### Contribution Areas

- 🐛 Bug fixes and improvements
- ✨ New database adapters (PostgreSQL, MongoDB, etc.)
- 📚 Documentation enhancements
- 🧪 Additional test coverage
- 🎨 Dashboard UI improvements
- 🔌 New MCP tools and resources

---

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) specification
- Rust community for excellent libraries
- Contributors and supporters

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/mecp/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/mecp/discussions)
- **Documentation**: [Full Documentation](INSTALLATION.md)

---

<div align="center">

**Built with ❤️ in Rust**

⭐ Star us on GitHub | 🐛 Report Issues | 🤝 Contribute

</div>
