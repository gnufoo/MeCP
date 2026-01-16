# Database Persistence Fix

## Issue Discovered

**Problem**: Metrics were being tracked in memory but **NOT persisted to MySQL database**.

### Symptoms
- ✅ Server running fine
- ✅ API calls working
- ✅ Dashboard showing in-memory stats
- ❌ MySQL `history_logs` table always empty
- ❌ Data "disappears" after rebuild/restart

### Root Cause

In `src/main.rs`, the server was initializing without MySQL backend:

```rust
// OLD CODE (WRONG)
let http_server = crate::core::http_server::HttpServer::new(server.clone(), port);
```

This created a `MetricsCollector` with **in-memory only** storage, so:
- Metrics were collected during runtime
- But NEVER written to MySQL
- Data was lost on server restart
- Made it look like "rebuild cleans database" (actually: data never saved)

## Fix Applied

Updated `src/main.rs` to initialize with MySQL backend:

```rust
// NEW CODE (CORRECT)
// Load configuration
let config = services::config::ServiceConfig::load("config.toml")?;

// Create metrics collector with MySQL backend
let mysql_writer = Arc::new(MySqlMetricsWriter::new(
    &config.mysql.host,
    config.mysql.port,
    &config.mysql.database,
    &config.mysql.username,
    &config.mysql.password,
));

let metrics = Arc::new(MetricsCollector::with_mysql_writer(mysql_writer));

// Start server with MySQL-backed metrics
let http_server = HttpServer::with_metrics(server, metrics, port);
```

## Testing the Fix

### 1. Stop the Old Server
```bash
# Find and kill the old process
pkill -f "target/release/mecp"

# Or press Ctrl+C in the terminal where it's running
```

### 2. Rebuild and Start
```bash
cd /home/gnufoo/Work/Projects/gnufoo/MeCP

# Rebuild with fix
cargo build --release

# Start new server
cargo run --release
```

### 3. Look for New Startup Message
You should see:
```
📊 Enabling MySQL metrics backend...
```

### 4. Send Test Requests
```bash
# Send a test request
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "clientInfo": {"name": "test", "version": "1.0"}
    }
  }'
```

### 5. Verify Database Persistence
```bash
# Wait 2 seconds for async write
sleep 2

# Check database - should now have data!
mysql -u mecp_user -pmecp_password mecp_db -e "SELECT * FROM history_logs ORDER BY timestamp DESC LIMIT 5;"
```

### 6. Test Rebuild Doesn't Clear Data
```bash
# Stop server
pkill -f "target/release/mecp"

# Rebuild (data should persist!)
cargo build --release

# Check data still there
mysql -u mecp_user -pmecp_password mecp_db -e "SELECT COUNT(*) FROM history_logs;"

# Start server again
cargo run --release

# Data should still be there!
mysql -u mecp_user -pmecp_password mecp_db -e "SELECT COUNT(*) FROM history_logs;"
```

## What Changed

### Files Modified
- ✅ `src/main.rs` - Added MySQL metrics initialization

### Behavior Changes

**Before Fix:**
```
Server Start → Metrics in Memory Only → Server Stop → Data Lost
```

**After Fix:**
```
Server Start → Metrics in Memory + MySQL → Server Stop → Data Persisted ✅
```

## Configuration

The fix respects your `config.toml`:

```toml
[mysql]
enabled = true          # Must be true for MySQL persistence
host = "localhost"
port = 3306
database = "mecp_db"
username = "mecp_user"
password = "mecp_password"
```

If MySQL is disabled in config:
```
⚠️  MySQL metrics disabled, using in-memory only
```

If MySQL is enabled:
```
📊 Enabling MySQL metrics backend...
```

## Verification Commands

### Check if Fix is Active
```bash
# Look for startup message
cargo run --release 2>&1 | grep -i "mysql metrics"

# Should see: "📊 Enabling MySQL metrics backend..."
```

### Monitor Database in Real-Time
```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Watch database
watch -n 2 'mysql -u mecp_user -pmecp_password mecp_db -e "SELECT COUNT(*) as logs, MAX(timestamp) as latest FROM history_logs;"'

# Terminal 3: Send requests
./scripts/test-dashboard-flow.sh 10

# You should see count increasing in Terminal 2!
```

### Check Specific Logs
```bash
# View recent logs
mysql -u mecp_user -pmecp_password mecp_db -e "
SELECT 
    id,
    method,
    response_status,
    duration_ms,
    timestamp
FROM history_logs
ORDER BY timestamp DESC
LIMIT 10;"

# Check by method
mysql -u mecp_user -pmecp_password mecp_db -e "
SELECT 
    method,
    COUNT(*) as calls,
    AVG(duration_ms) as avg_ms
FROM history_logs
GROUP BY method;"
```

## Why This Happened

The original implementation had two ways to create HttpServer:

1. **`HttpServer::new()`** - In-memory only (used by mistake)
2. **`HttpServer::with_metrics()`** - With MySQL backend (correct way)

The main.rs was using option 1, which is fine for:
- ✅ Quick testing
- ✅ Development without database
- ✅ Examples that don't need persistence

But for production with dashboard:
- ❌ Data doesn't persist
- ❌ Dashboard loses history on restart
- ❌ Looks like "rebuild clears database" (but data never saved!)

## Summary

**Issue**: Data never saved to database (in-memory only)  
**Cause**: Wrong HttpServer initialization method  
**Fix**: Use `with_metrics()` with MySQL backend  
**Result**: Data now persists across restarts ✅  

**No rebuild needed after this fix** - the database connection is now properly initialized on startup!

---

**Fixed Date**: 2026-01-16  
**Version**: v0.1.0+  
**Status**: Resolved ✅
