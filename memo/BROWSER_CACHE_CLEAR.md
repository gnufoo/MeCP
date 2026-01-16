# Clear Browser Cache for mecp.io

## Chrome/Edge (Chromium-based)

### Method 1: Clear Site Data (Recommended)
1. Open Chrome
2. Press `F12` to open DevTools
3. Right-click the refresh button (while DevTools is open)
4. Select **"Empty Cache and Hard Reload"**

### Method 2: Clear Specific Site
1. Click the lock icon (or info icon) in the address bar
2. Click **"Site settings"**
3. Click **"Clear data"** or **"Reset permissions"**
4. Close and reopen the browser

### Method 3: Clear SSL State
1. Go to: `chrome://settings/clearBrowserData`
2. Select **"Advanced"** tab
3. Check **"Hosted app data"** and **"Cached images and files"**
4. Time range: **"All time"**
5. Click **"Clear data"**

### Method 4: Clear HSTS (HTTP Strict Transport Security)
1. Go to: `chrome://net-internals/#hsts`
2. In "Delete domain security policies", enter: `mecp.io`
3. Click **"Delete"**
4. Also enter: `34.133.251.18` and click **"Delete"**

### Method 5: Full Reset (Nuclear Option)
1. Go to: `chrome://settings/reset`
2. Click **"Restore settings to their original defaults"**
3. Confirm

## Firefox

### Method 1: Clear Site Data
1. Press `Ctrl+Shift+Delete` (or `Cmd+Shift+Delete` on Mac)
2. Select **"Everything"** for time range
3. Check **"Cache"** and **"Cookies and Site Data"**
4. Click **"Clear Now"**

### Method 2: Clear SSL State
1. Go to: `about:preferences#privacy`
2. Scroll to **"Cookies and Site Data"**
3. Click **"Clear Data"**
4. Check **"Cached Web Content"** and **"Cookies and Site Data"**
5. Click **"Clear"**

### Method 3: Clear HSTS
1. Go to: `about:config`
2. Search for: `security.tls.insecure_fallback_hosts`
3. Add: `mecp.io` (if not already there)
4. Search for: `dom.security.https_only_mode`
5. Set to: `false` (temporarily)
6. Restart Firefox

## Safari

### Method 1: Clear Cache
1. Press `Cmd+Option+E` to empty caches
2. Or: Safari → Preferences → Advanced → "Show Develop menu"
3. Develop → Empty Caches

### Method 2: Clear Site Data
1. Safari → Preferences → Privacy
2. Click **"Manage Website Data"**
3. Search for: `mecp.io`
4. Click **"Remove"**
5. Click **"Remove Now"**

## Quick Fix: Use Incognito/Private Mode

**Easiest solution:**
1. Open a new **Incognito/Private window**
2. Go to: `https://mecp.io/dashboard`
3. This bypasses all cache

## Command Line (Chrome/Edge)

Close all Chrome windows, then run:

**Windows:**
```cmd
taskkill /F /IM chrome.exe
del /F /S /Q "%LOCALAPPDATA%\Google\Chrome\User Data\Default\Cache\*"
del /F /S /Q "%LOCALAPPDATA%\Google\Chrome\User Data\Default\Code Cache\*"
```

**Linux/Mac:**
```bash
pkill chrome
rm -rf ~/.cache/google-chrome/*
rm -rf ~/.config/google-chrome/Default/Cache/*
```

## Verify DNS in Browser

1. Open DevTools (`F12`)
2. Go to **Network** tab
3. Check **"Disable cache"**
4. Try accessing the site again
5. Look at the request - it should show the correct IP

## If Still Not Working

Try accessing via IP directly first:
- `https://34.133.251.18/dashboard`

If that works, the issue is DNS-related. Try:
1. Use a different DNS server (8.8.8.8 or 1.1.1.1)
2. Or wait 10-15 minutes for DNS to fully propagate
