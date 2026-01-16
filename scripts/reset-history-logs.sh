#!/bin/bash

# Reset history logs table in MySQL
# This script clears all data from the history_logs table

set -e

echo "========================================="
echo "MeCP History Logs Reset Script"
echo "========================================="
echo ""

# Load configuration
DB_NAME="mecp_db"
DB_USER="mecp_user"
DB_PASS="mecp_password"

# Check if MySQL is running
if ! sudo systemctl is-active --quiet mysql; then
    echo "Error: MySQL service is not running"
    echo "Please start MySQL first: sudo systemctl start mysql"
    exit 1
fi

echo "WARNING: This will delete all history logs!"
echo "Database: $DB_NAME"
echo ""
read -p "Are you sure you want to continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Reset cancelled."
    exit 0
fi

echo ""
echo "Resetting history_logs table..."

# Truncate the history_logs table
mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" <<EOF
TRUNCATE TABLE history_logs;
SELECT 'History logs table reset successfully!' as Status;
EOF

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ History logs table has been reset"
    echo "✓ All API call history has been cleared"
else
    echo ""
    echo "✗ Failed to reset history logs table"
    exit 1
fi

echo ""
echo "Reset complete!"
