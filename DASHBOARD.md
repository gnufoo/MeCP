# MeCP Monitoring Dashboard

## Overview

The MeCP Monitoring Dashboard provides real-time monitoring and analytics for your Model Context Protocol server. It tracks all API calls, displays performance metrics, and helps you debug issues efficiently.

## Features

- **Real-time Metrics**: Live tracking of all MCP interface calls
- **Performance Analytics**: Response times, success rates, and error tracking
- **History Logs**: Complete audit trail stored in MySQL database
- **Error Monitoring**: Quick access to recent errors with detailed information
- **Auto-refresh**: Automatic updates every 5 seconds (configurable)
- **Beautiful UI**: Modern, responsive design with intuitive visualizations

## Quick Start

### 1. Initialize the Database

First, set up the history_logs table in your MySQL database:

```bash
# Make sure MySQL is running
sudo systemctl start mysql

# Initialize the database schema
./scripts/init-mysql-db.sh
```

This creates:
- `history_logs` table for storing API call records
- Indexes for efficient querying
- A view for quick statistics

### 2. Start the MCP Server

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000` (default port).

### 3. Access the Dashboard

Open your browser and navigate to:

```
http://127.0.0.1:3000/dashboard
```

## Dashboard Components

### 1. Statistics Overview

The top section displays key metrics:

- **Total Calls**: Total number of API calls received
- **Success Rate**: Percentage of successful calls
- **Total Errors**: Number of failed requests
- **Avg Response Time**: Average duration of API calls

### 2. Endpoint Metrics Table

Shows detailed metrics for each endpoint:

- Method name (e.g., `initialize`, `tools/call`)
- Endpoint path
- Total calls, successful calls, and failed calls
- Average response duration
- Last called timestamp

### 3. Recent API Calls

Displays the most recent 100 API calls with:

- Timestamp
- Method name
- Endpoint
- Status (success/error)
- Duration

### 4. Recent Errors

Shows the last 50 errors with:

- Timestamp
- Method name
- Endpoint
- Error message
- Duration

## API Endpoints

The dashboard uses these REST API endpoints:

### GET /dashboard

Returns the HTML dashboard interface.

### GET /api/stats

Returns overall statistics.

**Response:**
```json
{
  "total_calls": 150,
  "total_errors": 5,
  "success_rate": 96.67,
  "avg_duration_ms": 45.3,
  "endpoints_count": 7,
  "recent_logs_count": 100,
  "timestamp": "2026-01-16T12:34:56Z"
}
```

### GET /api/metrics

Returns aggregated metrics per endpoint.

**Response:**
```json
{
  "metrics": [
    {
      "method": "initialize",
      "endpoint": "/mcp",
      "total_calls": 50,
      "successful_calls": 49,
      "failed_calls": 1,
      "avg_duration_ms": 42.5,
      "last_called": "2026-01-16T12:34:56Z"
    }
  ],
  "timestamp": "2026-01-16T12:34:56Z"
}
```

### GET /api/logs

Returns recent API call logs (last 100).

**Response:**
```json
{
  "logs": [
    {
      "id": 123,
      "method": "tools/call",
      "endpoint": "/mcp",
      "request_params": "{\"name\":\"hello_world\"}",
      "response_status": "success",
      "error_message": null,
      "duration_ms": 35,
      "timestamp": "2026-01-16T12:34:56Z",
      "client_info": null
    }
  ],
  "count": 100,
  "timestamp": "2026-01-16T12:34:56Z"
}
```

### GET /api/errors

Returns recent error logs (last 50 errors).

**Response:**
```json
{
  "errors": [
    {
      "id": 124,
      "method": "tools/call",
      "endpoint": "/mcp",
      "request_params": "{\"name\":\"unknown_tool\"}",
      "response_status": "error",
      "error_message": "Tool not found: unknown_tool",
      "duration_ms": 15,
      "timestamp": "2026-01-16T12:35:00Z",
      "client_info": null
    }
  ],
  "count": 1,
  "timestamp": "2026-01-16T12:35:01Z"
}
```

## Database Schema

### history_logs Table

```sql
CREATE TABLE history_logs (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    method VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    request_params TEXT,
    response_status VARCHAR(20) NOT NULL,
    error_message TEXT,
    duration_ms BIGINT UNSIGNED NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    client_info VARCHAR(255),
    
    INDEX idx_method (method),
    INDEX idx_endpoint (endpoint),
    INDEX idx_response_status (response_status),
    INDEX idx_timestamp (timestamp),
    INDEX idx_method_endpoint (method, endpoint)
);
```

### Fields

- **id**: Unique identifier for each log entry
- **method**: MCP method name (e.g., `initialize`, `tools/call`)
- **endpoint**: HTTP endpoint path (e.g., `/mcp`)
- **request_params**: JSON-encoded request parameters (optional)
- **response_status**: Either `success` or `error`
- **error_message**: Error description if status is `error` (optional)
- **duration_ms**: Response time in milliseconds
- **timestamp**: When the request was received
- **client_info**: Client identification (optional)

## Maintenance

### Reset History Logs

To clear all history logs (useful for testing):

```bash
./scripts/reset-history-logs.sh
```

**Warning**: This permanently deletes all API call history!

### Query History Logs Manually

Connect to MySQL:

```bash
mysql -u mecp_user -p mecp_db
# Password: mecp_password
```

View recent logs:

```sql
SELECT * FROM history_logs 
ORDER BY timestamp DESC 
LIMIT 10;
```

View error rate by method:

```sql
SELECT 
    method,
    COUNT(*) as total,
    SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as errors,
    ROUND(SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) * 100.0 / COUNT(*), 2) as error_rate
FROM history_logs
GROUP BY method
ORDER BY error_rate DESC;
```

View slowest endpoints:

```sql
SELECT 
    method,
    endpoint,
    AVG(duration_ms) as avg_duration,
    MAX(duration_ms) as max_duration,
    COUNT(*) as call_count
FROM history_logs
GROUP BY method, endpoint
ORDER BY avg_duration DESC
LIMIT 10;
```

## Configuration

### Change Server Port

Edit `config.toml`:

```toml
[server]
port = 3000  # Change to desired port
```

Or set environment variable:

```bash
export MCP_PORT=3000
cargo run
```

### Auto-refresh Interval

The dashboard auto-refreshes every 5 seconds by default. To change this:

1. Open `dashboard/index.html`
2. Find the line: `autoRefreshInterval = setInterval(refreshData, 5000);`
3. Change `5000` (milliseconds) to your desired interval

### Memory Limits

The metrics collector keeps the last 1000 entries in memory. To change this:

1. Edit `src/core/metrics.rs`
2. Find: `if logs.len() > 1000`
3. Change `1000` to your desired limit

## Troubleshooting

### Dashboard Shows No Data

**Check if the database table exists:**

```bash
mysql -u mecp_user -p mecp_db -e "SHOW TABLES;"
```

If `history_logs` is missing, run:

```bash
./scripts/init-mysql-db.sh
```

### Database Connection Errors

**Verify MySQL is running:**

```bash
sudo systemctl status mysql
```

**Check database credentials in `config.toml`:**

```toml
[mysql]
host = "localhost"
port = 3306
database = "mecp_db"
username = "mecp_user"
password = "mecp_password"
```

### Dashboard Not Loading

**Check server is running:**

```bash
# Look for "MCP HTTP Server starting on 127.0.0.1:3000"
# and "Dashboard available at http://127.0.0.1:3000/dashboard"
```

**Check firewall:**

```bash
# Allow port 3000
sudo ufw allow 3000
```

### High Memory Usage

If memory usage is high, reduce the in-memory log limit:

1. Edit `src/core/metrics.rs`
2. Change `if logs.len() > 1000` to a smaller value (e.g., 500)
3. Rebuild: `cargo build --release`

## Performance Considerations

### Database Growth

The `history_logs` table will grow over time. To manage this:

**Option 1: Periodic cleanup (recommended)**

```sql
-- Delete logs older than 30 days
DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);
```

**Option 2: Archive old logs**

```sql
-- Create archive table
CREATE TABLE history_logs_archive LIKE history_logs;

-- Move old logs to archive
INSERT INTO history_logs_archive 
SELECT * FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);

DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);
```

**Option 3: Add to cron job**

```bash
# Edit crontab
crontab -e

# Add daily cleanup (runs at 2 AM)
0 2 * * * mysql -u mecp_user -pmecp_password mecp_db -e "DELETE FROM history_logs WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);"
```

### Indexing

The table comes with pre-configured indexes for optimal query performance:

- `idx_method`: For filtering by method
- `idx_endpoint`: For filtering by endpoint
- `idx_response_status`: For filtering errors
- `idx_timestamp`: For time-based queries
- `idx_method_endpoint`: For combined queries

### Monitoring Performance

Check query performance:

```sql
EXPLAIN SELECT * FROM history_logs 
WHERE method = 'initialize' 
ORDER BY timestamp DESC 
LIMIT 100;
```

## Integration

### Programmatic Access

You can access the dashboard API programmatically:

**Python Example:**

```python
import requests

# Get stats
response = requests.get('http://127.0.0.1:3000/api/stats')
stats = response.json()
print(f"Total calls: {stats['total_calls']}")
print(f"Success rate: {stats['success_rate']}%")

# Get recent errors
response = requests.get('http://127.0.0.1:3000/api/errors')
errors = response.json()
for error in errors['errors']:
    print(f"Error: {error['method']} - {error['error_message']}")
```

**curl Example:**

```bash
# Get stats
curl http://127.0.0.1:3000/api/stats | jq .

# Get metrics
curl http://127.0.0.1:3000/api/metrics | jq .

# Get errors
curl http://127.0.0.1:3000/api/errors | jq '.errors[] | {method, error_message, timestamp}'
```

## Testing

Run the dashboard tests:

```bash
cargo test --test dashboard_test
```

Expected output:

```
running 7 tests
test test_metrics_collector_record_call ... ok
test test_metrics_collector_multiple_calls ... ok
test test_endpoint_metrics_aggregation ... ok
test test_metrics_collector_memory_limit ... ok
test test_endpoint_metrics_multiple_methods ... ok
test test_api_call_log_serialization ... ok
test test_metrics_with_errors ... ok

test result: ok. 7 passed; 0 failed; 0 ignored
```

## Security Considerations

### Production Deployment

For production environments:

1. **Change default passwords**:
   - Edit MySQL credentials in `config.toml`
   - Use strong passwords

2. **Restrict database access**:
   ```sql
   -- Allow connections only from localhost
   CREATE USER 'mecp_user'@'localhost' IDENTIFIED BY 'strong_password';
   ```

3. **Enable HTTPS**:
   - Use a reverse proxy (nginx, Apache)
   - Configure SSL certificates

4. **Add authentication**:
   - Consider adding basic auth or OAuth to the dashboard
   - Restrict access by IP address

5. **Firewall rules**:
   ```bash
   # Only allow from specific IP
   sudo ufw allow from YOUR_IP to any port 3000
   ```

## Support

For issues or questions:

1. Check the logs: Look for error messages in the console
2. Verify database connectivity: `mysql -u mecp_user -p mecp_db`
3. Review configuration: Ensure `config.toml` is correct
4. Run tests: `cargo test --test dashboard_test`
5. Check MySQL logs: `sudo journalctl -u mysql -n 50`

## Changelog

### Version 0.1.0 (Initial Release)

- Real-time metrics tracking
- In-memory and MySQL-backed storage
- Beautiful web dashboard
- REST API endpoints
- Comprehensive test suite
- Auto-refresh functionality
- Error monitoring
- Performance analytics
