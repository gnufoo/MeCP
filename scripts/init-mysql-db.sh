#!/bin/bash

# Initialize MySQL database for MeCP
# Creates database, user, and required tables

set -e

echo "========================================="
echo "MeCP MySQL Database Initialization"
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

echo "Creating database and user..."

# Create database and user
sudo mysql <<EOF
CREATE DATABASE IF NOT EXISTS $DB_NAME;
CREATE USER IF NOT EXISTS '$DB_USER'@'localhost' IDENTIFIED BY '$DB_PASS';
GRANT ALL PRIVILEGES ON $DB_NAME.* TO '$DB_USER'@'localhost';
FLUSH PRIVILEGES;
SELECT 'Database and user created successfully!' as Status;
EOF

echo ""
echo "Creating tables..."

# Create history_logs table
mysql -u"$DB_USER" -p"$DB_PASS" "$DB_NAME" < "$(dirname "$0")/setup_history_logs.sql"

if [ $? -eq 0 ]; then
    echo ""
    echo "✓ Database initialized successfully!"
    echo "✓ Database: $DB_NAME"
    echo "✓ User: $DB_USER"
    echo "✓ Tables created: history_logs"
else
    echo ""
    echo "✗ Failed to create tables"
    exit 1
fi

echo ""
echo "Initialization complete!"
