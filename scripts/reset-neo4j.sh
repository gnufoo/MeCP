#!/bin/bash
# MeCP - Reset Neo4j Database Script
# WARNING: This will delete ALL data in Neo4j

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - Reset Neo4j Database          ║"
echo "╚════════════════════════════════════════╝"
echo ""

echo "⚠️  WARNING: This will DELETE ALL DATA in the Neo4j database!"
echo ""

# Confirmation
read -p "Are you sure you want to continue? (type 'yes' to confirm): " confirmation

if [ "$confirmation" != "yes" ]; then
    echo "❌ Reset cancelled"
    exit 0
fi

echo ""
echo "🔄 Resetting Neo4j database..."

# Check if Neo4j is installed
if ! command -v neo4j &> /dev/null && [ ! -f /usr/bin/neo4j ]; then
    echo "❌ Neo4j is not installed"
    exit 1
fi

# Check if Neo4j is running
WAS_RUNNING=false
if systemctl is-active --quiet neo4j 2>/dev/null || pgrep -f "neo4j" >/dev/null 2>&1; then
    WAS_RUNNING=true
    echo "Stopping Neo4j..."
    sudo systemctl stop neo4j 2>/dev/null || sudo neo4j stop
    sleep 2
fi

# Remove data directories
echo "Removing Neo4j data..."
sudo rm -rf /var/lib/neo4j/data/databases/* 2>/dev/null || true
sudo rm -rf /var/lib/neo4j/data/transactions/* 2>/dev/null || true

# Restart if it was running
if [ "$WAS_RUNNING" = true ]; then
    echo "Starting Neo4j..."
    sudo systemctl start neo4j 2>/dev/null || sudo neo4j start
    
    # Wait for Neo4j to be ready
    echo "Waiting for Neo4j to be ready..."
    for i in {1..30}; do
        if systemctl is-active --quiet neo4j 2>/dev/null || pgrep -f "neo4j" >/dev/null 2>&1; then
            echo "✅ Neo4j is running"
            break
        fi
        sleep 1
    done
fi

echo "✅ Neo4j database reset complete"
echo ""
echo "Next steps:"
echo "1. Run: mecp-cli start neo4j"
echo "2. Visit: http://localhost:7474"
echo "3. Set new password for 'neo4j' user"
