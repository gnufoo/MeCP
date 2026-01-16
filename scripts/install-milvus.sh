#!/bin/bash
# MeCP - Milvus Installation Script
# This script installs Milvus via Docker for local development

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - Milvus Installation Script    ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo "❌ Please do not run this script as root"
   echo "   The script will use sudo when needed"
   exit 1
fi

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed"
    echo ""
    echo "Please install Docker first:"
    echo "  Ubuntu/Debian: sudo apt-get install docker.io"
    echo "  Or visit: https://docs.docker.com/get-docker/"
    exit 1
fi

# Check if user can run docker without sudo
if ! docker ps &> /dev/null; then
    echo "⚠️  Current user cannot run Docker commands"
    echo "   Adding user to docker group..."
    sudo usermod -aG docker $USER
    echo "   Please log out and back in for this to take effect"
    echo "   Then run this script again"
    exit 1
fi

echo "📦 Installing Milvus via Docker..."
echo ""

# Pull Milvus standalone image
echo "1/2 Pulling Milvus Docker image..."
docker pull milvusdb/milvus:latest

# Pull etcd (required for Milvus metadata)
echo "2/2 Pulling required dependencies..."
docker pull quay.io/coreos/etcd:latest 2>/dev/null || true
docker pull minio/minio:latest 2>/dev/null || true

echo ""
echo "✅ Milvus installation complete!"
echo ""
echo "Milvus Information:"
echo "  Type:       Vector Database (standalone mode)"
echo "  gRPC Port:  19530 (API connections)"
echo "  Web UI:     9091 (metrics/monitoring)"
echo "  Storage:    Local Docker volumes"
echo ""
echo "Next steps:"
echo "1. Enable Milvus in config.toml:"
echo "   [milvus]"
echo "   enabled = true"
echo ""
echo "2. Start Milvus:"
echo "   mecp-cli start milvus"
echo ""
echo "3. Or start all services:"
echo "   mecp-cli start"
echo ""
echo "Note: Milvus runs in a Docker container and stores data locally."
echo "      Use 'mecp-cli reset milvus' to clear all vector data."
