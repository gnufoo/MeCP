/// MeCP Test Client
/// 
/// Simulates realistic client traffic to test the entire monitoring flow:
/// Client -> MeCP Server -> Database -> Dashboard
/// 
/// This client sends various MCP requests including both successful calls
/// and intentional errors to verify error tracking.
/// 
/// Usage:
/// ```bash
/// # Send 50 requests (default)
/// cargo run --example test_client
/// 
/// # Send custom number of requests
/// cargo run --example test_client -- 100
/// 
/// # Use custom server URL
/// MCP_URL=http://localhost:3000 cargo run --example test_client
/// ```

use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
struct TestResult {
    total: usize,
    successful: usize,
    failed: usize,
}

async fn send_mcp_request(
    client: &reqwest::Client,
    base_url: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": rand::random::<u32>(),
        "method": method,
        "params": params,
    });

    let response = client
        .post(format!("{}/mcp", base_url))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let body = response.json::<serde_json::Value>().await?;
    Ok(body)
}

async fn test_initialize(
    client: &reqwest::Client,
    base_url: &str,
    client_id: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {},
        "clientInfo": {
            "name": format!("test-client-{}", client_id),
            "version": "1.0.0"
        }
    });

    let response = send_mcp_request(client, base_url, "initialize", params).await?;
    
    if response.get("error").is_some() {
        return Err("Initialize request failed".into());
    }
    
    Ok(())
}

async fn test_resources_list(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = send_mcp_request(client, base_url, "resources/list", json!({})).await?;
    
    if response.get("error").is_some() {
        return Err("Resources list request failed".into());
    }
    
    Ok(())
}

async fn test_resources_read(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = json!({
        "uri": "mock://example/resource"
    });

    let response = send_mcp_request(client, base_url, "resources/read", params).await?;
    
    if response.get("error").is_some() {
        return Err("Resources read request failed".into());
    }
    
    Ok(())
}

async fn test_tools_list(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = send_mcp_request(client, base_url, "tools/list", json!({})).await?;
    
    if response.get("error").is_some() {
        return Err("Tools list request failed".into());
    }
    
    Ok(())
}

async fn test_tools_call(
    client: &reqwest::Client,
    base_url: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = json!({
        "name": "hello_world",
        "arguments": {
            "name": name
        }
    });

    let response = send_mcp_request(client, base_url, "tools/call", params).await?;
    
    if response.get("error").is_some() {
        return Err("Tools call request failed".into());
    }
    
    Ok(())
}

async fn test_tools_call_error(
    client: &reqwest::Client,
    base_url: &str,
    tool_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = json!({
        "name": tool_name,
        "arguments": {}
    });

    let response = send_mcp_request(client, base_url, "tools/call", params).await?;
    
    // We expect an error here
    if response.get("error").is_none() {
        return Err("Expected error but got success".into());
    }
    
    Ok(())
}

async fn test_prompts_list(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = send_mcp_request(client, base_url, "prompts/list", json!({})).await?;
    
    if response.get("error").is_some() {
        return Err("Prompts list request failed".into());
    }
    
    Ok(())
}

async fn test_prompts_get(
    client: &reqwest::Client,
    base_url: &str,
    topic: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let params = json!({
        "name": "mock_prompt",
        "arguments": {
            "topic": topic
        }
    });

    let response = send_mcp_request(client, base_url, "prompts/get", params).await?;
    
    if response.get("error").is_some() {
        return Err("Prompts get request failed".into());
    }
    
    Ok(())
}

async fn check_server_health(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", base_url))
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err("Server health check failed".into());
    }
    
    Ok(())
}

async fn verify_dashboard_api(base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    println!("\n📊 Verifying Dashboard API Endpoints...\n");
    
    // Test /api/stats
    print!("  → Testing /api/stats... ");
    let stats = client
        .get(format!("{}/api/stats", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("✓");
    println!("     Total calls: {}", stats["total_calls"]);
    println!("     Success rate: {}%", stats["success_rate"]);
    println!("     Total errors: {}", stats["total_errors"]);
    
    // Test /api/metrics
    print!("  → Testing /api/metrics... ");
    let metrics = client
        .get(format!("{}/api/metrics", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("✓");
    if let Some(metrics_array) = metrics["metrics"].as_array() {
        println!("     Endpoints tracked: {}", metrics_array.len());
    }
    
    // Test /api/logs
    print!("  → Testing /api/logs... ");
    let logs = client
        .get(format!("{}/api/logs", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("✓");
    println!("     Recent logs: {}", logs["count"]);
    
    // Test /api/errors
    print!("  → Testing /api/errors... ");
    let errors = client
        .get(format!("{}/api/errors", base_url))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    println!("✓");
    println!("     Error logs: {}", errors["count"]);
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔════════════════════════════════════════════════════╗");
    println!("║   MeCP Test Client - Dashboard Flow Tester        ║");
    println!("╚════════════════════════════════════════════════════╝\n");

    // Get configuration
    let base_url = std::env::var("MCP_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let num_requests: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    println!("🎯 Target: {}", base_url);
    println!("📊 Requests: {}\n", num_requests);

    // Check server health
    print!("🔍 Checking server health... ");
    match check_server_health(&base_url).await {
        Ok(_) => println!("✓ Server is running"),
        Err(e) => {
            println!("✗ Server not responding: {}", e);
            println!("\nPlease start the server first:");
            println!("  cargo run --release");
            return Err(e);
        }
    }

    println!("\n════════════════════════════════════════════════════");
    println!("Starting test requests...");
    println!("════════════════════════════════════════════════════\n");

    let client = reqwest::Client::new();
    let mut result = TestResult {
        total: 0,
        successful: 0,
        failed: 0,
    };

    let requests_per_type = num_requests / 8;

    // Test data
    let names = vec!["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry"];
    let topics = vec![
        "Rust programming",
        "AI development",
        "Database design",
        "Web development",
        "System architecture",
        "Testing strategies",
        "Performance optimization",
        "Security best practices",
    ];
    let fake_tools = vec![
        "nonexistent_tool",
        "invalid_tool",
        "unknown_tool",
        "missing_tool",
        "bad_tool",
    ];

    // 1. Initialize requests
    println!("🔧 [1/8] Testing initialize endpoint...");
    for i in 0..requests_per_type {
        result.total += 1;
        match test_initialize(&client, &base_url, i).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 2. Resources list
    println!("📦 [2/8] Testing resources/list endpoint...");
    for _ in 0..requests_per_type {
        result.total += 1;
        match test_resources_list(&client, &base_url).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 3. Resources read
    println!("📖 [3/8] Testing resources/read endpoint...");
    for _ in 0..requests_per_type {
        result.total += 1;
        match test_resources_read(&client, &base_url).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 4. Tools list
    println!("🔨 [4/8] Testing tools/list endpoint...");
    for _ in 0..requests_per_type {
        result.total += 1;
        match test_tools_list(&client, &base_url).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 5. Tools call (success)
    println!("✅ [5/8] Testing tools/call endpoint (success)...");
    for i in 0..requests_per_type {
        result.total += 1;
        let name = names[i % names.len()];
        match test_tools_call(&client, &base_url, name).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 6. Tools call (errors)
    println!("❌ [6/8] Testing tools/call endpoint (errors)...");
    for i in 0..requests_per_type {
        result.total += 1;
        let tool = fake_tools[i % fake_tools.len()];
        match test_tools_call_error(&client, &base_url, tool).await {
            Ok(_) => {
                print!("E");
                result.successful += 1; // These are expected errors
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 7. Prompts list
    println!("💬 [7/8] Testing prompts/list endpoint...");
    for _ in 0..requests_per_type {
        result.total += 1;
        match test_prompts_list(&client, &base_url).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // 8. Prompts get
    println!("📝 [8/8] Testing prompts/get endpoint...");
    for i in 0..requests_per_type {
        result.total += 1;
        let topic = topics[i % topics.len()];
        match test_prompts_get(&client, &base_url, topic).await {
            Ok(_) => {
                print!(".");
                result.successful += 1;
            }
            Err(_) => {
                print!("×");
                result.failed += 1;
            }
        }
        sleep(Duration::from_millis(50)).await;
    }
    println!(" Done");

    // Wait for metrics to be written
    println!("\n⏳ Waiting for metrics to be written to database...");
    sleep(Duration::from_secs(2)).await;

    // Results
    println!("\n════════════════════════════════════════════════════");
    println!("Test Results");
    println!("════════════════════════════════════════════════════\n");
    println!("  Total requests:      {}", result.total);
    println!("  ✓ Successful:        {}", result.successful);
    println!("  ✗ Failed:            {}", result.failed);
    println!("  Success rate:        {:.1}%", 
        (result.successful as f64 / result.total as f64) * 100.0);

    // Verify dashboard API
    if let Err(e) = verify_dashboard_api(&base_url).await {
        println!("\n⚠️  Warning: Could not verify dashboard API: {}", e);
    }

    // Final instructions
    println!("\n════════════════════════════════════════════════════");
    println!("✅ Flow Test Complete!");
    println!("════════════════════════════════════════════════════\n");
    println!("Next steps:");
    println!("  1. Open dashboard: {}/dashboard", base_url);
    println!("  2. Verify metrics are displayed correctly");
    println!("  3. Check that errors are shown in the errors section");
    println!("  4. Confirm auto-refresh is working");
    println!("\n🌐 Dashboard URL: {}/dashboard\n", base_url);

    Ok(())
}
