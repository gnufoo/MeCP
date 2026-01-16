# MeCP CLI Quick Reference

## Installation

```bash
cargo build --release
```

## Common Commands

```bash
# Check status of all services
mecp-cli status

# Start all services (auto-install if needed)
mecp-cli start

# Stop all services
mecp-cli stop

# Reset all databases (with confirmation)
mecp-cli reset

# Check configuration and health
mecp-cli check
```

## Service-Specific

```bash
# MySQL
mecp-cli status --service mysql
mecp-cli start --service mysql
mecp-cli reset --service mysql

# Neo4j
mecp-cli status --service neo4j
mecp-cli start --service neo4j
mecp-cli reset --service neo4j

# Milvus
mecp-cli status --service milvus
mecp-cli start --service milvus
```

## Configuration

**File**: `config.toml`

```toml
[mysql]
enabled = true
host = "localhost"
port = 3306
database = "mecp_db"
username = "mecp_user"
password = "mecp_password"

[neo4j]
enabled = true
bolt_url = "bolt://localhost:7687"
http_url = "http://localhost:7474"
username = "neo4j"
password = "mecp_neo4j_password"

[milvus]
enabled = false
# api_key = "your-key"  # or use MILVUS_API_KEY env var
environment = "us-west1-gcp"
index_name = "mecp-vectors"
```

## Environment Variables

```bash
export MILVUS_API_KEY="your-milvus-api-key"
export MECP_MYSQL_DB="mecp_db"
```

## Access Services

**MySQL**:
```bash
mysql -u mecp_user -p mecp_db
```

**Neo4j Browser**:
- URL: http://localhost:7474
- Username: neo4j
- Password: (from config.toml)

**Milvus**:
- Dashboard: https://app.milvus.io/

## Troubleshooting

```bash
# View service logs
sudo journalctl -u mysql -n 50
sudo journalctl -u neo4j -n 50

# Check ports
sudo netstat -tlnp | grep 3306  # MySQL
sudo netstat -tlnp | grep 7687  # Neo4j

# Manual service control
sudo systemctl start mysql
sudo systemctl start neo4j
sudo systemctl status mysql
sudo systemctl status neo4j
```

## Scripts (Alternative)

```bash
# Make executable
chmod +x scripts/*.sh

# Install
./scripts/install-mysql.sh
./scripts/install-neo4j.sh
./scripts/setup-milvus.sh

# Reset
./scripts/reset-all.sh
./scripts/reset-mysql.sh
./scripts/reset-neo4j.sh
```

## Help

```bash
mecp-cli --help
mecp-cli <command> --help
```

## Documentation

- **Full Setup**: [DATABASE_SETUP.md](DATABASE_SETUP.md)
- **CLI Guide**: [CLI_USAGE.md](CLI_USAGE.md)
- **Installation**: [INSTALLATION.md](INSTALLATION.md)
- **Summary**: [MECP_CLI_SUMMARY.md](MECP_CLI_SUMMARY.md)
