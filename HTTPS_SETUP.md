# HTTPS Setup Guide for MeCP on GCE

## Current Setup ✅

Your MeCP server is now configured with HTTPS using a self-signed certificate. This works for testing, but browsers will show a security warning.

**Current Endpoints:**
- HTTP: `http://34.133.251.18` (redirects to HTTPS)
- HTTPS: `https://34.133.251.18` (self-signed certificate)

## Option 1: Let's Encrypt with Domain Name (Recommended)

For production use, you should set up Let's Encrypt SSL certificates with a real domain name.

### Prerequisites
1. A domain name (e.g., `mecp.example.com`)
2. DNS A record pointing to your GCE instance IP: `34.133.251.18`

### Setup Steps

#### 1. Point Your Domain to the Server

Add an A record in your DNS:
```
Type: A
Name: mecp (or @ for root domain)
Value: 34.133.251.18
TTL: 300
```

Wait for DNS propagation (can take a few minutes to hours).

#### 2. Update Nginx Configuration

SSH into your server and update the configuration:

```bash
ssh mecp-gce
sudo nano /etc/nginx/sites-available/mecp
```

Replace `34.133.251.18` with your domain name in both server blocks:
```nginx
server {
    listen 80;
    server_name your-domain.com;  # Change this

    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name your-domain.com;  # Change this
    # ... rest of config
}
```

Test and reload:
```bash
sudo nginx -t
sudo systemctl reload nginx
```

#### 3. Obtain Let's Encrypt Certificate

```bash
sudo certbot --nginx -d your-domain.com
```

Certbot will:
- Automatically obtain SSL certificates
- Update Nginx configuration
- Set up auto-renewal

#### 4. Verify Auto-Renewal

Let's Encrypt certificates expire every 90 days. Certbot sets up auto-renewal, but verify it:

```bash
sudo certbot renew --dry-run
```

## Option 2: Keep Self-Signed Certificate

If you don't have a domain name, you can keep the self-signed certificate. Note that:
- Browsers will show security warnings
- You'll need to accept the certificate manually
- Not recommended for production

### Accessing with Self-Signed Certificate

**Browser:**
1. Navigate to `https://34.133.251.18`
2. Click "Advanced" → "Proceed to site" (or similar)

**cURL:**
```bash
curl -k https://34.133.251.18/health
```

**API Clients:**
Most HTTP clients have options to skip SSL verification for self-signed certs.

## Current Configuration

### Nginx Configuration
- Location: `/etc/nginx/sites-available/mecp`
- SSL Certificate: `/etc/nginx/ssl/mecp.crt`
- SSL Key: `/etc/nginx/ssl/mecp.key`

### Firewall Rules
- Port 80 (HTTP): Open
- Port 443 (HTTPS): Open
- Port 8080 (MeCP): Internal only (proxied through Nginx)

### Service Status

Check Nginx status:
```bash
ssh mecp-gce "sudo systemctl status nginx"
```

View Nginx logs:
```bash
ssh mecp-gce "sudo tail -f /var/log/nginx/access.log"
ssh mecp-gce "sudo tail -f /var/log/nginx/error.log"
```

## Testing HTTPS

### Health Check
```bash
curl -k https://34.133.251.18/health
```

### API Endpoint
```bash
curl -k -X POST https://34.133.251.18/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"resources/list"}'
```

### Dashboard
Open in browser: `https://34.133.251.18/dashboard`

## Troubleshooting

### Certificate Issues

**Regenerate self-signed certificate:**
```bash
ssh mecp-gce
sudo rm /etc/nginx/ssl/mecp.*
sudo openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/nginx/ssl/mecp.key \
  -out /etc/nginx/ssl/mecp.crt \
  -subj '/CN=34.133.251.18/O=MeCP/C=US'
sudo systemctl reload nginx
```

### Nginx Not Starting

**Check configuration:**
```bash
ssh mecp-gce "sudo nginx -t"
```

**View error logs:**
```bash
ssh mecp-gce "sudo journalctl -u nginx -n 50"
```

### Firewall Issues

**Check firewall rules:**
```bash
gcloud compute firewall-rules list --project=tony-projects-464503 | grep -E '(allow-http|allow-https)'
```

## Security Recommendations

1. **Use Let's Encrypt** for production (free, trusted certificates)
2. **Enable HSTS** (already configured in Nginx)
3. **Keep certificates updated** (auto-renewal is set up with Certbot)
4. **Monitor certificate expiration:**
   ```bash
   sudo certbot certificates
   ```

## Next Steps

1. If you have a domain: Follow Option 1 to set up Let's Encrypt
2. If no domain: Keep self-signed for now, but plan to get a domain for production
3. Update your application clients to use HTTPS endpoints
4. Consider setting up monitoring for certificate expiration
