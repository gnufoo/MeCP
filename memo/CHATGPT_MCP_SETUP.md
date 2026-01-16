# ChatGPT MCP Connector Setup Guide

## Overview

Your MeCP server now supports Server-Sent Events (SSE) which is required for ChatGPT's MCP connector integration.

## Endpoints

### Available Endpoints

1. **SSE Endpoint (for ChatGPT):**
   - URL: `https://34.133.251.18/sse`
   - Method: GET
   - Content-Type: `text/event-stream`
   - Purpose: Streaming connection for MCP protocol

2. **MCP JSON-RPC Endpoint:**
   - URL: `https://34.133.251.18/mcp`
   - Method: POST
   - Content-Type: `application/json`
   - Purpose: Standard MCP JSON-RPC requests

3. **Health Check:**
   - URL: `https://34.133.251.18/health`
   - Method: GET

## Setting Up in ChatGPT

### Step 1: Access ChatGPT Connectors

1. Open ChatGPT
2. Go to Settings → Connectors (or Developer Mode → Connectors)
3. Click "Add Connector" or "Create New Connector"

### Step 2: Configure the Connector

**Connection Details:**
- **Name:** MeCP Server
- **Type:** HTTP/SSE
- **Base URL:** `https://34.133.251.18`
- **SSE Endpoint:** `/sse`
- **MCP Endpoint:** `/mcp`

**Authentication:**
- If you have Web3 auth enabled, you may need to configure authentication
- For testing, you can disable auth in `config.toml` by setting `[auth] enabled = false`

### Step 3: Test Connection

ChatGPT should:
1. Connect to the SSE endpoint
2. Receive a "connected" event
3. Be able to send MCP requests

## Troubleshooting

### Error: "Error creating connector"

**Possible causes:**

1. **SSL Certificate Issue:**
   - The server uses a self-signed certificate
   - ChatGPT may reject self-signed certificates
   - **Solution:** Set up Let's Encrypt with a real domain (see `HTTPS_SETUP.md`)

2. **SSE Endpoint Not Responding:**
   - Check if the endpoint is accessible: `curl -k -N https://34.133.251.18/sse`
   - Verify the service is running: `ssh mecp-gce "sudo systemctl status mecp"`

3. **CORS Issues:**
   - The server has CORS enabled, but verify headers are correct
   - Check Nginx configuration allows CORS

4. **Firewall/Network Issues:**
   - Ensure port 443 is open
   - Check GCE firewall rules

### Testing the SSE Endpoint

```bash
# Test SSE connection
curl -k -N -H "Accept: text/event-stream" https://34.133.251.18/sse

# Expected output:
# event: connected
# data: {"status":"connected","service":"mecp","version":"0.1.0","protocol":"sse"}
```

### Testing the MCP Endpoint

```bash
# Test MCP JSON-RPC
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

## Advanced Configuration

### Using a Custom Domain

If you have a domain name:

1. Set up Let's Encrypt (see `HTTPS_SETUP.md`)
2. Update the connector URL to use your domain
3. This will resolve SSL certificate warnings

### Disabling Authentication

If ChatGPT has issues with authentication:

1. SSH into the server: `ssh mecp-gce`
2. Edit config: `nano ~/MeCP/config.toml`
3. Set `[auth] enabled = false`
4. Restart: `sudo systemctl restart mecp`

### Custom SSE Events

The current SSE implementation sends:
- **Connected event:** When client connects
- **Heartbeat events:** Every 30 seconds to keep connection alive

You can extend this to handle MCP requests over SSE if needed.

## MCP Protocol Support

Your server supports the following MCP methods:

- `initialize` - Initialize MCP session
- `resources/list` - List available resources
- `resources/read` - Read a resource
- `tools/list` - List available tools
- `tools/call` - Call a tool
- `prompts/list` - List available prompts
- `prompts/get` - Get a prompt

## Verification Checklist

- [ ] SSE endpoint accessible: `https://34.133.251.18/sse`
- [ ] MCP endpoint accessible: `https://34.133.251.18/mcp`
- [ ] Health check working: `https://34.133.251.18/health`
- [ ] Service running: `sudo systemctl status mecp`
- [ ] Firewall allows HTTPS: Port 443 open
- [ ] SSL certificate configured (self-signed or Let's Encrypt)

## Next Steps

1. **For Production:** Set up Let's Encrypt with a real domain
2. **For Testing:** Use the current self-signed certificate (accept warnings)
3. **Monitor Logs:** `ssh mecp-gce "sudo journalctl -u mecp -f"`

## Support

If you continue to have issues:

1. Check server logs: `ssh mecp-gce "sudo journalctl -u mecp -n 50"`
2. Check Nginx logs: `ssh mecp-gce "sudo tail -f /var/log/nginx/error.log"`
3. Test endpoints manually with curl
4. Verify firewall rules in GCE console
