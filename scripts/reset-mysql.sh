#!/bin/bash
# MeCP - Reset MySQL Database Script
# WARNING: This will delete ALL data in MySQL

set -e

echo "╔════════════════════════════════════════╗"
echo "║  MeCP - Reset MySQL Database          ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Load database name from config or use default
DB_NAME="${MECP_MYSQL_DB:-mecp_db}"

echo "⚠️  WARNING: This will DELETE ALL DATA in the MySQL database!"
echo "Database: $DB_NAME"
echo ""

# Confirmation
read -p "Are you sure you want to continue? (type 'yes' to confirm): " confirmation

if [ "$confirmation" != "yes" ]; then
    echo "❌ Reset cancelled"
    exit 0
fi

echo ""
echo "🔄 Resetting MySQL database..."

# Check if MySQL is running
if ! systemctl is-active --quiet mysql 2>/dev/null && ! pgrep -x mysqld >/dev/null 2>&1; then
    echo "❌ MySQL is not running"
    echo "   Start it with: sudo systemctl start mysql"
    exit 1
fi

# Drop and recreate database
sudo mysql -e "DROP DATABASE IF EXISTS $DB_NAME; CREATE DATABASE $DB_NAME; FLUSH PRIVILEGES;"

if [ $? -eq 0 ]; then
    echo "✅ MySQL database reset complete"
else
    echo "❌ Failed to reset MySQL database"
    exit 1
fi

echo ""
echo "Next steps:"
echo "1. Run: mecp-cli start mysql"
echo "2. Verify: mecp-cli status mysql"
