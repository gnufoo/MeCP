# MeCP Database Management Scripts

This directory contains helper scripts for managing MeCP database services.

## Installation Scripts

### Install MySQL
```bash
./scripts/install-mysql.sh
```
Installs MySQL Community Server on Ubuntu/Debian systems.

### Install Neo4j
```bash
./scripts/install-neo4j.sh
```
Installs Neo4j Community Edition on Ubuntu/Debian systems.

### Setup Milvus
```bash
./scripts/setup-milvus.sh
```
Shows instructions for setting up Milvus (cloud service).

## Reset Scripts

### Reset All Databases
```bash
./scripts/reset-all.sh [config.toml]
```
⚠️ **WARNING**: Deletes ALL data in MySQL, Neo4j, and Milvus.

### Reset MySQL Only
```bash
./scripts/reset-mysql.sh
```
⚠️ **WARNING**: Deletes ALL data in MySQL database.

### Reset Neo4j Only
```bash
./scripts/reset-neo4j.sh
```
⚠️ **WARNING**: Deletes ALL data in Neo4j database.

## Making Scripts Executable

Before running any script, make it executable:

```bash
chmod +x scripts/*.sh
```

## Recommended Usage

For most operations, use the `mecp-cli` tool instead of running scripts directly:

```bash
# Check status
mecp-cli status

# Install services
mecp-cli install

# Start services
mecp-cli start

# Stop services
mecp-cli stop

# Reset databases (with confirmation)
mecp-cli reset

# Check configuration
mecp-cli check
```

## Environment Variables

You can set these environment variables to customize behavior:

- `MECP_MYSQL_DB` - MySQL database name (default: mecp_db)
- `MILVUS_API_KEY` - Milvus API key

## Requirements

- Ubuntu/Debian-based Linux (including WSL)
- `sudo` access
- Internet connection for downloading packages

## Troubleshooting

### MySQL won't start
```bash
sudo systemctl status mysql
sudo journalctl -u mysql -n 50
```

### Neo4j won't start
```bash
sudo systemctl status neo4j
sudo journalctl -u neo4j -n 50
```

### Permission issues
Make sure scripts are executable:
```bash
chmod +x scripts/*.sh
```

## Safety Features

All reset scripts:
- Require explicit confirmation (type 'yes')
- Can be skipped with the `mecp-cli reset --yes` flag (use with caution)
- Create backups when possible (future enhancement)

## Next Steps

After installation, refer to the main README for:
- Configuration options
- API usage examples
- Integration guides
