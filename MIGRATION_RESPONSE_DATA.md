# Migration Guide: Adding Response Data Recording

## Overview

This migration adds response data recording to the MeCP monitoring system. The `response_data` field captures the full response returned by each API call, enabling detailed endpoint logic analysis.

## What's New

### Database Schema
- **New Column**: `response_data TEXT` in `history_logs` table
- **Position**: After `request_params` column
- **Purpose**: Stores the complete JSON response from each API call

### Code Changes
- **ApiCallLog struct**: Added `response_data: Option<String>` field
- **HTTP handlers**: Now capture and serialize responses
- **Dashboard UI**: Displays response previews in logs and errors tables

## Migration Steps

### For New Installations

If you're setting up MeCP for the first time, the `response_data` column will be automatically created:

```bash
./scripts/init-mysql-db.sh
```

No additional steps needed!

### For Existing Installations

If you have an existing `history_logs` table, you need to add the `response_data` column:

#### Option 1: Automatic Migration (Recommended)

```bash
./scripts/migrate-database.sh
```

This script:
- ✅ Checks if the column already exists
- ✅ Adds the column if missing
- ✅ Shows the updated schema
- ✅ Safe to run multiple times

#### Option 2: Manual Migration

```bash
mysql -u mecp_user -p mecp_db < scripts/migrate_add_response_data.sql
```

#### Option 3: SQL Command

```sql
-- Connect to database
mysql -u mecp_user -p mecp_db

-- Add column
ALTER TABLE history_logs 
ADD COLUMN response_data TEXT AFTER request_params;

-- Verify
DESCRIBE history_logs;
```

## Verification

After migration, verify the column exists:

```bash
mysql -u mecp_user -p mecp_db -e "DESCRIBE history_logs;"
```

Expected output should include:

```
+-----------------+---------------------+------+-----+-------------------+
| Field           | Type                | Null | Key | Default           |
+-----------------+---------------------+------+-----+-------------------+
| id              | bigint              | NO   | PRI | NULL              |
| method          | varchar(50)         | NO   | MUL | NULL              |
| endpoint        | varchar(255)        | NO   | MUL | NULL              |
| request_params  | text                | YES  |     | NULL              |
| response_data   | text                | YES  |     | NULL              | <-- NEW
| response_status | varchar(20)         | NO   | MUL | NULL              |
| error_message   | text                | YES  |     | NULL              |
| duration_ms     | bigint unsigned     | NO   |     | NULL              |
| timestamp       | timestamp           | NO   | MUL | CURRENT_TIMESTAMP |
| client_info     | varchar(255)        | YES  |     | NULL              |
+-----------------+---------------------+------+-----+-------------------+
```

## What Gets Recorded

The `response_data` field now captures:

### Successful Responses
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2024-11-05",
    "capabilities": {...},
    "serverInfo": {...}
  }
}
```

### Error Responses
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found: unknown_method"
  }
}
```

## Dashboard Changes

The dashboard now displays response data:

### Recent API Calls Table
- **New Column**: "Response"
- **Shows**: Truncated response (first 50 characters)
- **Tooltip**: Hover to see full response

### Recent Errors Table
- **New Column**: "Response"
- **Shows**: Truncated error response (first 40 characters)
- **Tooltip**: Hover to see full response

## Using Response Data

### Query Examples

**View responses for a specific method:**
```sql
SELECT 
    timestamp,
    method,
    response_status,
    response_data
FROM history_logs
WHERE method = 'initialize'
ORDER BY timestamp DESC
LIMIT 10;
```

**Analyze error responses:**
```sql
SELECT 
    method,
    error_message,
    response_data
FROM history_logs
WHERE response_status = 'error'
AND timestamp >= DATE_SUB(NOW(), INTERVAL 1 HOUR)
ORDER BY timestamp DESC;
```

**Compare request and response:**
```sql
SELECT 
    method,
    request_params,
    response_data,
    duration_ms
FROM history_logs
WHERE method = 'tools/call'
ORDER BY timestamp DESC
LIMIT 5;
```

**Find responses matching a pattern:**
```sql
SELECT 
    timestamp,
    method,
    response_data
FROM history_logs
WHERE response_data LIKE '%specific_value%'
ORDER BY timestamp DESC;
```

## Storage Considerations

### Disk Space

Response data can increase database size significantly:

- **Average response size**: 500-5000 bytes
- **With 1000 logs/day**: ~2-10 MB/day
- **Monthly growth**: ~60-300 MB/month

### Recommendations

**For production:**
```sql
-- Clean old logs periodically
DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 30 DAY);

-- Or archive old data
INSERT INTO history_logs_archive 
SELECT * FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);

DELETE FROM history_logs 
WHERE timestamp < DATE_SUB(NOW(), INTERVAL 90 DAY);
```

**For development:**
```bash
# Reset frequently
./scripts/reset-history-logs.sh
```

## Compatibility

### Backward Compatibility
- ✅ Existing logs without `response_data` remain valid (NULL values)
- ✅ Old queries continue to work
- ✅ API endpoints unchanged
- ✅ Dashboard gracefully handles missing data

### Forward Compatibility
- ✅ All new logs include response data
- ✅ Response data is optional (can be NULL)
- ✅ Dashboard shows "-" for missing responses

## Rollback

If you need to remove the column:

```sql
-- WARNING: This will delete all response data!
ALTER TABLE history_logs DROP COLUMN response_data;
```

**Note**: You'll need to revert code changes as well to avoid compilation errors.

## Benefits

### Enhanced Debugging
- See exactly what was returned
- Compare expected vs actual responses
- Identify response format issues

### Endpoint Logic Analysis
- Analyze response patterns
- Track data transformations
- Verify business logic

### Compliance & Audit
- Complete request-response cycle logged
- Full audit trail for debugging
- Evidence for troubleshooting

### Testing & Validation
- Verify endpoint behavior
- Validate response formats
- Compare across versions

## Performance Impact

### Write Performance
- **Overhead**: ~1-2ms per request
- **Reason**: JSON serialization
- **Impact**: Minimal (async operation)

### Storage Performance
- **Indexes**: No new indexes needed
- **Query speed**: Unchanged
- **Disk I/O**: Slightly increased

### Query Performance
- **Select queries**: Add `response_data` only when needed
- **Aggregations**: No impact (column not used)
- **Full-text search**: Consider indexing if needed

## Troubleshooting

### Migration Fails

**Error: "Column already exists"**
```bash
# Safe to ignore - column already added
# Verify with:
mysql -u mecp_user -p mecp_db -e "DESCRIBE history_logs;"
```

**Error: "Access denied"**
```bash
# Check user permissions
mysql -u mecp_user -p mecp_db -e "SHOW GRANTS;"
```

### Compilation Errors

**Error: "missing field `response_data`"**
```bash
# You need to update your code
# Pull latest changes or add the field manually
```

### Dashboard Not Showing Responses

**Check logs have data:**
```sql
SELECT COUNT(*), 
       SUM(CASE WHEN response_data IS NOT NULL THEN 1 ELSE 0 END) as with_response
FROM history_logs;
```

**Clear browser cache:**
```bash
# Hard refresh: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)
```

## Testing After Migration

### 1. Send Test Requests

```bash
cargo run --example test_client -- 10
```

### 2. Verify Data

```sql
SELECT 
    id,
    method,
    CHAR_LENGTH(response_data) as response_size,
    response_status
FROM history_logs
ORDER BY id DESC
LIMIT 5;
```

### 3. Check Dashboard

```bash
# Open dashboard
http://127.0.0.1:3000/dashboard

# Verify:
# - Response column appears in Recent API Calls
# - Response column appears in Recent Errors
# - Hover shows full response
```

### 4. API Verification

```bash
# Check response data is included
curl -s http://127.0.0.1:3000/api/logs | jq '.logs[0] | {method, response_data}'
```

## Next Steps

After successful migration:

1. **Test thoroughly** - Send various requests
2. **Monitor disk space** - Watch database growth
3. **Set up cleanup** - Schedule periodic maintenance
4. **Update monitoring** - Adjust alerts if needed
5. **Document usage** - Share query examples with team

## Support

If you encounter issues:

1. Check [DASHBOARD.md](DASHBOARD.md) for general documentation
2. Review [TROUBLESHOOTING.md](TROUBLESHOOTING.md) if available
3. Verify database connection and permissions
4. Check server logs for errors
5. Test with fresh database if needed

---

**Migration Version**: 1.0  
**Date**: 2026-01-16  
**Compatibility**: MeCP v0.1.0+
