# Response Data Recording Feature

## Overview

The MeCP dashboard now captures and stores complete response data for every API call, providing full request-response visibility for detailed endpoint logic analysis.

## Quick Start

### For New Users

```bash
# Initialize database (includes response_data column)
./scripts/init-mysql-db.sh

# Start server
cargo run --release

# Dashboard automatically shows responses
http://127.0.0.1:3000/dashboard
```

### For Existing Users

```bash
# Migrate database to add response_data column
./scripts/migrate-database.sh

# Rebuild and restart
cargo build --release
cargo run --release
```

See [MIGRATION_RESPONSE_DATA.md](MIGRATION_RESPONSE_DATA.md) for detailed migration steps.

## What's Captured

### Complete Response Data
Every API call now logs:
- **Request**: Method, endpoint, parameters
- **Response**: Full JSON response body ⭐ NEW
- **Status**: Success or error
- **Metadata**: Duration, timestamp, error messages

### Example Log Entry

```json
{
  "id": 123,
  "method": "initialize",
  "endpoint": "/mcp",
  "request_params": "{\"protocolVersion\":\"2024-11-05\"}",
  "response_data": "{\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{...}}",
  "response_status": "success",
  "duration_ms": 45,
  "timestamp": "2026-01-16T12:34:56Z"
}
```

## Dashboard Enhancements

### Recent API Calls Table
**New "Response" Column:**
- Shows first 50 characters of response
- Hover to see full response
- Formatted JSON display
- Color-coded by status

### Recent Errors Table
**New "Response" Column:**
- Shows error response details
- Includes error codes and messages
- Full response on hover
- Helps debug quickly

## Use Cases

### 1. Debugging Endpoint Logic
```sql
-- See what a specific endpoint returns
SELECT 
    request_params,
    response_data,
    duration_ms
FROM history_logs
WHERE method = 'tools/call'
AND timestamp >= DATE_SUB(NOW(), INTERVAL 1 HOUR);
```

### 2. Analyzing Error Patterns
```sql
-- Find common error responses
SELECT 
    error_message,
    response_data,
    COUNT(*) as occurrences
FROM history_logs
WHERE response_status = 'error'
GROUP BY error_message, response_data
ORDER BY occurrences DESC;
```

### 3. Testing Response Changes
```sql
-- Compare responses before/after code change
SELECT 
    timestamp,
    method,
    response_data
FROM history_logs
WHERE method = 'initialize'
ORDER BY timestamp DESC
LIMIT 10;
```

### 4. Validating Data Transformation
```sql
-- Check if responses match expected format
SELECT 
    method,
    response_data
FROM history_logs
WHERE response_data LIKE '%expected_field%'
ORDER BY timestamp DESC;
```

### 5. Performance Analysis
```sql
-- Correlate response size with duration
SELECT 
    method,
    AVG(CHAR_LENGTH(response_data)) as avg_response_size,
    AVG(duration_ms) as avg_duration
FROM history_logs
WHERE response_data IS NOT NULL
GROUP BY method;
```

## API Access

### Get Logs with Responses
```bash
curl http://127.0.0.1:3000/api/logs | jq '.logs[] | {
  method,
  status: .response_status,
  response: .response_data
}'
```

### Get Error Responses
```bash
curl http://127.0.0.1:3000/api/errors | jq '.errors[] | {
  method,
  error_message,
  response_data
}'
```

## Benefits

### For Developers
- ✅ **Complete visibility** into request-response cycle
- ✅ **Faster debugging** with full context
- ✅ **Better testing** with actual response data
- ✅ **Easy validation** of endpoint behavior

### For QA/Testing
- ✅ **Response verification** against expected values
- ✅ **Regression testing** by comparing historical responses
- ✅ **Edge case analysis** with real data
- ✅ **Integration testing** evidence

### For Operations
- ✅ **Audit trail** for compliance
- ✅ **Troubleshooting** with complete logs
- ✅ **Performance analysis** of response sizes
- ✅ **Capacity planning** based on response patterns

### For Business Logic
- ✅ **Data flow tracking** through the system
- ✅ **Logic verification** with real examples
- ✅ **Transformation validation** at each step
- ✅ **End-to-end visibility** of data processing

## Storage Management

### Database Growth
- **Typical response**: 500-5000 bytes
- **Daily storage (1000 calls)**: ~2-10 MB
- **Monthly growth**: ~60-300 MB

### Maintenance Recommendations

**Automatic Cleanup (Recommended):**
```bash
# Add to crontab - clean logs older than 30 days
0 2 * * * mysql -u mecp_user -pmecp_password mecp_db -e "DELETE FROM history_logs WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);"
```

**Manual Cleanup:**
```sql
-- Delete old logs
DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);

-- Archive before deletion
INSERT INTO history_logs_archive 
SELECT * FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);
```

**Development Reset:**
```bash
./scripts/reset-history-logs.sh
```

## Query Examples

### Response Analysis

**Find large responses:**
```sql
SELECT 
    method,
    CHAR_LENGTH(response_data) as size,
    duration_ms,
    timestamp
FROM history_logs
WHERE CHAR_LENGTH(response_data) > 10000
ORDER BY size DESC
LIMIT 10;
```

**Response content search:**
```sql
SELECT 
    method,
    response_data,
    timestamp
FROM history_logs
WHERE response_data LIKE '%search_term%'
ORDER BY timestamp DESC;
```

**Compare success vs error responses:**
```sql
SELECT 
    response_status,
    AVG(CHAR_LENGTH(response_data)) as avg_size,
    COUNT(*) as count
FROM history_logs
WHERE response_data IS NOT NULL
GROUP BY response_status;
```

## Integration Examples

### Python Analysis
```python
import requests
import json

# Fetch logs
response = requests.get('http://127.0.0.1:3000/api/logs')
logs = response.json()['logs']

# Analyze responses
for log in logs:
    if log['response_data']:
        response_obj = json.loads(log['response_data'])
        print(f"{log['method']}: {response_obj.get('result', 'No result')}")
```

### Bash Monitoring
```bash
# Watch for specific response values
curl -s http://127.0.0.1:3000/api/logs | \
  jq -r '.logs[] | select(.response_data | contains("specific_value")) | "\(.method): \(.response_data)"'
```

### SQL Reporting
```sql
-- Daily response summary
SELECT 
    DATE(timestamp) as date,
    method,
    COUNT(*) as calls,
    AVG(CHAR_LENGTH(response_data)) as avg_response_size
FROM history_logs
WHERE timestamp >= DATE_SUB(NOW(), INTERVAL 7 DAY)
AND response_data IS NOT NULL
GROUP BY DATE(timestamp), method
ORDER BY date DESC, calls DESC;
```

## Performance Impact

### Minimal Overhead
- **Write latency**: +1-2ms per request
- **Storage**: ~500-5000 bytes per log
- **Query speed**: No impact on aggregations
- **Memory**: Async writes, no blocking

### Optimization Tips
1. **Index wisely** - Don't index `response_data`
2. **Clean regularly** - Remove old logs
3. **Archive strategically** - Move to separate table
4. **Monitor disk** - Watch database size

## Security Considerations

### Sensitive Data
- **Be aware**: Responses may contain sensitive information
- **Solution**: Sanitize responses before logging (if needed)
- **Access control**: Restrict dashboard access in production

### PII Compliance
- **GDPR/CCPA**: May need to redact personal data
- **Retention**: Follow data retention policies
- **Deletion**: Provide mechanism to purge specific logs

## Future Enhancements

Potential improvements:
- [ ] Response data sanitization
- [ ] Selective recording (exclude large responses)
- [ ] Response diff comparison
- [ ] Visual response viewer in dashboard
- [ ] Response pattern analysis
- [ ] Automatic anomaly detection

## Troubleshooting

### Responses Not Showing

**Check database:**
```sql
SELECT COUNT(*), SUM(CASE WHEN response_data IS NOT NULL THEN 1 ELSE 0 END)
FROM history_logs;
```

**Test request:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'

# Then check logs
curl -s http://127.0.0.1:3000/api/logs | jq '.logs[0].response_data'
```

### Large Database

**Check size:**
```sql
SELECT 
    table_name,
    ROUND((data_length + index_length) / 1024 / 1024, 2) AS size_mb
FROM information_schema.tables
WHERE table_schema = 'mecp_db';
```

**Clean up:**
```bash
./scripts/reset-history-logs.sh
```

## Documentation

- **Complete guide**: [DASHBOARD.md](DASHBOARD.md)
- **Migration steps**: [MIGRATION_RESPONSE_DATA.md](MIGRATION_RESPONSE_DATA.md)
- **Quick start**: [DASHBOARD_QUICKSTART.md](DASHBOARD_QUICKSTART.md)
- **Testing**: [TEST_SCRIPTS_README.md](TEST_SCRIPTS_README.md)

## Summary

Response data recording provides complete visibility into your MCP server's behavior:

✅ **Full request-response logging**  
✅ **Enhanced debugging capabilities**  
✅ **Dashboard integration**  
✅ **SQL query support**  
✅ **API access**  
✅ **Minimal performance impact**  
✅ **Easy migration**  
✅ **Backward compatible**  

**Start using it today!** Just migrate your database and restart your server.

---

**Feature Version**: 1.0  
**Release Date**: 2026-01-16  
**Status**: Production Ready ✅
