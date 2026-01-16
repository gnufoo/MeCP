#!/bin/bash
# MeCP - MySQL Installation Script
# This script installs MySQL Community Server on WSL/Ubuntu

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - MySQL Installation Script     ║"
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

# Check if MySQL is already installed
if command -v mysql &> /dev/null; then
    echo "✅ MySQL is already installed"
    mysql --version
    exit 0
fi

echo "📦 Installing MySQL Community Server..."
echo ""

# Update package list
echo "1/3 Updating package list..."
sudo apt-get update -y

# Install MySQL
echo "2/3 Installing MySQL server..."
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y mysql-server

# Start MySQL service
echo "3/3 Starting MySQL service..."
sudo systemctl start mysql
sudo systemctl enable mysql

echo ""
echo "✅ MySQL installation complete!"
echo ""

# Check status
if systemctl is-active --quiet mysql; then
    echo "✅ MySQL service is running"
else
    echo "⚠️  MySQL service is not running"
    echo "   Try: sudo systemctl start mysql"
fi

echo ""
echo "Next steps:"
echo "1. Run: mecp-cli start mysql"
echo "2. Or initialize manually with: sudo mysql"
