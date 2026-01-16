# MeCP Dashboard Access Guide

## Dashboard URLs

### Current Access (IP Address)
- **Dashboard:** https://34.133.251.18/dashboard
- **Login Page:** https://34.133.251.18/login

### With Domain (once configured)
- **Dashboard:** https://mecp.io/dashboard
- **Login Page:** https://mecp.io/login

## Authentication Requirements

The dashboard uses **Web3 wallet authentication**. You need:

1. **MetaMask** (or another EVM-compatible wallet) browser extension
2. **Wallet address** that matches the configured `allowed_address` in `config.toml`

### Current Allowed Address
```
0x2F5989AF501F5879Bb059d5af51e8b204717a9D9
```

## Step-by-Step Access Instructions

### Step 1: Install MetaMask (if not already installed)

1. Go to https://metamask.io
2. Install the browser extension
3. Create or import a wallet
4. **Important:** Make sure your wallet address matches the allowed address above

### Step 2: Access the Dashboard

1. Open your browser
2. Navigate to: `https://34.133.251.18/dashboard` or `https://34.133.251.18/login`
3. You'll be redirected to the login page if not authenticated

### Step 3: Connect Your Wallet

1. Click **"Connect Wallet"** button
2. MetaMask will pop up asking for connection
3. Select the account that matches: `0x2F5989AF501F5879Bb059d5af51e8b204717a9D9`
4. Click **"Next"** and **"Connect"**

### Step 4: Sign the Authentication Message

1. After connecting, click **"Sign Message"** button
2. MetaMask will show a message to sign
3. **This is gasless** - no transaction fee required
4. Click **"Sign"** in MetaMask
5. You'll be automatically redirected to the dashboard

### Step 5: Access Dashboard

Once authenticated, you'll see:
- **Real-time Metrics** - API call rates, success rates, response times
- **Request History** - Detailed logs with full request/response data
- **Error Tracking** - Dedicated error monitoring
- **Analytics** - Per-endpoint statistics and trends

## Troubleshooting

### "Address not allowed" Error

**Problem:** Your wallet address doesn't match the configured address.

**Solution:** Update `config.toml` on the server:

```bash
ssh mecp-gce
nano ~/MeCP/config.toml
```

Change the `allowed_address` to your wallet address:
```toml
[auth]
allowed_address = "0xYourWalletAddress"  # Your actual wallet address
```

Then restart the service:
```bash
sudo systemctl restart mecp
```

### SSL Certificate Warning

**Problem:** Browser shows "Not Secure" or certificate warning.

**Current Status:** Using self-signed certificate

**Solutions:**
1. **Accept the warning** (click "Advanced" → "Proceed to site")
2. **Set up Let's Encrypt** with your domain (see `CLOUDFLARE_SETUP.md`)

### MetaMask Not Detected

**Problem:** "MetaMask is not installed" error.

**Solution:**
1. Install MetaMask browser extension
2. Refresh the page
3. Make sure MetaMask is unlocked

### Can't Connect to Dashboard

**Problem:** Page won't load or connection error.

**Check:**
1. Service is running: `ssh mecp-gce "sudo systemctl status mecp"`
2. Firewall allows HTTPS: Port 443 should be open
3. Nginx is running: `ssh mecp-gce "sudo systemctl status nginx"`

### Token Expired

**Problem:** Dashboard says "Unauthorized" after some time.

**Solution:**
- Tokens expire after 24 hours (configurable)
- Simply log in again using the same process

## Disabling Authentication (For Testing)

If you want to disable authentication temporarily for testing:

```bash
ssh mecp-gce
nano ~/MeCP/config.toml
```

Change:
```toml
[auth]
enabled = false
```

Then restart:
```bash
sudo systemctl restart mecp
```

**Warning:** This makes the dashboard publicly accessible without authentication!

## Quick Access Commands

```bash
# Check if dashboard is accessible
curl -k https://34.133.251.18/health

# View service logs
ssh mecp-gce "sudo journalctl -u mecp -f"

# Check authentication status
ssh mecp-gce "grep allowed_address ~/MeCP/config.toml"
```

## Dashboard Features

Once logged in, you can:

- 📊 **View Metrics** - Real-time API statistics
- 📝 **Browse Logs** - All API requests and responses
- 🐛 **Monitor Errors** - Track and debug issues
- 📈 **Analytics** - Performance trends and insights
- 🔄 **Auto-refresh** - Updates every 5 seconds

## Security Notes

- Authentication uses **cryptographic signatures** (no passwords)
- **Gasless** - No blockchain transactions required
- **JWT tokens** - Secure session management
- **24-hour sessions** - Automatic expiration
- Only the configured wallet address can access

## Next Steps

1. ✅ Access dashboard at `https://34.133.251.18/dashboard`
2. ✅ Connect your MetaMask wallet
3. ✅ Sign the authentication message
4. ✅ Explore the dashboard features

For more details, see `WEB3_AUTH_GUIDE.md`
