# MeCP Dashboard Test Scripts

## Overview

These scripts help you test the complete monitoring flow:
**Client → MeCP Server → Database → Dashboard**

Two options are provided:
1. **Bash Script** (`test-dashboard-flow.sh`) - Quick and simple
2. **Rust Client** (`test_client.rs`) - More sophisticated and realistic

## 🚀 Quick Start

### Option 1: Bash Script (Recommended for Quick Tests)

```bash
# Send 20 requests (default)
./scripts/test-dashboard-flow.sh

# Send 50 requests
./scripts/test-dashboard-flow.sh 50

# Send 100 requests
./scripts/test-dashboard-flow.sh 100
```

### Option 2: Rust Client (More Realistic)

```bash
# Send 50 requests (default)
cargo run --example test_client

# Send 100 requests
cargo run --example test_client -- 100

# Use custom server URL
MCP_URL=http://localhost:3000 cargo run --example test_client
```

## 📋 Prerequisites

Before running the tests, ensure:

1. **MySQL is running:**
   ```bash
   sudo systemctl start mysql
   ```

2. **Database is initialized:**
   ```bash
   ./scripts/init-mysql-db.sh
   ```

3. **MeCP server is running:**
   ```bash
   cargo run --release
   ```

4. **Required tools installed (for bash script):**
   ```bash
   sudo apt-get install curl jq mysql-client
   ```

## 🧪 What the Tests Do

Both scripts test all MCP endpoints:

### 1. **Initialize** (`initialize`)
- Sends client initialization requests
- Tests: Protocol version negotiation

### 2. **Resources List** (`resources/list`)
- Lists available resources
- Tests: Resource discovery

### 3. **Resources Read** (`resources/read`)
- Reads specific resources
- Tests: Resource retrieval

### 4. **Tools List** (`tools/list`)
- Lists available tools
- Tests: Tool discovery

### 5. **Tools Call (Success)** (`tools/call`)
- Calls valid tools (hello_world)
- Tests: Successful tool execution

### 6. **Tools Call (Errors)** (`tools/call`)
- Attempts to call nonexistent tools
- Tests: Error handling and tracking

### 7. **Prompts List** (`prompts/list`)
- Lists available prompts
- Tests: Prompt discovery

### 8. **Prompts Get** (`prompts/get`)
- Generates prompts with various topics
- Tests: Prompt generation

## 📊 What Gets Verified

### During Test Run

✅ Server is running and responding  
✅ MySQL database is accessible  
✅ history_logs table exists  
✅ Requests are being processed  
✅ Success/error tracking works  

### After Test Run

✅ Metrics are written to database  
✅ API endpoints return correct data  
✅ Statistics are calculated correctly  
✅ Error logs are captured  
✅ Dashboard displays the data  

## 🎯 Expected Output

### Bash Script Output

```
╔════════════════════════════════════════════════════╗
║   MeCP Dashboard Flow Test Script                 ║
╚════════════════════════════════════════════════════╝

Target: http://127.0.0.1:3000
Requests: 20

Checking if MeCP server is running... ✓ Server is running
Checking if MySQL is running... ✓ MySQL is running
Checking if history_logs table exists... ✓ Table exists

════════════════════════════════════════════════════
Starting test requests...
════════════════════════════════════════════════════

[1/7] Testing initialize endpoint...
  → Initialize request #1... Success
  → Initialize request #2... Success
  ...

[2/7] Testing resources/list endpoint...
  → List resources #1... Success
  ...

[6/7] Testing tools/call endpoint (errors)...
  → Call nonexistent tool: unknown_tool... Expected Error
  ...

════════════════════════════════════════════════════
Test Results
════════════════════════════════════════════════════

Successful requests: 16
Error requests: 4
Total requests: 20

════════════════════════════════════════════════════
Database Verification
════════════════════════════════════════════════════

Total logs in database: 142

Recent logs:
+----------+----------------+-----------------+-------------+-------+
| time     | method         | response_status | duration_ms | error |
+----------+----------------+-----------------+-------------+-------+
| 16:45:23 | prompts/get    | success         |          42 | -     |
| 16:45:22 | tools/call     | error           |          15 | ...   |
...

Metrics by endpoint:
+----------------+-------+---------+--------+--------+
| method         | total | success | errors | avg_ms |
+----------------+-------+---------+--------+--------+
| initialize     |    25 |      25 |      0 |  45.30 |
| tools/call     |    20 |      15 |      5 |  38.50 |
...

════════════════════════════════════════════════════
API Endpoints Verification
════════════════════════════════════════════════════

Testing /api/stats... ✓ Working
{
  "total_calls": 142,
  "total_errors": 12,
  "success_rate": 91.5,
  "avg_duration_ms": 42.3
}

Testing /api/metrics... ✓ Working
  Endpoints tracked: 7

Testing /api/logs... ✓ Working
  Recent logs count: 100

Testing /api/errors... ✓ Working
  Error logs count: 12

════════════════════════════════════════════════════
Summary
════════════════════════════════════════════════════

✓ Flow Test Complete!

Next steps:
  1. Open dashboard: http://127.0.0.1:3000/dashboard
  2. Verify metrics are displayed correctly
  3. Check that errors are shown in the errors section
  4. Confirm auto-refresh is working

Dashboard URL: http://127.0.0.1:3000/dashboard
```

### Rust Client Output

```
╔════════════════════════════════════════════════════╗
║   MeCP Test Client - Dashboard Flow Tester        ║
╚════════════════════════════════════════════════════╝

🎯 Target: http://127.0.0.1:3000
📊 Requests: 50

🔍 Checking server health... ✓ Server is running

════════════════════════════════════════════════════
Starting test requests...
════════════════════════════════════════════════════

🔧 [1/8] Testing initialize endpoint...
...... Done
📦 [2/8] Testing resources/list endpoint...
...... Done
📖 [3/8] Testing resources/read endpoint...
...... Done
🔨 [4/8] Testing tools/list endpoint...
...... Done
✅ [5/8] Testing tools/call endpoint (success)...
...... Done
❌ [6/8] Testing tools/call endpoint (errors)...
EEEEEE Done
💬 [7/8] Testing prompts/list endpoint...
...... Done
📝 [8/8] Testing prompts/get endpoint...
...... Done

⏳ Waiting for metrics to be written to database...

════════════════════════════════════════════════════
Test Results
════════════════════════════════════════════════════

  Total requests:      50
  ✓ Successful:        44
  ✗ Failed:            6
  Success rate:        88.0%

📊 Verifying Dashboard API Endpoints...

  → Testing /api/stats... ✓
     Total calls: 192
     Success rate: 89.6%
     Total errors: 20
  → Testing /api/metrics... ✓
     Endpoints tracked: 7
  → Testing /api/logs... ✓
     Recent logs: 100
  → Testing /api/errors... ✓
     Error logs: 20

════════════════════════════════════════════════════
✅ Flow Test Complete!
════════════════════════════════════════════════════

Next steps:
  1. Open dashboard: http://127.0.0.1:3000/dashboard
  2. Verify metrics are displayed correctly
  3. Check that errors are shown in the errors section
  4. Confirm auto-refresh is working

🌐 Dashboard URL: http://127.0.0.1:3000/dashboard
```

## 🔍 Verification Steps

After running the tests:

### 1. Check Dashboard

Open `http://127.0.0.1:3000/dashboard`

Verify:
- ✅ Total calls increased
- ✅ Success rate is calculated (should be ~85-90%)
- ✅ Errors are displayed in "Recent Errors"
- ✅ All endpoints show in "Endpoint Metrics"
- ✅ Recent logs show the test requests
- ✅ Auto-refresh is working (watch the timestamp)

### 2. Check Database

```bash
mysql -u mecp_user -p mecp_db
# Password: mecp_password

# View recent logs
SELECT * FROM history_logs ORDER BY timestamp DESC LIMIT 10;

# Check metrics
SELECT method, COUNT(*) as total 
FROM history_logs 
GROUP BY method;
```

### 3. Test API Endpoints

```bash
# Get stats
curl http://127.0.0.1:3000/api/stats | jq .

# Get metrics
curl http://127.0.0.1:3000/api/metrics | jq .

# Get errors
curl http://127.0.0.1:3000/api/errors | jq '.errors[] | {method, error_message}'
```

## 🎛️ Configuration

### Bash Script

**Environment Variables:**
- `MCP_HOST` - Server hostname (default: 127.0.0.1)
- `MCP_PORT` - Server port (default: 3000)

**Usage:**
```bash
# Custom host/port
MCP_HOST=localhost MCP_PORT=8080 ./scripts/test-dashboard-flow.sh

# Custom number of requests
./scripts/test-dashboard-flow.sh 100
```

### Rust Client

**Environment Variables:**
- `MCP_URL` - Full server URL (default: http://127.0.0.1:3000)

**Usage:**
```bash
# Custom URL
MCP_URL=http://localhost:8080 cargo run --example test_client

# Custom number of requests
cargo run --example test_client -- 200
```

## 🐛 Troubleshooting

### "Server not responding"

**Problem:** Script can't connect to MeCP server

**Solution:**
```bash
# Start the server
cargo run --release

# Check it's running
curl http://127.0.0.1:3000/health
```

### "MySQL is not running"

**Problem:** MySQL service is down

**Solution:**
```bash
sudo systemctl start mysql
sudo systemctl enable mysql  # Auto-start on boot
```

### "Table not found"

**Problem:** history_logs table doesn't exist

**Solution:**
```bash
./scripts/init-mysql-db.sh
```

### "Connection refused" to MySQL

**Problem:** Database credentials incorrect

**Solution:**
```bash
# Reset MySQL setup
sudo mysql <<EOF
CREATE DATABASE IF NOT EXISTS mecp_db;
CREATE USER IF NOT EXISTS 'mecp_user'@'localhost' IDENTIFIED BY 'mecp_password';
GRANT ALL PRIVILEGES ON mecp_db.* TO 'mecp_user'@'localhost';
FLUSH PRIVILEGES;
EOF

# Reinitialize
./scripts/init-mysql-db.sh
```

### "jq: command not found" (bash script)

**Problem:** jq not installed

**Solution:**
```bash
sudo apt-get install jq
```

### Port already in use

**Problem:** Port 3000 is occupied

**Solution:**
```bash
# Use different port
MCP_PORT=3001 cargo run --release

# Then test with:
MCP_PORT=3001 ./scripts/test-dashboard-flow.sh
```

## 📈 Performance Testing

### Load Test

Generate sustained load:

```bash
# Bash script - Multiple rounds
for i in {1..10}; do
    echo "Round $i"
    ./scripts/test-dashboard-flow.sh 100
    sleep 5
done

# Rust client - High volume
cargo run --release --example test_client -- 1000
```

### Concurrent Clients

Test concurrent access:

```bash
# Run 5 clients in parallel
for i in {1..5}; do
    cargo run --release --example test_client -- 100 &
done
wait

echo "All clients finished"
```

## 🧹 Cleanup

After testing, you may want to clean up:

```bash
# Reset history logs
./scripts/reset-history-logs.sh

# Or manually
mysql -u mecp_user -pmecp_password mecp_db -e "TRUNCATE TABLE history_logs;"

# Verify clean state
curl http://127.0.0.1:3000/api/stats | jq .
```

## 📝 Example Workflow

Complete test workflow:

```bash
# 1. Start fresh
./scripts/reset-history-logs.sh

# 2. Start server
cargo run --release &
SERVER_PID=$!

# 3. Wait for server to start
sleep 2

# 4. Run tests
./scripts/test-dashboard-flow.sh 50

# 5. Check dashboard
echo "Open: http://127.0.0.1:3000/dashboard"

# 6. View results in database
mysql -u mecp_user -pmecp_password mecp_db <<EOF
SELECT method, COUNT(*) as calls, 
       SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as errors
FROM history_logs 
GROUP BY method;
EOF

# 7. Cleanup
kill $SERVER_PID
```

## 🎓 Advanced Usage

### Custom Request Patterns

Modify `test_client.rs` to add custom patterns:

```rust
// Add burst traffic
for _ in 0..100 {
    test_tools_call(&client, &base_url, "TestUser").await?;
}
sleep(Duration::from_secs(10)).await;

// Add gradual ramp-up
for i in 1..50 {
    test_initialize(&client, &base_url, i).await?;
    sleep(Duration::from_millis(100 * i as u64)).await;
}
```

### Integration with CI/CD

```yaml
# .github/workflows/test.yml
- name: Test Dashboard Flow
  run: |
    cargo run --release &
    sleep 5
    cargo run --example test_client -- 100
    
    # Verify results
    STATS=$(curl -s http://127.0.0.1:3000/api/stats)
    SUCCESS_RATE=$(echo $STATS | jq '.success_rate')
    
    if (( $(echo "$SUCCESS_RATE < 90" | bc -l) )); then
      echo "❌ Success rate too low: $SUCCESS_RATE%"
      exit 1
    fi
```

## 📚 Related Documentation

- [DASHBOARD.md](DASHBOARD.md) - Complete dashboard documentation
- [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md) - Quick start guide
- [DASHBOARD_USAGE_GUIDE.md](DASHBOARD_USAGE_GUIDE.md) - Usage guide

## 💡 Tips

1. **Run tests regularly** during development
2. **Check dashboard** after each test run
3. **Monitor error rates** - should stay under 10-15%
4. **Use Rust client** for realistic load testing
5. **Use bash script** for quick verification
6. **Clean logs** between test runs for clarity
7. **Automate tests** in your CI/CD pipeline

---

**Happy Testing!** 🚀
