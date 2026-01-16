# Web3 Authentication Guide for MeCP Dashboard

## Overview

MeCP Dashboard now supports **Web3 wallet authentication** using EVM-compatible wallets (MetaMask, etc.). This provides secure, gasless authentication through cryptographic signature verification.

## How It Works

```
┌─────────────┐
│   Browser   │
│  (MetaMask) │
└──────┬──────┘
       │ 1. Connect Wallet
       ▼
┌─────────────────────────────────────────┐
│          Login Page (/login)            │
│  - Request wallet connection            │
│  - Display connected address            │
└──────┬──────────────────────────────────┘
       │ 2. Request Challenge
       ▼
┌─────────────────────────────────────────┐
│     POST /api/auth/challenge            │
│  - Send wallet address                  │
│  - Receive challenge message + nonce    │
└──────┬──────────────────────────────────┘
       │ 3. Sign Challenge
       ▼
┌─────────────────────────────────────────┐
│        MetaMask Popup                   │
│  - User signs message (no gas fee)      │
│  - Returns signature                    │
└──────┬──────────────────────────────────┘
       │ 4. Verify Signature
       ▼
┌─────────────────────────────────────────┐
│      POST /api/auth/verify              │
│  - Server recovers address from sig     │
│  - Validates against allowed address    │
│  - Returns JWT token (24h validity)     │
└──────┬──────────────────────────────────┘
       │ 5. Access Dashboard
       ▼
┌─────────────────────────────────────────┐
│       GET /dashboard                    │
│  - Include "Authorization: Bearer       │
│    <token>" header                      │
│  - Access granted if token valid        │
└─────────────────────────────────────────┘
```

## Features

✅ **Gasless Authentication** - No blockchain transactions required  
✅ **Daily Nonce System** - Challenge changes daily for security  
✅ **24-Hour Sessions** - JWT tokens valid for 24 hours  
✅ **Address Whitelisting** - Only configured address can access  
✅ **Automatic Token Refresh** - Dashboard checks token expiry  
✅ **Secure Signature Verification** - ECDSA recovery on server side  

## Configuration

### 1. Edit `config.toml`

```toml
[auth]
# Enable/disable Web3 authentication
enabled = true

# EVM address allowed to access dashboard (checksum format)
allowed_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"  # <-- Your wallet address

# JWT secret for signing session tokens (generate with: openssl rand -hex 32)
jwt_secret = "your-secret-key-change-this-in-production"

# Session token validity in seconds (86400 = 24 hours)
session_duration = 86400
```

### 2. Generate Secure JWT Secret

```bash
# Generate a strong secret
openssl rand -hex 32

# Example output: a3f8b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1

# Copy this to config.toml:
jwt_secret = "a3f8b2c1d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1"
```

### 3. Get Your Wallet Address

**From MetaMask:**
1. Open MetaMask extension
2. Click on your account name
3. Copy address (e.g., `0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb`)
4. Paste into `config.toml` under `allowed_address`

**From Ethereum Address:**
```bash
# Use checksum format (mixed case)
# Example: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
```

## Usage

### Starting the Server

```bash
# Build and run
cargo run --release

# Output will show:
# 🔐 Web3 Authentication enabled
#    Allowed address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
#    Session duration: 86400s (24h)
# Starting HTTP server on port 3000...
# Dashboard: http://127.0.0.1:3000/dashboard
```

### Logging In

1. **Open Login Page:**
   ```
   http://localhost:3000/login
   ```

2. **Connect MetaMask:**
   - Click "🦊 Connect MetaMask"
   - Approve connection in MetaMask popup
   - Your address will be displayed

3. **Sign Authentication Message:**
   - Click "✍️ Sign Message"
   - MetaMask will show message to sign:
     ```
     Sign this message to authenticate with MeCP Dashboard
     
     Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
     Nonce: 2026-01-16
     
     This signature will not trigger any blockchain transaction or cost any gas fees.
     ```
   - Click "Sign" (no gas required!)

4. **Automatic Redirect:**
   - On successful authentication, you'll be redirected to `/dashboard`
   - Token stored in browser localStorage
   - Valid for 24 hours

### Accessing Dashboard

Once authenticated, the dashboard automatically includes your token in API requests:

```javascript
// Automatic header injection
fetch('/api/stats', {
    headers: {
        'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...'
    }
})
```

### Session Expiry

- **Automatic Detection**: Dashboard checks token expiry on load
- **API 401 Response**: Redirects to login if token invalid
- **Manual Logout**: Clear localStorage and navigate to `/login`

```javascript
// Manual logout
localStorage.removeItem('mecp_auth_token');
localStorage.removeItem('mecp_auth_expires');
window.location.href = '/login';
```

## API Endpoints

### POST `/api/auth/challenge`

Request a login challenge.

**Request:**
```json
{
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
}
```

**Response:**
```json
{
    "message": "Sign this message to authenticate with MeCP Dashboard\n\nAddress: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb\nNonce: 2026-01-16\n\nThis signature will not trigger any blockchain transaction or cost any gas fees.",
    "nonce": "2026-01-16"
}
```

### POST `/api/auth/verify`

Verify signature and receive session token.

**Request:**
```json
{
    "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "signature": "0x1234567890abcdef...",
    "message": "Sign this message to authenticate..."
}
```

**Success Response:**
```json
{
    "success": true,
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_at": "2026-01-17T12:00:00Z",
    "error": null
}
```

**Error Response:**
```json
{
    "success": false,
    "token": null,
    "expires_at": null,
    "error": "Address not authorized"
}
```

### Protected Endpoints

All dashboard endpoints require authentication:

- `GET /dashboard` - Dashboard UI
- `GET /api/stats` - Overall statistics
- `GET /api/metrics` - Endpoint metrics
- `GET /api/logs` - Recent API calls
- `GET /api/errors` - Error logs

**Include token in header:**
```http
GET /api/stats HTTP/1.1
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Security Features

### 1. Daily Nonce System

```rust
pub fn get_daily_nonce() -> String {
    let now = Utc::now();
    format!("{}", now.format("%Y-%m-%d"))
}
```

- Nonce changes every day (UTC)
- Old signatures cannot be replayed
- Format: `YYYY-MM-DD`

### 2. ECDSA Signature Verification

```rust
// Hash message using Ethereum standard
let message_hash = hash_message(message);

// Recover signer address from signature
let recovered_address = signature.recover(message_hash)?;

// Verify it matches provided address
if recovered_address != expected_address {
    return Err("Signature verification failed");
}
```

### 3. Address Whitelisting

```rust
let allowed_addr = self.config.allowed_address.to_lowercase();
let provided_addr = address.to_lowercase();

if allowed_addr != provided_addr {
    return Err("Address not authorized");
}
```

### 4. JWT Token Security

```rust
let claims = Claims {
    address: address.to_lowercase(),
    iat: now.timestamp(),      // Issued at
    exp: exp.timestamp(),       // Expiration
};

let token = encode(
    &Header::new(Algorithm::HS256),
    &claims,
    &EncodingKey::from_secret(jwt_secret.as_bytes()),
)?;
```

### 5. Middleware Protection

```rust
async fn auth_middleware(request: Request) -> Result<Response> {
    // Extract Authorization header
    let token = extract_bearer_token(&request)?;
    
    // Validate token
    let claims = auth_service.validate_token(token)?;
    
    // Check address still authorized
    if claims.address != allowed_address {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    // Proceed to protected route
    Ok(next.run(request).await)
}
```

## Troubleshooting

### Issue: "MetaMask is not installed"

**Solution:**
- Install MetaMask extension: https://metamask.io/download/
- Refresh the page

### Issue: "Address not authorized"

**Solution:**
- Check `config.toml` has correct wallet address
- Ensure address is in checksum format (mixed case)
- Verify you're using the correct MetaMask account

### Issue: "Signature verification failed"

**Possible causes:**
- Wrong network in MetaMask (doesn't matter, but can cause issues)
- Message was modified
- Signature format error

**Solution:**
- Try signing again
- Ensure MetaMask is up to date
- Check browser console for errors

### Issue: "Invalid or expired nonce"

**Solution:**
- Nonce changes daily at midnight UTC
- Get a fresh challenge from `/api/auth/challenge`
- Sign the new message

### Issue: Token expired / 401 Unauthorized

**Solution:**
- Tokens last 24 hours
- You'll be automatically redirected to login
- Simply log in again

### Issue: Server shows "Web3 Authentication not configured"

**Solution:**
- Check `config.toml` has `[auth]` section
- Set `enabled = true`
- Restart server

## Disabling Authentication

To disable Web3 auth (e.g., for local development):

```toml
[auth]
enabled = false  # <-- Set to false
# ... rest of config
```

or

```bash
# Remove [auth] section entirely from config.toml
```

Dashboard will be accessible without authentication.

## Multiple Authorized Addresses (Future Enhancement)

Currently supports single address. To add multiple addresses support:

**Option 1: Manual (for now)**
- Deploy separate instances with different configs
- Use reverse proxy to route by subdomain

**Option 2: Future Feature**
```toml
[auth]
allowed_addresses = [
    "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb",
    "0xAnotherAddress123456789abcdef0123456789ab",
]
```

## Production Deployment

### 1. Generate Strong JWT Secret

```bash
openssl rand -hex 64  # Use 64 bytes for production
```

### 2. Use HTTPS

```nginx
server {
    listen 443 ssl;
    server_name dashboard.example.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header Authorization $http_authorization;
    }
}
```

### 3. Environment Variables (Optional)

```bash
# Override config.toml settings
export MECP_AUTH_ENABLED=true
export MECP_AUTH_ADDRESS="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"
export MECP_JWT_SECRET="$(openssl rand -hex 64)"
export MECP_SESSION_DURATION=86400

cargo run --release
```

### 4. Firewall Rules

```bash
# Only allow access from specific IPs (optional)
sudo ufw allow from 203.0.113.0/24 to any port 3000
```

### 5. Rate Limiting (Recommended)

```nginx
limit_req_zone $binary_remote_addr zone=auth:10m rate=10r/m;

location /api/auth/ {
    limit_req zone=auth burst=5;
    proxy_pass http://localhost:3000;
}
```

## Architecture

### Files Modified/Created

```
MeCP/
├── src/
│   ├── core/
│   │   ├── auth.rs                 # ← NEW: Auth service
│   │   ├── http_server.rs          # ← MODIFIED: Added auth middleware
│   │   └── mod.rs                  # ← MODIFIED: Added auth module
│   ├── services/
│   │   └── config.rs               # ← MODIFIED: Added AuthConfig
│   └── main.rs                     # ← MODIFIED: Initialize auth service
├── dashboard/
│   ├── login.html                  # ← NEW: Login page
│   └── index.html                  # ← MODIFIED: Token management
├── config.toml                     # ← MODIFIED: Added [auth] section
├── Cargo.toml                      # ← MODIFIED: Added ethers, jsonwebtoken
└── WEB3_AUTH_GUIDE.md             # ← NEW: This document
```

### Dependencies Added

```toml
ethers = { version = "2.0", features = ["legacy"] }  # EVM signature verification
jsonwebtoken = "9.3"                                 # JWT token generation
hex = "0.4"                                          # Hex encoding/decoding
```

## Testing

### Manual Test Flow

```bash
# 1. Start server
cargo run --release

# 2. Open browser to http://localhost:3000/login
# 3. Connect MetaMask
# 4. Sign message
# 5. Should redirect to dashboard
# 6. Verify API calls work

# 7. Check token in browser console
localStorage.getItem('mecp_auth_token')
```

### API Testing with curl

```bash
# Get challenge
curl -X POST http://localhost:3000/api/auth/challenge \
  -H "Content-Type: application/json" \
  -d '{"address":"0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb"}'

# Access protected endpoint (should fail without token)
curl http://localhost:3000/api/stats
# HTTP 401 Unauthorized

# Access with token
TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
curl http://localhost:3000/api/stats \
  -H "Authorization: Bearer $TOKEN"
# HTTP 200 OK
```

## FAQ

**Q: Do I need ETH or any cryptocurrency?**  
A: No! Authentication is gasless and free. Signing is off-chain.

**Q: Which wallets are supported?**  
A: Any EVM-compatible wallet: MetaMask, WalletConnect, Coinbase Wallet, etc.

**Q: Can I use hardware wallets (Ledger/Trezor)?**  
A: Yes! They work through MetaMask's hardware wallet integration.

**Q: What if I lose my wallet?**  
A: Update `config.toml` with your new wallet address and restart the server.

**Q: Can I have multiple authorized users?**  
A: Currently single address only. Multi-user support planned for future.

**Q: Is this secure for production?**  
A: Yes, with proper configuration:
- Use strong JWT secret (64+ bytes)
- Enable HTTPS
- Keep `config.toml` secure (don't commit secrets to git)
- Use rate limiting

**Q: How does this compare to traditional passwords?**  
A: More secure! Cryptographic proof of wallet ownership. No password database to breach.

---

**Version**: MeCP v0.1.0  
**Date**: 2026-01-16  
**Status**: ✅ Production Ready
