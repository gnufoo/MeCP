#!/bin/bash

# Migration script to add response_data column to history_logs table

set -e

echo "========================================="
echo "MeCP Database Migration"
echo "========================================="
echo ""

# Configuration
DB_NAME="mecp_db"
DB_USER="mecp_user"
DB_PASS="mecp_password"

# Check if MySQL is running
if ! sudo systemctl is-active --quiet mysql; then
    echo "Error: MySQL service is not running"
    echo "Please start MySQL first: sudo systemctl start mysql"
    exit 1
fi

echo "📊 Applying migration: Add response_data column..."
echo ""

# Apply migration
mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" < "$(dirname "$0")/migrate_add_response_data.sql"

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Migration applied successfully!"
    echo ""
    echo "Database schema updated:"
    mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" -e "DESCRIBE history_logs;"
else
    echo ""
    echo "✗ Migration failed"
    exit 1
fi

echo ""
echo "Migration complete!"
