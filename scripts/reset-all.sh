#!/bin/bash
# MeCP - Reset All Databases Script
# WARNING: This will delete ALL data in MySQL, Neo4j, and Milvus

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - Reset All Databases           ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Load configuration
CONFIG_FILE="${1:-config.toml}"

if [ ! -f "$CONFIG_FILE" ]; then
    echo "❌ Config file not found: $CONFIG_FILE"
    exit 1
fi

echo "⚠️  WARNING: This will DELETE ALL DATA in the databases!"
echo ""
echo "Services that will be reset:"
echo "  - MySQL database"
echo "  - Neo4j graph database"
echo "  - Milvus vector database (if configured)"
echo ""

# Confirmation
read -p "Are you sure you want to continue? (type 'yes' to confirm): " confirmation

if [ "$confirmation" != "yes" ]; then
    echo "❌ Reset cancelled"
    exit 0
fi

echo ""
echo "🔄 Resetting databases..."
echo ""

# Use the CLI tool for reset
if command -v mecp-cli &> /dev/null; then
    mecp-cli --config "$CONFIG_FILE" reset --yes
else
    # Fallback to manual reset
    echo "📦 mecp-cli not found, performing manual reset..."
    
    # Reset MySQL
    echo "1/3 Resetting MySQL..."
    if command -v mysql &> /dev/null; then
        sudo mysql -e "DROP DATABASE IF EXISTS mecp_db; CREATE DATABASE mecp_db; FLUSH PRIVILEGES;" 2>/dev/null || \
            echo "⚠️  Could not reset MySQL (may not be installed or running)"
    else
        echo "⚠️  MySQL not installed, skipping"
    fi
    
    # Reset Neo4j
    echo "2/3 Resetting Neo4j..."
    if systemctl is-active --quiet neo4j 2>/dev/null; then
        sudo systemctl stop neo4j
        sudo rm -rf /var/lib/neo4j/data/databases/* 2>/dev/null || \
            echo "⚠️  Could not remove Neo4j data"
        sudo systemctl start neo4j
        echo "✅ Neo4j reset complete"
    else
        echo "⚠️  Neo4j not running, skipping"
    fi
    
    # Milvus
    echo "3/3 Resetting Milvus..."
    if docker ps --filter "name=milvus-standalone" --format "{{.Names}}" | grep -q milvus-standalone 2>/dev/null; then
        docker stop milvus-standalone 2>/dev/null || true
        docker rm -f milvus-standalone 2>/dev/null || true
        echo "✅ Milvus reset complete"
    else
        echo "⚠️  Milvus not running, skipping"
    fi
fi

echo ""
echo "✅ Reset complete!"
echo ""
echo "Next steps:"
echo "1. Run: mecp-cli start"
echo "2. Verify: mecp-cli status"
