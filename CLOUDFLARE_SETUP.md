# Cloudflare Domain Setup for MeCP

## Step 1: Configure Cloudflare DNS

1. **Log in to Cloudflare Dashboard**
   - Go to https://dash.cloudflare.com
   - Select your domain: `mecp.io`

2. **Add DNS Records**

   **Option A: Root Domain (mecp.io)**
   - Type: `A`
   - Name: `@` (or leave blank)
   - IPv4 address: `34.133.251.18`
   - Proxy status: **DNS only** (gray cloud) - Important for Let's Encrypt!
   - TTL: Auto

   **Option B: Subdomain (api.mecp.io or mcp.mecp.io)**
   - Type: `A`
   - Name: `api` (or `mcp`)
   - IPv4 address: `34.133.251.18`
   - Proxy status: **DNS only** (gray cloud)
   - TTL: Auto

   **Important:** Make sure the proxy is **OFF** (gray cloud, not orange). Let's Encrypt needs direct access to your server, and Cloudflare's proxy can interfere.

3. **Wait for DNS Propagation**
   - Usually takes 1-5 minutes
   - Verify with: `dig mecp.io` or `nslookup mecp.io`

## Step 2: Verify DNS is Working

```bash
# Check if DNS is pointing to your server
dig mecp.io +short
# Should return: 34.133.251.18

# Or
nslookup mecp.io
# Should show: 34.133.251.18
```

## Step 3: Set Up Let's Encrypt on Server

Once DNS is propagated, run on the server:

```bash
ssh mecp-gce

# Install certbot if not already installed
sudo apt-get update
sudo apt-get install -y certbot python3-certbot-nginx

# Get SSL certificate (replace with your chosen subdomain if using one)
sudo certbot --nginx -d mecp.io

# Or if using subdomain:
# sudo certbot --nginx -d api.mecp.io
```

Certbot will:
- Automatically obtain SSL certificates
- Update Nginx configuration
- Set up auto-renewal

## Step 4: Update Nginx Configuration

Certbot should automatically update your Nginx config, but verify:

```bash
ssh mecp-gce
sudo nginx -t
sudo systemctl reload nginx
```

## Step 5: Test

```bash
# Test HTTPS with real certificate
curl https://mecp.io/health

# Test SSE endpoint
curl -N -H "Accept: text/event-stream" https://mecp.io/sse

# Test MCP endpoint
curl -X POST https://mecp.io/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
```

## Step 6: Update ChatGPT Connector

In ChatGPT:
1. Go to Connector settings
2. Update the URL to: `https://mecp.io` (or your subdomain)
3. SSE endpoint: `https://mecp.io/sse`
4. MCP endpoint: `https://mecp.io/mcp`
5. Save and test connection

## Troubleshooting

### DNS Not Propagating

- Check Cloudflare proxy is OFF (gray cloud)
- Wait a few more minutes
- Clear DNS cache: `sudo systemd-resolve --flush-caches` (Linux) or `ipconfig /flushdns` (Windows)

### Certbot Fails

**Error: "Failed to verify domain"**
- Ensure DNS is pointing correctly
- Ensure Cloudflare proxy is OFF
- Check firewall allows port 80 (for HTTP-01 challenge)

**Error: "Connection refused"**
- Ensure Nginx is running: `sudo systemctl status nginx`
- Check port 80 is open: `sudo netstat -tlnp | grep 80`

### Cloudflare Proxy Issues

If you want to use Cloudflare's proxy (orange cloud) later:
1. First get Let's Encrypt certificate with proxy OFF
2. Then enable proxy in Cloudflare
3. Update Nginx to use Cloudflare's real IP headers

## Security Notes

- Let's Encrypt certificates expire every 90 days
- Certbot sets up auto-renewal automatically
- Test renewal: `sudo certbot renew --dry-run`

## Next Steps After Setup

1. ✅ Test all endpoints with the new domain
2. ✅ Update ChatGPT connector URL
3. ✅ Verify SSL certificate is trusted
4. ✅ Monitor certificate renewal
