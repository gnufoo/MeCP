# MeCP Dashboard Quick Start

## 🚀 Get Started in 3 Steps

### Step 1: Initialize Database

```bash
# Start MySQL
sudo systemctl start mysql

# Initialize the database
./scripts/init-mysql-db.sh
```

### Step 2: Start the Server

```bash
# Build and run
cargo run --release

# Or use the example
cargo run --release --example dashboard_usage
```

### Step 3: Open Dashboard

Open your browser:
```
http://127.0.0.1:3000/dashboard
```

## ✨ Features at a Glance

- ✅ **Real-time Metrics** - Live tracking of all API calls
- 📊 **Performance Analytics** - Response times and success rates
- 🔍 **Error Monitoring** - Quick access to errors with details
- 💾 **Persistent Storage** - MySQL backend for history
- 🔄 **Auto-refresh** - Updates every 5 seconds
- 🎨 **Beautiful UI** - Modern, responsive design

## 📡 API Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /dashboard` | Web dashboard interface |
| `GET /api/stats` | Overall statistics |
| `GET /api/metrics` | Per-endpoint metrics |
| `GET /api/logs` | Recent API calls (100) |
| `GET /api/errors` | Recent errors (50) |

## 🧪 Test It

```bash
# Run tests
cargo test --test dashboard_test

# Make some API calls
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

# Check dashboard for the call
```

## 🛠️ Maintenance

### Reset Logs
```bash
./scripts/reset-history-logs.sh
```

### Query Manually
```bash
mysql -u mecp_user -p mecp_db
# Password: mecp_password

SELECT * FROM history_logs ORDER BY timestamp DESC LIMIT 10;
```

### Cleanup Old Logs
```sql
-- Delete logs older than 30 days
DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);
```

## 🔧 Configuration

### Change Port

Edit `config.toml`:
```toml
[server]
port = 3000
```

Or set environment variable:
```bash
export MCP_PORT=3000
```

### Database Settings

Edit `config.toml`:
```toml
[mysql]
host = "localhost"
port = 3306
database = "mecp_db"
username = "mecp_user"
password = "mecp_password"
```

## 📚 More Information

For detailed documentation, see [DASHBOARD.md](DASHBOARD.md)

## 🐛 Troubleshooting

### Dashboard shows no data?
```bash
# Check if table exists
mysql -u mecp_user -p mecp_db -e "SHOW TABLES;"

# If missing, reinitialize
./scripts/init-mysql-db.sh
```

### Can't connect to MySQL?
```bash
# Check MySQL is running
sudo systemctl status mysql

# Start if not running
sudo systemctl start mysql
```

### Server won't start?
```bash
# Check if port is in use
sudo lsof -i :3000

# Use different port
MCP_PORT=3001 cargo run
```

## 💡 Pro Tips

1. **Keep auto-refresh enabled** for real-time monitoring
2. **Check errors regularly** to catch issues early
3. **Monitor response times** to identify slow endpoints
4. **Clean up old logs** periodically to save space
5. **Use the API** for automated monitoring and alerts

## 🎯 What's Next?

- Set up automated log cleanup (cron job)
- Add custom alerts for error thresholds
- Export metrics for external monitoring tools
- Create dashboards in Grafana or similar tools

Happy Monitoring! 🚀
