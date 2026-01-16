# ChatGPT MCP Connector Troubleshooting

## Current Status

Your MeCP server has:
- ✅ SSE endpoint at `/sse` with proper streaming configuration
- ✅ MCP JSON-RPC endpoint at `/mcp`
- ✅ Nginx configured with buffering disabled for SSE
- ⚠️ Self-signed SSL certificate (likely causing ChatGPT rejection)

## Most Likely Issue: Self-Signed Certificate

ChatGPT's MCP connector **rejects self-signed SSL certificates** for security reasons. This is the most common cause of "Error creating connector".

### Solution 1: Use Let's Encrypt (Recommended)

**You need a domain name for this:**

1. **Get a domain** (e.g., from Namecheap, Google Domains, etc.)
2. **Point DNS to your server:**
   ```
   Type: A
   Name: mecp (or @)
   Value: 34.133.251.18
   TTL: 300
   ```
3. **Set up Let's Encrypt:**
   ```bash
   ssh mecp-gce
   sudo certbot --nginx -d your-domain.com
   ```
4. **Update connector URL in ChatGPT** to use your domain

### Solution 2: Temporary Workaround (Not Recommended)

If you absolutely cannot use a domain right now, you could:
1. Use HTTP instead of HTTPS (not secure, not recommended)
2. Or accept that ChatGPT won't work until you have a valid certificate

## Verification Steps

### 1. Test SSE Endpoint

```bash
# Should stream events immediately (no buffering)
curl -k -N -H "Accept: text/event-stream" https://34.133.251.18/sse
```

**Expected output:**
```
event: connected
data: {"protocol":"sse","service":"mecp","status":"connected","version":"0.1.0"}

event: heartbeat
data: {"timestamp":"2026-01-16T..."}
```

### 2. Test MCP Endpoint

```bash
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {"name": "test", "version": "1.0.0"}
    }
  }'
```

### 3. Check Nginx Configuration

```bash
ssh mecp-gce "sudo nginx -t"
ssh mecp-gce "sudo systemctl status nginx"
```

### 4. Check Service Logs

```bash
ssh mecp-gce "sudo journalctl -u mecp -n 50"
ssh mecp-gce "sudo tail -f /var/log/nginx/error.log"
```

## Common Error Messages

### "Error creating connector"

**Causes:**
1. ❌ Self-signed certificate (most likely)
2. ❌ SSE endpoint not accessible
3. ❌ Nginx buffering (should be fixed now)
4. ❌ CORS issues (should be fixed now)
5. ❌ Firewall blocking

**Solutions:**
- Set up Let's Encrypt with a domain
- Verify endpoints are accessible
- Check firewall rules

### "SSL certificate problem"

**Cause:** Self-signed certificate

**Solution:** Use Let's Encrypt with a real domain

### "Connection timeout"

**Causes:**
1. Firewall blocking port 443
2. Service not running
3. Network issues

**Solutions:**
```bash
# Check firewall
gcloud compute firewall-rules list --project=tony-projects-464503 | grep allow-https

# Check service
ssh mecp-gce "sudo systemctl status mecp"
```

## Quick Fix Checklist

- [ ] Domain name pointing to `34.133.251.18`
- [ ] Let's Encrypt certificate installed
- [ ] Nginx configuration updated with domain
- [ ] SSE endpoint accessible: `curl -k https://your-domain.com/sse`
- [ ] MCP endpoint accessible: `curl -k https://your-domain.com/mcp`
- [ ] ChatGPT connector URL updated to use domain

## Testing Without ChatGPT

You can test the MCP server directly:

```bash
# Initialize
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'

# List tools
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'

# List resources
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"resources/list"}'
```

## Next Steps

1. **Get a domain name** (if you don't have one)
   - Free options: Freenom, or use a subdomain from a service
   - Paid options: Namecheap, Google Domains, etc.

2. **Set up Let's Encrypt:**
   ```bash
   ssh mecp-gce
   sudo certbot --nginx -d your-domain.com
   ```

3. **Update ChatGPT connector** to use your domain

4. **Test connection** in ChatGPT

## Support

If issues persist after setting up Let's Encrypt:

1. Check server logs: `ssh mecp-gce "sudo journalctl -u mecp -f"`
2. Check Nginx logs: `ssh mecp-gce "sudo tail -f /var/log/nginx/error.log"`
3. Verify endpoints with curl
4. Check ChatGPT's connector logs (if available)
