# MeCP Dashboard - Complete Usage Guide

## 🎯 What You Just Got

A complete monitoring dashboard for your MeCP server that:
- **Tracks every API call** with full details
- **Stores history** in MySQL for long-term analysis
- **Shows real-time metrics** with auto-refresh
- **Displays errors** prominently for debugging
- **Provides REST API** for programmatic access

## 🚀 Getting Started (3 Steps)

### Step 1: Initialize the Database

```bash
# Start MySQL if not running
sudo systemctl start mysql

# Initialize the history_logs table
./scripts/init-mysql-db.sh
```

This creates:
- ✅ `history_logs` table in MySQL
- ✅ Indexes for fast queries
- ✅ View for statistics

### Step 2: Run Your Server

```bash
# Build and run (port 3000 by default)
cargo run --release
```

You should see:
```
MCP HTTP Server starting on 127.0.0.1:3000
Dashboard available at http://127.0.0.1:3000/dashboard
```

### Step 3: Access the Dashboard

Open your browser:
```
http://127.0.0.1:3000/dashboard
```

## 📊 Dashboard Overview

### Top Section - Quick Stats
- **Total Calls**: How many API calls received
- **Success Rate**: Percentage of successful calls (target: 95%+)
- **Total Errors**: Number of failed requests
- **Avg Response Time**: Average duration (target: < 100ms)

### Endpoint Metrics Table
Shows details for each MCP method:
- `initialize` - Server initialization
- `resources/list` - List available resources
- `resources/read` - Read specific resource
- `tools/list` - List available tools
- `tools/call` - Execute a tool
- `prompts/list` - List available prompts
- `prompts/get` - Generate a prompt

### Recent API Calls
Last 100 calls with timestamps, methods, status, and duration.

### Recent Errors
Last 50 errors with detailed error messages.

## 🧪 Test the Dashboard

### Make Some API Calls

```bash
# Initialize
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

# List tools
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }'

# Call a tool
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "hello_world",
      "arguments": {"name": "Dashboard"}
    }
  }'

# Generate an error (unknown tool)
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/call",
    "params": {
      "name": "nonexistent_tool",
      "arguments": {}
    }
  }'
```

### Check the Dashboard

Refresh the dashboard to see:
- ✅ Total calls increased
- ✅ Success rate calculated
- ✅ Error appears in Recent Errors section
- ✅ All calls listed in Recent API Calls

## 🔍 Using the REST API

### Get Overall Stats

```bash
curl http://127.0.0.1:3000/api/stats | jq .
```

Response:
```json
{
  "total_calls": 4,
  "total_errors": 1,
  "success_rate": 75.0,
  "avg_duration_ms": 42.5,
  "endpoints_count": 3,
  "recent_logs_count": 4,
  "timestamp": "2026-01-16T12:34:56Z"
}
```

### Get Endpoint Metrics

```bash
curl http://127.0.0.1:3000/api/metrics | jq '.metrics'
```

### Get Recent Logs

```bash
curl http://127.0.0.1:3000/api/logs | jq '.logs[] | {method, status, duration_ms}'
```

### Get Errors Only

```bash
curl http://127.0.0.1:3000/api/errors | jq '.errors[] | {method, error_message}'
```

## 🗄️ Database Queries

### Connect to MySQL

```bash
mysql -u mecp_user -p mecp_db
# Password: mecp_password
```

### View Recent Logs

```sql
SELECT 
    timestamp,
    method,
    response_status,
    duration_ms,
    error_message
FROM history_logs
ORDER BY timestamp DESC
LIMIT 10;
```

### Check Error Rate

```sql
SELECT 
    method,
    COUNT(*) as total,
    SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as errors,
    ROUND(100.0 * SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) / COUNT(*), 2) as error_rate
FROM history_logs
GROUP BY method
ORDER BY error_rate DESC;
```

### Find Slow Endpoints

```sql
SELECT 
    method,
    AVG(duration_ms) as avg_ms,
    MAX(duration_ms) as max_ms,
    COUNT(*) as calls
FROM history_logs
GROUP BY method
ORDER BY avg_ms DESC;
```

### Get Hourly Activity

```sql
SELECT 
    DATE_FORMAT(timestamp, '%Y-%m-%d %H:00:00') as hour,
    COUNT(*) as calls,
    SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as errors
FROM history_logs
GROUP BY DATE_FORMAT(timestamp, '%Y-%m-%d %H:00:00')
ORDER BY hour DESC
LIMIT 24;
```

## 🧹 Maintenance

### Reset Logs (For Testing)

```bash
# This will delete ALL history
./scripts/reset-history-logs.sh
```

### Clean Up Old Logs (Production)

```bash
# Keep only last 30 days
mysql -u mecp_user -p mecp_db <<EOF
DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);
EOF
```

### Check Database Size

```bash
mysql -u mecp_user -p mecp_db <<EOF
SELECT 
    table_name,
    ROUND((data_length + index_length) / 1024 / 1024, 2) AS size_mb,
    table_rows
FROM information_schema.tables
WHERE table_schema = 'mecp_db' AND table_name = 'history_logs';
EOF
```

## ⚙️ Configuration

### Change Server Port

```bash
# Option 1: Environment variable
export MCP_PORT=8080
cargo run --release

# Option 2: Edit config.toml
[server]
port = 8080
```

### Change Database Credentials

Edit `config.toml`:
```toml
[mysql]
host = "localhost"
port = 3306
database = "mecp_db"
username = "your_user"
password = "your_password"
```

### Adjust Auto-Refresh Interval

Edit `dashboard/index.html`, line ~586:
```javascript
// Change 5000 (5 seconds) to desired milliseconds
autoRefreshInterval = setInterval(refreshData, 5000);
```

## 🐛 Troubleshooting

### Dashboard Shows "No data"

**Check MySQL:**
```bash
sudo systemctl status mysql
```

**Check table exists:**
```bash
mysql -u mecp_user -p mecp_db -e "SHOW TABLES;"
```

**If missing, reinitialize:**
```bash
./scripts/init-mysql-db.sh
```

### Server Won't Start

**Check port availability:**
```bash
sudo lsof -i :3000
```

**Use different port:**
```bash
MCP_PORT=3001 cargo run --release
```

### "Connection refused" in logs

**Verify MySQL is running:**
```bash
sudo systemctl start mysql
sudo systemctl enable mysql  # Auto-start on boot
```

**Test connection:**
```bash
mysql -u mecp_user -p mecp_db -e "SELECT 1;"
```

### Dashboard loads but no metrics

**Make some API calls first:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```

**Check server logs:**
Look for "Received MCP request" messages in console.

## 📝 Example Scripts

### Python Monitoring Script

```python
#!/usr/bin/env python3
import requests
import time

def check_health():
    stats = requests.get('http://127.0.0.1:3000/api/stats').json()
    
    print(f"📊 MeCP Health Check")
    print(f"   Total Calls: {stats['total_calls']}")
    print(f"   Success Rate: {stats['success_rate']:.1f}%")
    print(f"   Errors: {stats['total_errors']}")
    print(f"   Avg Response: {stats['avg_duration_ms']:.1f}ms")
    
    if stats['success_rate'] < 95:
        print(f"⚠️  WARNING: Success rate below 95%!")
    
    if stats['total_errors'] > 10:
        errors = requests.get('http://127.0.0.1:3000/api/errors').json()
        print(f"\n❌ Recent Errors:")
        for error in errors['errors'][:5]:
            print(f"   - {error['method']}: {error['error_message']}")

if __name__ == "__main__":
    while True:
        check_health()
        print()
        time.sleep(30)  # Check every 30 seconds
```

### Bash Alert Script

```bash
#!/bin/bash
# alert.sh - Alert if error rate is high

ERROR_THRESHOLD=5

stats=$(curl -s http://127.0.0.1:3000/api/stats)
errors=$(echo "$stats" | jq -r '.total_errors')

if [ "$errors" -gt "$ERROR_THRESHOLD" ]; then
    echo "🚨 ALERT: $errors errors detected!"
    curl -s http://127.0.0.1:3000/api/errors | jq -r '.errors[0] | "Latest: \(.method) - \(.error_message)"'
    # Add notification here (email, Slack, etc.)
fi
```

## 🎓 Next Steps

1. **Set up automated monitoring** - Use the Python script above
2. **Configure log rotation** - Add cron job to clean old logs
3. **Export metrics** - Integrate with Grafana or Prometheus
4. **Add alerts** - Notify on high error rates
5. **Analyze patterns** - Use SQL queries to find trends

## 📚 Additional Documentation

- **[DASHBOARD.md](DASHBOARD.md)** - Complete technical documentation
- **[DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)** - Quick reference
- **[DASHBOARD_IMPLEMENTATION_SUMMARY.md](DASHBOARD_IMPLEMENTATION_SUMMARY.md)** - Technical details

## 💡 Pro Tips

1. **Keep auto-refresh on** during debugging
2. **Check errors first** when something breaks
3. **Monitor response times** to catch performance issues
4. **Clean logs regularly** to maintain database performance
5. **Use the REST API** for automated monitoring
6. **Query the database** for historical analysis
7. **Set up alerts** for production environments

---

**Happy Monitoring!** 🚀

If you have questions, check the full documentation in `DASHBOARD.md`.
