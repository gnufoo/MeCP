use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

static REQUEST_ID: AtomicU16 = AtomicU16::new(1);
const TEST_SERVER_PORT: u16 = 13000;

lazy_static! {
    static ref TEST_SERVER: Mutex<Option<()>> = {
        // Start the server once
        std::thread::spawn(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // Create server
                let server = Arc::new(mecp::core::server::McpServer::new());
                
                // Register mock components
                server
                    .register_resource(Box::new(mecp::resources::mock::MockResource::new()))
                    .await;
                server
                    .register_tool(Box::new(mecp::tools::mock::HelloWorldTool::new()))
                    .await;
                server
                    .register_prompt(Box::new(mecp::prompts::mock::MockPrompt::new()))
                    .await;
                
                // Start HTTP server
                let http_server = mecp::core::http_server::HttpServer::new(server, TEST_SERVER_PORT);
                
                println!("✓ Test server starting on port {}", TEST_SERVER_PORT);
                
                // This will run indefinitely
                let _ = http_server.start().await;
            });
        });
        
        // Wait for server to be ready
        std::thread::sleep(Duration::from_millis(1500));
        
        Mutex::new(Some(()))
    };
}

/// Ensure server is initialized
fn ensure_server() {
    let _ = *TEST_SERVER;
}

/// Test client for MCP server
pub struct TestClient {
    client: Client,
    base_url: String,
}

impl TestClient {
    /// Create a new test client
    pub async fn new() -> Self {
        // Ensure server is running
        ensure_server();
        
        // Wait a bit more to ensure readiness
        sleep(Duration::from_millis(200)).await;
        
        // Verify server is reachable
        let client = Client::new();
        for attempt in 1..=20 {
            if let Ok(response) = client
                .get(format!("http://127.0.0.1:{}/health", TEST_SERVER_PORT))
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                if response.status().is_success() {
                    if attempt > 1 {
                        println!("✓ Test server ready after {} attempts", attempt);
                    }
                    break;
                }
            }
            if attempt == 20 {
                panic!("Test server failed to start after 20 attempts");
            }
            sleep(Duration::from_millis(100)).await;
        }
        
        Self {
            client,
            base_url: format!("http://127.0.0.1:{}", TEST_SERVER_PORT),
        }
    }

    /// Send a JSON-RPC request
    pub async fn send_request(&self, method: &str, params: Option<Value>) -> Value {
        let id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
        
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        let response = self
            .client
            .post(format!("{}/mcp", self.base_url))
            .json(&request)
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .expect("Failed to send request");

        response.json().await.expect("Failed to parse response")
    }

    /// Health check
    pub async fn health_check(&self) -> Value {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to send health check");

        response.json().await.expect("Failed to parse response")
    }

    /// Initialize MCP session
    pub async fn initialize(&self) -> Value {
        self.send_request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            })),
        )
        .await
    }

    /// List resources
    pub async fn list_resources(&self) -> Value {
        self.send_request("resources/list", None).await
    }

    /// Read a resource
    pub async fn read_resource(&self, uri: &str) -> Value {
        self.send_request("resources/read", Some(json!({ "uri": uri })))
            .await
    }

    /// List tools
    pub async fn list_tools(&self) -> Value {
        self.send_request("tools/list", None).await
    }

    /// Call a tool
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Value {
        self.send_request(
            "tools/call",
            Some(json!({
                "name": name,
                "arguments": arguments
            })),
        )
        .await
    }

    /// List prompts
    pub async fn list_prompts(&self) -> Value {
        self.send_request("prompts/list", None).await
    }

    /// Get a prompt
    pub async fn get_prompt(&self, name: &str, arguments: Value) -> Value {
        self.send_request(
            "prompts/get",
            Some(json!({
                "name": name,
                "arguments": arguments
            })),
        )
        .await
    }
}
