# MeCP CLI Usage Guide

The `mecp-cli` tool provides a simple command-line interface for managing database services required by MeCP.

## Installation

Build the CLI tool:

```bash
cargo build --release
```

The binary will be available at `./target/release/mecp-cli`

## Quick Start

```bash
# Check current status
./target/release/mecp-cli status

# Install and start all services
./target/release/mecp-cli start

# Check health
./target/release/mecp-cli check

# Stop all services
./target/release/mecp-cli stop
```

## Commands

### Status

Show the status of all database services.

```bash
# Show all services
mecp-cli status

# Show specific service
mecp-cli status --service mysql
mecp-cli status --service neo4j
mecp-cli status --service milvus
```

**Output includes:**
- Installation status
- Running status
- Connection details (host, port)
- Database name

### Start

Start database services. Automatically checks and installs missing services.

```bash
# Start all enabled services
mecp-cli start

# Start specific service
mecp-cli start --service mysql
mecp-cli start --service neo4j
mecp-cli start --service milvus
```

**What it does:**
1. Checks if service is installed
2. Installs if missing (requires sudo)
3. Starts the service
4. Initializes database/configuration
5. Waits for service to be ready

### Stop / Shutdown

Stop database services.

```bash
# Stop all services
mecp-cli stop
# or
mecp-cli shutdown

# Stop specific service
mecp-cli stop --service mysql
mecp-cli shutdown --service neo4j
```

### Install

Install database services without starting them.

```bash
# Install all enabled services
mecp-cli install

# Install specific service
mecp-cli install --service mysql
mecp-cli install --service neo4j
mecp-cli install --service milvus
```

**Note:** Requires sudo access and internet connection.

### Reset

Reset databases to clean state. **WARNING: This deletes ALL data!**

```bash
# Reset all databases (with confirmation prompt)
mecp-cli reset

# Reset specific database
mecp-cli reset --service mysql

# Reset without confirmation (use with caution!)
mecp-cli reset --yes
```

**What it does:**
- MySQL: Drops and recreates database
- Neo4j: Stops service, deletes data files, restarts
- Milvus: Shows instructions for dashboard reset

### Check

Check configuration and service health.

```bash
mecp-cli check
```

**Output includes:**
- Configuration file location
- Detailed status of each service
- Connection URLs
- API key status (for Milvus)

## Options

### Config File

Specify a custom configuration file:

```bash
mecp-cli --config /path/to/config.toml status
mecp-cli -c custom-config.toml start
```

**Default:** `config.toml` in current directory

### Service Filter

Most commands accept `--service` or `-s` to target a specific service:

```bash
mecp-cli status --service mysql
mecp-cli start -s neo4j
mecp-cli reset --service milvus
```

**Available services:**
- `mysql` - MySQL database
- `neo4j` - Neo4j graph database
- `milvus` - Milvus vector database

### Skip Confirmation

For reset command, skip the confirmation prompt:

```bash
mecp-cli reset --yes
mecp-cli reset -y
```

## Examples

### First Time Setup

```bash
# 1. Check what's needed
mecp-cli status

# 2. Install and start everything
mecp-cli start

# 3. Verify installation
mecp-cli check
```

### Daily Usage

```bash
# Start services
mecp-cli start

# ... do your work ...

# Stop services when done
mecp-cli stop
```

### Debugging

```bash
# Reset all databases
mecp-cli reset

# Start fresh
mecp-cli start

# Check status
mecp-cli status
```

### Single Service Management

```bash
# Start only MySQL
mecp-cli start --service mysql

# Check MySQL status
mecp-cli status --service mysql

# Reset MySQL database
mecp-cli reset --service mysql

# Stop MySQL
mecp-cli stop --service mysql
```

### Configuration Testing

```bash
# Use test configuration
mecp-cli --config test-config.toml check

# Start with test config
mecp-cli --config test-config.toml start
```

## Exit Codes

- `0` - Success
- `1` - Error (check error message for details)

## Environment Variables

The following environment variables can be used:

- `MILVUS_API_KEY` - Milvus API key (overrides config.toml)
- `MECP_MYSQL_DB` - MySQL database name
- `RUST_LOG` - Logging level (e.g., `debug`, `info`, `warn`)

## Troubleshooting

### Permission Denied

Make sure the CLI is executable:

```bash
chmod +x ./target/release/mecp-cli
```

### Service Won't Start

Check system logs:

```bash
# MySQL
sudo journalctl -u mysql -n 50

# Neo4j
sudo journalctl -u neo4j -n 50
```

Try manual start:

```bash
sudo systemctl start mysql
sudo systemctl start neo4j
```

### Config Not Found

Ensure `config.toml` exists in the current directory or specify path:

```bash
mecp-cli --config /path/to/config.toml status
```

### Installation Fails

Ensure you have:
- Internet connection
- sudo access
- Ubuntu/Debian-based system

Check system logs for details.

### Can't Connect to Service

Verify service is running:

```bash
mecp-cli status

# Check system service
sudo systemctl status mysql
sudo systemctl status neo4j
```

Check ports:

```bash
sudo netstat -tlnp | grep 3306  # MySQL
sudo netstat -tlnp | grep 7687  # Neo4j
```

## Advanced Usage

### Scripting

Use in bash scripts:

```bash
#!/bin/bash
set -e

echo "Starting MeCP services..."
mecp-cli start

echo "Running application..."
cargo run

echo "Stopping services..."
mecp-cli stop
```

### CI/CD Integration

```yaml
# Example GitHub Actions workflow
- name: Setup databases
  run: |
    cargo build --release --bin mecp-cli
    ./target/release/mecp-cli install
    ./target/release/mecp-cli start

- name: Run tests
  run: cargo test

- name: Cleanup
  run: ./target/release/mecp-cli stop
```

### Docker Integration

```dockerfile
FROM rust:1.75

# Copy project
WORKDIR /app
COPY . .

# Build CLI
RUN cargo build --release --bin mecp-cli

# Install services
RUN ./target/release/mecp-cli install

# Start services on container start
CMD ["./target/release/mecp-cli", "start"]
```

## Best Practices

1. **Check status first**: Always run `mecp-cli status` before operations
2. **Use specific services**: Target specific services in production
3. **Backup before reset**: Always backup data before running reset
4. **Environment variables**: Use env vars for sensitive data
5. **Test configurations**: Use separate config files for dev/test/prod
6. **Monitor logs**: Check system logs if services fail to start

## Getting Help

```bash
# General help
mecp-cli --help

# Command-specific help
mecp-cli start --help
mecp-cli reset --help
```

## See Also

- [Database Setup Guide](DATABASE_SETUP.md) - Detailed setup instructions
- [Configuration Reference](config.toml) - All configuration options
- [Troubleshooting Guide](DATABASE_SETUP.md#troubleshooting) - Common issues
- [Scripts README](scripts/README.md) - Alternative shell scripts
