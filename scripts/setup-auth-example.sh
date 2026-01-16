#!/bin/bash
# Setup script for Web3 authentication

echo "🔐 MeCP Web3 Authentication Setup"
echo "=================================="
echo ""

# Generate JWT secret
echo "1. Generating secure JWT secret..."
JWT_SECRET=$(openssl rand -hex 32)
echo "   Generated: $JWT_SECRET"
echo ""

# Prompt for wallet address
echo "2. Enter your EVM wallet address:"
echo "   (Get this from MetaMask - click on account name to copy)"
read -p "   Address: " WALLET_ADDRESS
echo ""

# Validate address format (basic check)
if [[ ! $WALLET_ADDRESS =~ ^0x[a-fA-F0-9]{40}$ ]]; then
    echo "❌ Invalid address format. Must be 0x followed by 40 hex characters."
    exit 1
fi

echo "✅ Address validated: $WALLET_ADDRESS"
echo ""

# Ask for session duration
echo "3. Session token duration in hours (default: 24):"
read -p "   Hours: " SESSION_HOURS
SESSION_HOURS=${SESSION_HOURS:-24}
SESSION_SECONDS=$((SESSION_HOURS * 3600))
echo "   Session duration: ${SESSION_HOURS}h (${SESSION_SECONDS}s)"
echo ""

# Check if config.toml exists
if [ ! -f "config.toml" ]; then
    echo "❌ config.toml not found. Please run this script from the MeCP project root."
    exit 1
fi

# Backup existing config
echo "4. Backing up existing config.toml..."
cp config.toml config.toml.backup
echo "   Backup created: config.toml.backup"
echo ""

# Check if [auth] section already exists
if grep -q "\[auth\]" config.toml; then
    echo "⚠️  Warning: [auth] section already exists in config.toml"
    read -p "   Overwrite? (y/n): " OVERWRITE
    if [[ $OVERWRITE != "y" ]]; then
        echo "   Aborted. No changes made."
        echo "   Your JWT secret: $JWT_SECRET"
        echo "   Please update config.toml manually."
        exit 0
    fi
    
    # Remove existing [auth] section
    sed -i '/\[auth\]/,/^$/d' config.toml
fi

# Add [auth] section
echo "5. Adding [auth] section to config.toml..."
cat >> config.toml << EOF

[auth]
# Web3 Authentication Settings
enabled = true
# EVM address allowed to access dashboard (checksum format)
allowed_address = "$WALLET_ADDRESS"
# JWT secret for signing session tokens
jwt_secret = "$JWT_SECRET"
# Session token validity in seconds ($SESSION_HOURS hours)
session_duration = $SESSION_SECONDS
EOF

echo "   ✅ Config updated successfully!"
echo ""

# Print summary
echo "📋 Configuration Summary"
echo "========================"
echo "Allowed Address: $WALLET_ADDRESS"
echo "JWT Secret: $JWT_SECRET"
echo "Session Duration: ${SESSION_HOURS}h"
echo ""

# Security reminders
echo "🔒 Security Reminders:"
echo "   1. Keep config.toml secure (add to .gitignore)"
echo "   2. Never commit JWT secrets to version control"
echo "   3. Use HTTPS in production"
echo "   4. Backup your wallet private keys"
echo ""

# Next steps
echo "🚀 Next Steps:"
echo "   1. Build and run: cargo run --release"
echo "   2. Open browser: http://localhost:3000/login"
echo "   3. Connect MetaMask with address: $WALLET_ADDRESS"
echo "   4. Sign authentication message"
echo "   5. Access dashboard!"
echo ""

echo "✅ Setup complete!"
