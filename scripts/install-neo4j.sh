#!/bin/bash
# MeCP - Neo4j Installation Script
# This script installs Neo4j Community Edition on WSL/Ubuntu

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - Neo4j Installation Script     ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "❌ Please do not run this script as root"
   echo "   The script will use sudo when needed"
   exit 1
fi

# Check OS
if ! grep -qi "ubuntu\|debian" /etc/os-release; then
    echo "❌ This script is designed for Ubuntu/Debian systems"
    exit 1
fi

# Check if Neo4j is already installed
if command -v neo4j &> /dev/null || [ -f /usr/bin/neo4j ]; then
    echo "✅ Neo4j is already installed"
    neo4j version || echo "Neo4j found but version check failed"
    exit 0
fi

echo "📦 Installing Neo4j Community Edition..."
echo ""

# Install dependencies
echo "1/6 Installing dependencies..."
sudo apt-get update -y
sudo apt-get install -y wget gnupg software-properties-common apt-transport-https

# Install Java (required for Neo4j)
echo "2/6 Installing Java..."
if ! command -v java &> /dev/null; then
    sudo apt-get install -y openjdk-17-jre
else
    echo "✅ Java already installed"
fi

# Add Neo4j repository key
echo "3/6 Adding Neo4j repository..."

# Create keyrings directory
sudo mkdir -p /etc/apt/keyrings

# Download and add GPG key
wget -O - https://debian.neo4j.com/neotechnology.gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/neo4j.gpg

# Add Neo4j repository
echo "deb [signed-by=/etc/apt/keyrings/neo4j.gpg] https://debian.neo4j.com stable latest" | \
    sudo tee /etc/apt/sources.list.d/neo4j.list

# Update package list
echo "4/6 Updating package list..."
sudo apt-get update -y

# Install Neo4j
echo "5/6 Installing Neo4j..."
sudo apt-get install -y neo4j

# Configure Neo4j (allow remote connections)
echo "6/6 Configuring Neo4j..."
if [ -f /etc/neo4j/neo4j.conf ]; then
    # Uncomment the line to allow connections from any interface
    sudo sed -i 's/#server.default_listen_address=0.0.0.0/server.default_listen_address=0.0.0.0/' /etc/neo4j/neo4j.conf || true
fi

# Start Neo4j service
echo "Starting Neo4j service..."
sudo systemctl enable neo4j
sudo systemctl start neo4j

echo ""
echo "✅ Neo4j installation complete!"
echo ""

# Wait a bit for Neo4j to start
sleep 3

# Check status
if systemctl is-active --quiet neo4j; then
    echo "✅ Neo4j service is running"
else
    echo "⚠️  Neo4j service is not running"
    echo "   Try: sudo systemctl start neo4j"
fi

echo ""
echo "Neo4j Information:"
echo "  Bolt URL:  bolt://localhost:7687"
echo "  HTTP URL:  http://localhost:7474"
echo "  Default username: neo4j"
echo "  Default password: neo4j (must change on first login)"
echo ""
echo "Next steps:"
echo "1. Run: mecp-cli start neo4j"
echo "2. Or visit: http://localhost:7474"
