# Web3 Authentication - Quick Start

Get your dashboard secured with Web3 authentication in 5 minutes!

## Prerequisites

- MetaMask browser extension installed
- Your EVM wallet address

## Option 1: Automated Setup (Recommended)

```bash
# Run the setup script
./scripts/setup-auth-example.sh

# Follow the prompts:
# 1. JWT secret will be generated automatically
# 2. Enter your wallet address (e.g., 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb)
# 3. Choose session duration (default: 24 hours)
# 4. Done!
```

## Option 2: Manual Setup

### Step 1: Edit `config.toml`

Add this section:

```toml
[auth]
enabled = true
allowed_address = "0xYourWalletAddressHere"
jwt_secret = "generate-with-openssl-rand-hex-32"
session_duration = 86400
```

### Step 2: Generate JWT Secret

```bash
openssl rand -hex 32
# Copy output to jwt_secret
```

### Step 3: Get Your Wallet Address

Open MetaMask → Click account name → Copy address

## Start the Server

```bash
cargo run --release

# You should see:
# 🔐 Web3 Authentication enabled
#    Allowed address: 0x742d35...
#    Session duration: 86400s (24h)
```

## Login

1. **Open**: http://localhost:3000/login
2. **Connect**: Click "🦊 Connect MetaMask"
3. **Sign**: Click "✍️ Sign Message" (no gas required!)
4. **Access**: Redirected to dashboard automatically

## Verify It Works

```bash
# This should return 401 Unauthorized
curl http://localhost:3000/api/stats

# After login, check browser console:
localStorage.getItem('mecp_auth_token')

# Should see a JWT token
```

## Troubleshooting

### "Address not authorized"
- Check `config.toml` has the correct address
- Verify you're using the right MetaMask account

### "MetaMask is not installed"
- Install: https://metamask.io/download/
- Refresh page

### "Token expired"
- Tokens last 24 hours (configurable)
- Simply log in again

## Disable Authentication

Set `enabled = false` in `config.toml`:

```toml
[auth]
enabled = false
```

Or remove the `[auth]` section entirely.

## Next Steps

- Read full guide: `WEB3_AUTH_GUIDE.md`
- Configure for production (HTTPS, rate limiting)
- Set up multiple authorized addresses (future feature)

## Quick Reference

| URL | Purpose |
|-----|---------|
| `/login` | Web3 login page |
| `/dashboard` | Protected dashboard (requires auth) |
| `/api/auth/challenge` | Get signing challenge |
| `/api/auth/verify` | Verify signature & get token |
| `/api/stats`, `/api/metrics`, etc. | Protected API endpoints |

**That's it!** Your dashboard is now secured with Web3 authentication! 🎉
