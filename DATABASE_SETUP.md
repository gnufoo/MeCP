# MeCP Database Setup Guide

This guide will help you install and configure MySQL, Neo4j, and Milvus for use with MeCP.

## Table of Contents

- [Quick Start](#quick-start)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [Method 1: Using MeCP CLI (Recommended)](#method-1-using-mecp-cli-recommended)
  - [Method 2: Using Shell Scripts](#method-2-using-shell-scripts)
  - [Method 3: Manual Installation](#method-3-manual-installation)
- [Configuration](#configuration)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)

## Quick Start

The fastest way to get started:

```bash
# Build the CLI tool
cargo build --release

# Check status (will show what's missing)
./target/release/mecp-cli status

# Install and start all services
./target/release/mecp-cli start

# Verify everything is running
./target/release/mecp-cli check
```

## Prerequisites

- **Operating System**: Ubuntu/Debian Linux (including WSL2)
- **Rust**: 1.70 or later
- **sudo access**: Required for installing system packages
- **Internet connection**: For downloading packages

## Installation

### Method 1: Using MeCP CLI (Recommended)

The MeCP CLI automatically checks, installs, and manages all database services.

```bash
# Build the CLI
cargo build --release

# Install all services (will check if already installed)
./target/release/mecp-cli install

# Start all services
./target/release/mecp-cli start

# Check status
./target/release/mecp-cli status
```

Install specific services:

```bash
# Install only MySQL
./target/release/mecp-cli install --service mysql

# Install only Neo4j
./target/release/mecp-cli install --service neo4j

# Setup Milvus (cloud service)
./target/release/mecp-cli install --service milvus
```

### Method 2: Using Shell Scripts

Alternatively, use the provided shell scripts:

```bash
# Make scripts executable (if not already)
chmod +x scripts/*.sh

# Install MySQL
./scripts/install-mysql.sh

# Install Neo4j
./scripts/install-neo4j.sh

# Setup Milvus (shows instructions)
./scripts/setup-milvus.sh
```

### Method 3: Manual Installation

#### MySQL Community Server

```bash
# Update package list
sudo apt-get update

# Install MySQL
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y mysql-server

# Start MySQL
sudo systemctl start mysql
sudo systemctl enable mysql

# Initialize database
sudo mysql -e "
  CREATE DATABASE IF NOT EXISTS mecp_db;
  CREATE USER IF NOT EXISTS 'mecp_user'@'localhost' IDENTIFIED BY 'mecp_password';
  GRANT ALL PRIVILEGES ON mecp_db.* TO 'mecp_user'@'localhost';
  FLUSH PRIVILEGES;
"
```

#### Neo4j Community Edition

```bash
# Install dependencies
sudo apt-get update
sudo apt-get install -y wget gnupg software-properties-common openjdk-17-jre

# Add Neo4j repository
sudo mkdir -p /etc/apt/keyrings
wget -O - https://debian.neo4j.com/neotechnology.gpg.key | \
  sudo gpg --dearmor -o /etc/apt/keyrings/neo4j.gpg

echo "deb [signed-by=/etc/apt/keyrings/neo4j.gpg] https://debian.neo4j.com stable latest" | \
  sudo tee /etc/apt/sources.list.d/neo4j.list

# Install Neo4j
sudo apt-get update
sudo apt-get install -y neo4j

# Start Neo4j
sudo systemctl start neo4j
sudo systemctl enable neo4j

# Set initial password
sudo neo4j-admin dbms set-initial-password mecp_neo4j_password
```

#### Milvus (Cloud Service)

1. **Sign up**: Visit [https://www.milvus.io/](https://www.milvus.io/)
2. **Get API key**: Login to [https://app.milvus.io/](https://app.milvus.io/) → API Keys
3. **Create index**: 
   - Name: `mecp-vectors`
   - Dimension: `384` (adjust for your embedding model)
   - Metric: `cosine`

## Configuration

Edit `config.toml` in the project root:

```toml
[mysql]
enabled = true
host = "localhost"
port = 3306
database = "mecp_db"
username = "mecp_user"
password = "mecp_password"
pool_min = 5
pool_max = 20
connect_timeout = 30

[neo4j]
enabled = true
host = "localhost"
port = 7687
bolt_url = "bolt://localhost:7687"
http_url = "http://localhost:7474"
username = "neo4j"
password = "mecp_neo4j_password"
database = "neo4j"
encrypted = false

[milvus]
enabled = false  # Set to true when configured
# api_key = "your-api-key"  # Or use MILVUS_API_KEY env var
environment = "us-west1-gcp"
index_name = "mecp-vectors"
dimension = 384
metric = "cosine"
```

### Environment Variables

You can also use environment variables (they override config.toml):

```bash
export MILVUS_API_KEY="your-milvus-api-key"
export MECP_MYSQL_DB="mecp_db"
```

## Usage

### MeCP CLI Commands

```bash
# Check status of all services
mecp-cli status

# Check specific service
mecp-cli status --service mysql

# Start all services
mecp-cli start

# Start specific service
mecp-cli start --service neo4j

# Stop all services
mecp-cli stop
# or
mecp-cli shutdown

# Stop specific service
mecp-cli shutdown --service mysql

# Reset all databases (WARNING: deletes all data)
mecp-cli reset

# Reset specific database
mecp-cli reset --service mysql

# Reset without confirmation (use with caution!)
mecp-cli reset --yes

# Check configuration and health
mecp-cli check

# Show help
mecp-cli --help
```

### Shell Script Usage

```bash
# Reset all databases
./scripts/reset-all.sh

# Reset specific databases
./scripts/reset-mysql.sh
./scripts/reset-neo4j.sh
```

### Example Workflow

```bash
# 1. Build the project
cargo build --release

# 2. Check current status
./target/release/mecp-cli status

# 3. Install missing services
./target/release/mecp-cli install

# 4. Start all services
./target/release/mecp-cli start

# 5. Verify everything is running
./target/release/mecp-cli check

# 6. Run your application
cargo run

# 7. When done, stop services
./target/release/mecp-cli stop

# 8. For debugging, reset databases
./target/release/mecp-cli reset
```

## Service Details

### MySQL

- **Type**: SQL Database
- **Default Port**: 3306
- **Service Name**: `mysql`
- **Data Location**: `/var/lib/mysql`
- **Web Interface**: None (use `mysql` client or tools like MySQL Workbench)

**Connect to MySQL**:
```bash
mysql -u mecp_user -p mecp_db
# Password: mecp_password
```

### Neo4j

- **Type**: Graph Database
- **Default Ports**: 
  - Bolt: 7687
  - HTTP: 7474
  - HTTPS: 7473
- **Service Name**: `neo4j`
- **Data Location**: `/var/lib/neo4j/data`
- **Web Interface**: [http://localhost:7474](http://localhost:7474)

**Access Neo4j Browser**:
1. Open [http://localhost:7474](http://localhost:7474)
2. Username: `neo4j`
3. Password: `mecp_neo4j_password`

### Milvus

- **Type**: Vector Database (Cloud)
- **Default Port**: 443 (HTTPS)
- **Service Name**: `milvus`
- **Data Location**: Cloud (no local storage)
- **Web Interface**: [https://app.milvus.io/](https://app.milvus.io/)

**No local installation required** - it's a cloud service.

## Troubleshooting

### MySQL Issues

**MySQL won't start**:
```bash
# Check status
sudo systemctl status mysql

# View logs
sudo journalctl -u mysql -n 50

# Try restarting
sudo systemctl restart mysql
```

**Connection refused**:
```bash
# Check if MySQL is running
sudo systemctl is-active mysql

# Check port
sudo netstat -tlnp | grep 3306

# Try connecting
mysql -u mecp_user -p
```

### Neo4j Issues

**Neo4j won't start**:
```bash
# Check status
sudo systemctl status neo4j

# View logs
sudo journalctl -u neo4j -n 50
# or
sudo cat /var/log/neo4j/neo4j.log

# Try restarting
sudo systemctl restart neo4j
```

**Can't access web interface**:
```bash
# Check if Neo4j is running
sudo systemctl is-active neo4j

# Check ports
sudo netstat -tlnp | grep 747

# Check configuration
sudo cat /etc/neo4j/neo4j.conf | grep server.default
```

**Password issues**:
```bash
# Reset password
sudo neo4j-admin dbms set-initial-password newpassword
```

### Milvus Issues

**API key not working**:
- Verify key at [https://app.milvus.io/](https://app.milvus.io/)
- Check environment variable: `echo $MILVUS_API_KEY`
- Ensure no extra spaces in config.toml

**Index not found**:
- Create index via dashboard: [https://app.milvus.io/](https://app.milvus.io/)
- Verify index name matches config.toml
- Check environment setting

### General Issues

**Permission denied**:
```bash
# Make scripts executable
chmod +x scripts/*.sh

# Check sudo access
sudo -v
```

**Port conflicts**:
```bash
# Check what's using a port
sudo netstat -tlnp | grep <port>
sudo lsof -i :<port>

# Kill process using port (if needed)
sudo kill <pid>
```

**WSL-specific issues**:
```bash
# Restart WSL services
sudo service mysql restart
sudo service neo4j restart

# Check WSL version
wsl --version
```

## Best Practices

1. **Backup before reset**: Always backup important data before running reset scripts
2. **Use environment variables**: For sensitive data like API keys
3. **Check status first**: Always check `mecp-cli status` before operations
4. **Test in development**: Test reset procedures in development environment first
5. **Monitor logs**: Check logs if services fail to start
6. **Keep passwords secure**: Don't commit passwords to version control

## Security Considerations

1. **Change default passwords**: Update passwords in `config.toml`
2. **Use strong passwords**: Especially for production environments
3. **Restrict access**: Configure firewall rules for database ports
4. **Environment variables**: Use env vars for sensitive data
5. **Regular updates**: Keep database software updated

## Next Steps

After setup is complete:

1. Run integration tests: `cargo test`
2. Try the examples: `cargo run --example database_usage`
3. Read the API documentation: `DATABASE_API.md`
4. Explore the MCP server: `cargo run`

## Additional Resources

- [MySQL Documentation](https://dev.mysql.com/doc/)
- [Neo4j Documentation](https://neo4j.com/docs/)
- [Milvus Documentation](https://docs.milvus.io/)
- [MeCP Architecture](ARCHITECTURE.md)
- [MeCP Quick Start](QUICKSTART.md)
