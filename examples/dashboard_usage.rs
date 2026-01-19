/// Example demonstrating how to use the MeCP monitoring dashboard with MySQL integration
/// 
/// This example shows how to:
/// 1. Create a metrics collector with MySQL backend
/// 2. Start the HTTP server with dashboard
/// 3. Access metrics and logs programmatically
/// 
/// Prerequisites:
/// - MySQL server running
/// - Database initialized with `./scripts/init-mysql-db.sh`
/// 
/// Run with:
/// ```
/// cargo run --example dashboard_usage
/// ```
/// 
/// Then open http://127.0.0.1:3000/dashboard in your browser

use mecp::core::{
    http_server::HttpServer,
    server::McpServer,
    metrics::{MetricsCollector, MySqlMetricsWriter},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("===========================================");
    println!("MeCP Dashboard Usage Example");
    println!("===========================================");
    println!();

    // Create MCP server
    println!("1. Creating MCP server...");
    let mcp_server = Arc::new(McpServer::new());

    // Option 1: Create metrics collector without MySQL (in-memory only)
    println!("2. Creating metrics collector (in-memory)...");
    let metrics = Arc::new(MetricsCollector::new());

    // Option 2: Create metrics collector with MySQL backend (uncomment to use)
    // println!("2. Creating metrics collector with MySQL backend...");
    // let mysql_writer = Arc::new(MySqlMetricsWriter::new(
    //     "localhost",
    //     3306,
    //     "mecp_db",
    //     "mecp_user",
    //     "mecp_password",
    // ));
    // let metrics = Arc::new(MetricsCollector::with_mysql_writer(mysql_writer));

    // Create HTTP server with metrics
    println!("3. Starting HTTP server with dashboard...");
    let host = "127.0.0.1".to_string();
    let port = std::env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let server = HttpServer::with_metrics(mcp_server, metrics, host.clone(), port);

    println!();
    println!("✓ Server starting on http://{}:{}", host, port);
    println!("✓ Dashboard available at http://{}:{}/dashboard", host, port);
    println!("✓ API endpoints:");
    println!("  - GET  /dashboard       - Web dashboard");
    println!("  - GET  /api/stats       - Overall statistics");
    println!("  - GET  /api/metrics     - Endpoint metrics");
    println!("  - GET  /api/logs        - Recent logs");
    println!("  - GET  /api/errors      - Recent errors");
    println!("  - POST /mcp             - MCP operations");
    println!();
    println!("Press Ctrl+C to stop the server");
    println!();

    // Start the server
    server.start().await?;

    Ok(())
}

// Example of programmatic access to metrics
#[cfg(test)]
mod tests {
    use super::*;
    use mecp::core::metrics::ApiCallLog;
    use chrono::Utc;

    #[tokio::test]
    async fn example_programmatic_access() {
        // Create metrics collector
        let metrics = MetricsCollector::new();

        // Simulate some API calls
        for i in 0..10 {
            let log = ApiCallLog {
                id: None,
                method: format!("method_{}", i % 3),
                endpoint: "/mcp".to_string(),
                request_params: None,
                response_data: if i % 5 == 0 { None } else { Some(r#"{"result":"ok"}"#.to_string()) },
                response_status: if i % 5 == 0 { "error" } else { "success" }.to_string(),
                error_message: if i % 5 == 0 { Some("Example error".to_string()) } else { None },
                duration_ms: 50 + (i * 5) as u64,
                timestamp: Utc::now(),
                client_info: Some("test-client".to_string()),
            };

            metrics.record_call(log).await.unwrap();
        }

        // Get endpoint metrics
        let endpoint_metrics = metrics.get_endpoint_metrics().await;
        println!("Endpoint Metrics:");
        for metric in endpoint_metrics {
            println!("  - {}: {} calls, {} successful, {} failed, {:.2}ms avg",
                metric.method,
                metric.total_calls,
                metric.successful_calls,
                metric.failed_calls,
                metric.avg_duration_ms
            );
        }

        // Get recent logs
        let logs = metrics.get_recent_logs(5).await;
        println!("\nRecent Logs:");
        for log in logs {
            println!("  - {}: {} - {} ({}ms)",
                log.timestamp.format("%Y-%m-%d %H:%M:%S"),
                log.method,
                log.response_status,
                log.duration_ms
            );
        }
    }
}
