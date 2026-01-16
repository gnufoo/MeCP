# MeCP Installation Guide

Complete guide for installing and setting up MeCP with all required database services.

## System Requirements

- **OS**: Ubuntu 20.04+ or Debian 11+ (including WSL2)
- **Rust**: 1.70 or later
- **RAM**: 4GB minimum, 8GB recommended
- **Disk Space**: 2GB for databases and dependencies
- **Network**: Internet connection for package installation
- **Privileges**: sudo access required

## Quick Installation

```bash
# 1. Clone the repository (if needed)
git clone <repository-url>
cd MeCP

# 2. Build the project
cargo build --release

# 3. Install and start all services
./target/release/mecp-cli start

# 4. Verify installation
./target/release/mecp-cli check
```

That's it! MeCP is now ready to use.

## Step-by-Step Installation

### 1. Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Verify installation:

```bash
rustc --version
cargo --version
```

### 2. Clone and Build MeCP

```bash
# Clone the repository
git clone <repository-url>
cd MeCP

# Build the project
cargo build --release

# Build time: ~5-10 minutes on first build
```

### 3. Configure Services

Edit `config.toml` to customize settings:

```bash
# Copy example config (if needed)
cp config.toml.example config.toml

# Edit configuration
nano config.toml
```

Key settings to review:

```toml
[mysql]
enabled = true          # Enable MySQL
database = "mecp_db"    # Database name
username = "mecp_user"  # Username
password = "change_me"  # Change this!

[neo4j]
enabled = true          # Enable Neo4j
password = "change_me"  # Change this!

[milvus]
enabled = false         # Enable only if using Milvus
# Milvus runs locally via Docker
```

### 4. Install Database Services

Using MeCP CLI (recommended):

```bash
# Install all enabled services
./target/release/mecp-cli install

# Or install individually
./target/release/mecp-cli install --service mysql
./target/release/mecp-cli install --service neo4j
```

Using shell scripts:

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Install MySQL
./scripts/install-mysql.sh

# Install Neo4j
./scripts/install-neo4j.sh

# Install Milvus
./scripts/install-milvus.sh
```

### 5. Start Services

```bash
# Start all services
./target/release/mecp-cli start

# Or start individually
./target/release/mecp-cli start --service mysql
./target/release/mecp-cli start --service neo4j
```

### 6. Verify Installation

```bash
# Check status
./target/release/mecp-cli status

# Expected output:
# ✅ MySQL: Installed and Running
# ✅ Neo4j: Installed and Running
# ⚠️  Milvus: Not running (if not enabled)

# Detailed health check
./target/release/mecp-cli check
```

### 7. Run Tests

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --test integration_test
```

### 8. Optional: Add CLI to PATH

For convenience, add the CLI to your PATH:

```bash
# Option 1: Create symlink
sudo ln -s $(pwd)/target/release/mecp-cli /usr/local/bin/mecp-cli

# Option 2: Add to PATH in ~/.bashrc
echo "export PATH=\"$(pwd)/target/release:\$PATH\"" >> ~/.bashrc
source ~/.bashrc

# Now you can use it from anywhere
mecp-cli status
```

## Installation Options

### Option 1: Full Installation (Recommended)

Install all services for complete functionality:

```bash
./target/release/mecp-cli start
```

**Installs:**
- MySQL (SQL database)
- Neo4j (Graph database)
- Milvus (Vector database)

### Option 2: Minimal Installation

Install only required services:

```bash
# Edit config.toml
[mysql]
enabled = true

[neo4j]
enabled = false  # Disable Neo4j

[milvus]
enabled = false  # Disable Milvus

# Install
./target/release/mecp-cli start
```

### Option 3: Custom Installation

Pick and choose services:

```bash
# Install only MySQL
./target/release/mecp-cli install --service mysql
./target/release/mecp-cli start --service mysql

# Add Neo4j later
./target/release/mecp-cli install --service neo4j
./target/release/mecp-cli start --service neo4j
```

## Milvus Setup

Milvus is a local vector database that runs via Docker:

### 1. Install Docker

If Docker is not already installed:

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io
sudo systemctl start docker
sudo systemctl enable docker

# Add your user to docker group
sudo usermod -aG docker $USER
# Log out and back in for this to take effect
```

### 2. Install Milvus

Using MeCP CLI (recommended):

```bash
./target/release/mecp-cli install --service milvus
```

Or using the install script:

```bash
./scripts/install-milvus.sh
```

### 3. Configure

Edit `config.toml`:

```toml
[milvus]
enabled = true
host = "localhost"
port = 19530
collection_name = "mecp_vectors"
dimension = 384  # Adjust for your embedding model
metric = "L2"    # L2, IP, or COSINE
index_type = "IVF_FLAT"
```

### 4. Start Milvus

```bash
./target/release/mecp-cli start --service milvus
```

### 5. Verify

```bash
./target/release/mecp-cli check

# Access Milvus web UI (optional)
# Visit: http://localhost:9091
```

## Post-Installation

### Verify Services

```bash
# Check all services
./target/release/mecp-cli status

# Test MySQL connection
mysql -u mecp_user -p mecp_db
# Password: from config.toml

# Test Neo4j connection
# Open browser: http://localhost:7474
# Username: neo4j
# Password: from config.toml
```

### Run Examples

```bash
# Database usage examples
cargo run --example database_usage

# LLM usage examples
cargo run --example llm_usage
```

### Enable Autostart (Optional)

To automatically start services on boot:

```bash
sudo systemctl enable mysql
sudo systemctl enable neo4j
```

## Upgrading

### Update MeCP

```bash
# Pull latest changes
git pull

# Rebuild
cargo build --release

# Restart services
./target/release/mecp-cli stop
./target/release/mecp-cli start
```

### Update Databases

```bash
# Update MySQL
sudo apt-get update
sudo apt-get upgrade mysql-server

# Update Neo4j
sudo apt-get update
sudo apt-get upgrade neo4j
```

## Uninstallation

### Remove MeCP

```bash
# Stop all services
./target/release/mecp-cli stop

# Remove build artifacts
cargo clean

# Remove CLI from PATH (if installed)
sudo rm /usr/local/bin/mecp-cli
```

### Remove Databases

**MySQL:**

```bash
sudo systemctl stop mysql
sudo apt-get remove --purge mysql-server mysql-client mysql-common
sudo rm -rf /var/lib/mysql
```

**Neo4j:**

```bash
sudo systemctl stop neo4j
sudo apt-get remove --purge neo4j
sudo rm -rf /var/lib/neo4j
```

## Troubleshooting

### Build Fails

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Installation Fails

```bash
# Check sudo access
sudo -v

# Check internet connection
ping -c 3 google.com

# Check system logs
sudo journalctl -xe
```

### Service Won't Start

```bash
# Check what's wrong
mecp-cli status

# View service logs
sudo journalctl -u mysql -n 50
sudo journalctl -u neo4j -n 50

# Try manual start
sudo systemctl start mysql
sudo systemctl start neo4j
```

### Port Conflicts

```bash
# Check what's using the port
sudo netstat -tlnp | grep 3306  # MySQL
sudo netstat -tlnp | grep 7687  # Neo4j

# Kill conflicting process
sudo kill <pid>
```

## WSL-Specific Notes

### Enable systemd in WSL2

Edit `/etc/wsl.conf`:

```ini
[boot]
systemd=true
```

Restart WSL:

```powershell
# In PowerShell
wsl --shutdown
wsl
```

### WSL Performance Tips

- Store the project in WSL filesystem (not /mnt/c/)
- Use WSL2 (not WSL1)
- Allocate sufficient memory in `.wslconfig`

## Getting Help

- **Documentation**: See [DATABASE_SETUP.md](DATABASE_SETUP.md)
- **CLI Help**: Run `mecp-cli --help`
- **Issues**: Check [Troubleshooting](#troubleshooting) section
- **Community**: (Add your support channels here)

## Next Steps

After installation:

1. Read [CLI Usage Guide](CLI_USAGE.md)
2. Explore [Examples](examples/)
3. Review [Architecture](ARCHITECTURE.md)
4. Try the [Quick Start](QUICKSTART.md)
5. Read [Testing Guide](TESTING.md)

## Summary

You should now have:

- ✅ MeCP built and ready
- ✅ MySQL installed and running
- ✅ Neo4j installed and running
- ✅ Milvus installed and running (if enabled)
- ✅ CLI tool accessible
- ✅ All tests passing

**Quick reference:**

```bash
mecp-cli status    # Check services
mecp-cli start     # Start services
mecp-cli stop      # Stop services
mecp-cli reset     # Reset databases (dev only!)
mecp-cli check     # Health check
```

Happy coding! 🚀
