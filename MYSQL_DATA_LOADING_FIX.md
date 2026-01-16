# MySQL Data Loading Fix

## Problem

After restarting the MeCP server, the dashboard showed **empty data** even though MySQL database contained historical records. The issue was that the dashboard API endpoints were only reading from **in-memory** storage, which gets cleared on every restart.

## Root Cause

The `MetricsCollector` had two separate data access paths:

### Before Fix

```
┌─────────────┐
│   Request   │
│   Arrives   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  MetricsCollector│
│   record_call()  │
└──────┬──────────┘
       │
       ├──────────────────────┐
       │                      │
       ▼                      ▼
┌─────────────┐      ┌──────────────┐
│  In-Memory  │      │    MySQL     │
│   Storage   │      │   Database   │
└──────┬──────┘      └──────────────┘
       │                      │
       │ ✅ Write to both     │
       │                      │
       │                      │
       ▼                      ▼
┌─────────────┐      ┌──────────────┐
│  Dashboard  │      │    (unused)  │
│  APIs read  │      │              │
│  from here  │      │              │
│     ONLY    │      │              │
└─────────────┘      └──────────────┘
     ❌                    ❌
 Empty on restart    Never read from
```

## The Fix

Modified `MetricsCollector` methods to **prioritize MySQL** over in-memory storage:

### After Fix

```
┌─────────────┐
│   Request   │
│   Arrives   │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  MetricsCollector│
│   record_call()  │
└──────┬──────────┘
       │
       ├──────────────────────┐
       │                      │
       ▼                      ▼
┌─────────────┐      ┌──────────────┐
│  In-Memory  │      │    MySQL     │
│   Storage   │      │   Database   │
│  (backup)   │      │  (primary)   │
└─────────────┘      └──────┬───────┘
                             │
                             │ ✅ Write
                             │
                             │
                             ▼
                     ┌──────────────┐
                     │  Dashboard   │
                     │  APIs read   │
                     │  from MySQL  │
                     │    FIRST     │
                     └──────────────┘
                            ✅
                    Persistent across
                       restarts!
```

## Code Changes

### 1. `get_recent_logs()` - Loads from MySQL

```rust
pub async fn get_recent_logs(&self, limit: usize) -> Vec<ApiCallLog> {
    // Try MySQL first if available
    if let Some(writer) = &self.mysql_writer {
        if let Ok(logs) = writer.get_logs(limit, 0).await {
            return logs;  // ✅ Return MySQL data
        }
    }
    
    // Fallback to in-memory (for tests or if MySQL unavailable)
    let logs = self.logs.read().await;
    logs.iter().rev().take(limit).cloned().collect()
}
```

### 2. `get_endpoint_metrics()` - Loads aggregated metrics from MySQL

```rust
pub async fn get_endpoint_metrics(&self) -> Vec<EndpointMetrics> {
    // Try MySQL first if available
    if let Some(writer) = &self.mysql_writer {
        if let Ok(metrics) = writer.get_metrics().await {
            return metrics;  // ✅ Return MySQL aggregated data
        }
    }
    
    // Fallback to in-memory calculation
    // ... (existing in-memory logic)
}
```

### 3. `get_error_logs()` - New method to load errors from MySQL

```rust
pub async fn get_error_logs(&self, limit: usize) -> Vec<ApiCallLog> {
    // Try MySQL first if available
    if let Some(writer) = &self.mysql_writer {
        if let Ok(errors) = writer.get_error_logs(limit).await {
            return errors;  // ✅ Return MySQL error logs
        }
    }
    
    // Fallback to in-memory filtering
    let logs = self.logs.read().await;
    logs.iter()
        .rev()
        .filter(|log| log.response_status == "error")
        .take(limit)
        .cloned()
        .collect()
}
```

### 4. Dashboard API endpoint simplified

```rust
async fn get_errors(State(state): State<AppState>) -> impl IntoResponse {
    // Now uses MySQL-aware method
    let errors = state.metrics.get_error_logs(50).await;
    
    (StatusCode::OK, Json(json!({
        "errors": errors,
        "count": errors.len(),
        "timestamp": Utc::now()
    })))
}
```

## Benefits

### ✅ Data Persistence
- Dashboard shows **all historical data** across restarts
- No data loss between server restarts
- Scales to millions of records

### ✅ Backward Compatible
- In-memory fallback for tests
- Works even if MySQL is unavailable
- Graceful degradation

### ✅ Performance
- MySQL handles aggregation efficiently
- Indexed queries for fast retrieval
- In-memory cache still available as backup

## How It Works

### Data Flow on Request

1. **Client makes API call** → MeCP server
2. **Server records metrics** → Both in-memory AND MySQL
3. **MySQL stores** → Persistent storage

### Data Flow on Dashboard Load

1. **Dashboard requests data** → `/api/logs`, `/api/metrics`, `/api/errors`
2. **API endpoints call** → `MetricsCollector` methods
3. **MetricsCollector checks** → MySQL writer available?
   - ✅ **Yes** → Query MySQL directly (persistent data)
   - ❌ **No** → Use in-memory (tests/fallback)
4. **Return data** → Dashboard displays

## Testing the Fix

### Before Fix
```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Send some requests
./scripts/test-dashboard-flow.sh

# Terminal 1: Stop and restart server
# Ctrl+C, then cargo run --release again

# Browser: Open dashboard
# Result: ❌ Empty dashboard
```

### After Fix
```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Send some requests
./scripts/test-dashboard-flow.sh

# Terminal 1: Stop and restart server
# Ctrl+C, then cargo run --release again

# Browser: Open dashboard
# Result: ✅ All historical data visible!
```

## Verification

To verify your database has data:

```bash
# Check MySQL data
mysql -u mecp_user -p mecp_db -e "SELECT COUNT(*) FROM history_logs;"

# Should show number of records > 0
```

Open dashboard at `http://localhost:3000/dashboard` after restart and you should see:
- ✅ Total calls count
- ✅ Recent API calls table populated
- ✅ Endpoint metrics with historical data
- ✅ Error logs if any

## Related Files

- `/home/gnufoo/Work/Projects/gnufoo/MeCP/src/core/metrics.rs` - Data loading logic
- `/home/gnufoo/Work/Projects/gnufoo/MeCP/src/core/http_server.rs` - Dashboard API endpoints
- `/home/gnufoo/Work/Projects/gnufoo/MeCP/src/main.rs` - MySQL initialization
- `/home/gnufoo/Work/Projects/gnufoo/MeCP/dashboard/index.html` - Dashboard UI

## Performance Considerations

### MySQL Queries
- `get_logs()`: `LIMIT` + `OFFSET` with timestamp index
- `get_metrics()`: Aggregation query with `GROUP BY`
- `get_error_logs()`: Filtered query with index on `response_status`

### Optimization
All queries use:
- ✅ Indexed columns (`timestamp`, `response_status`)
- ✅ `LIMIT` clauses to prevent full table scans
- ✅ Connection pooling (implicit in `mysql_async`)

## Future Enhancements

1. **Connection Pooling**: Reuse MySQL connections instead of creating new ones
2. **Caching Layer**: Redis cache for frequently accessed data
3. **Pagination**: Dashboard pagination for very large datasets
4. **Real-time Updates**: WebSocket for live updates without polling

---

**Status**: ✅ Fixed and tested
**Date**: 2026-01-16
**Version**: MeCP v0.1.0
