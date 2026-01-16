# Cloudflare Configuration Fix for mecp.io

## Problem

When accessing `http://mecp.io` in browser, you're seeing a domain vendor unavailable page instead of your server.

## Root Cause

This happens when **Cloudflare Proxy is enabled** (orange cloud) but the domain isn't fully configured, OR Cloudflare is showing a default page.

## Solution: Check Cloudflare DNS Settings

### Step 1: Verify DNS Record

1. Go to Cloudflare Dashboard: https://dash.cloudflare.com
2. Select domain: `mecp.io`
3. Go to **DNS → Records**
4. Find the A record for `mecp.io`

### Step 2: Check Proxy Status

**CRITICAL:** The proxy status should be:
- ✅ **DNS only** (gray cloud) - Direct connection to your server
- ❌ **Proxied** (orange cloud) - Goes through Cloudflare (can cause issues)

**If it's orange (proxied):**
1. Click on the A record
2. Click the orange cloud icon to turn it **gray** (DNS only)
3. Wait 1-2 minutes for changes to propagate

### Step 3: Verify SSL/TLS Settings

1. Go to **SSL/TLS** in Cloudflare dashboard
2. Set encryption mode to: **Full** or **Full (strict)**
3. This ensures HTTPS works correctly

### Step 4: Check Page Rules

1. Go to **Rules → Page Rules**
2. Make sure there are no rules blocking or redirecting `mecp.io/*`
3. If there are conflicting rules, disable or delete them

## Current Server Status

✅ **Server is working correctly:**
- HTTP (port 80): Redirects to HTTPS (301)
- HTTPS (port 443): Working with Let's Encrypt certificate
- DNS: Points to `34.133.251.18`
- Nginx: Running and configured correctly

## Testing

After making changes in Cloudflare:

```bash
# Test HTTP (should redirect to HTTPS)
curl -I http://mecp.io

# Test HTTPS (should work)
curl -I https://mecp.io

# Test dashboard
curl -I https://mecp.io/dashboard
```

## Common Issues

### Issue 1: Orange Cloud (Proxy Enabled)

**Symptom:** Browser shows Cloudflare default page or "domain unavailable"

**Fix:** Turn proxy OFF (gray cloud) in DNS settings

### Issue 2: SSL/TLS Mode Wrong

**Symptom:** HTTPS doesn't work or shows errors

**Fix:** Set SSL/TLS mode to "Full" or "Full (strict)"

### Issue 3: Browser Cache

**Symptom:** Still seeing old page after fixing Cloudflare

**Fix:** 
- Clear browser cache
- Use incognito/private mode
- Or wait 5-10 minutes

### Issue 4: DNS Propagation

**Symptom:** Changes not taking effect

**Fix:**
- Wait 2-5 minutes after changing DNS
- Clear local DNS cache
- Use different DNS server (8.8.8.8)

## Recommended Cloudflare Settings

### DNS Record
```
Type: A
Name: @ (or blank)
IPv4: 34.133.251.18
Proxy: OFF (gray cloud) ⚠️ IMPORTANT
TTL: Auto
```

### SSL/TLS Settings
```
Encryption mode: Full
Always Use HTTPS: ON (optional)
Minimum TLS Version: 1.2
```

### Page Rules
- No rules needed for basic setup
- If you add rules later, make sure they don't conflict

## Verification Checklist

- [ ] DNS A record points to `34.133.251.18`
- [ ] Proxy is OFF (gray cloud, not orange)
- [ ] SSL/TLS mode is "Full"
- [ ] No conflicting page rules
- [ ] Wait 2-5 minutes after changes
- [ ] Clear browser cache
- [ ] Test in incognito mode

## Quick Fix Command

If you want to test directly without Cloudflare:

```bash
# Add to /etc/hosts (temporary, for testing only)
echo "34.133.251.18 mecp.io" | sudo tee -a /etc/hosts

# Then access: https://mecp.io/dashboard
# (Remove from /etc/hosts after testing)
```

## Next Steps

1. **Check Cloudflare DNS** - Make sure proxy is OFF (gray)
2. **Wait 2-5 minutes** for changes to propagate
3. **Clear browser cache** or use incognito mode
4. **Test again** - Should work now!

If still not working after these steps, the issue might be:
- Cloudflare account/domain configuration
- Domain registrar settings
- Browser-specific issues
