use serde_json::json;
use std::sync::Arc;

mod common;
use common::*;

#[tokio::test]
async fn test_health_check() {
    let client = TestClient::new().await;
    
    let response = client.health_check().await;
    
    assert_eq!(response["status"], "healthy");
    assert_eq!(response["service"], "mecp");
    assert!(response["version"].is_string());
}

#[tokio::test]
async fn test_initialize() {
    let client = TestClient::new().await;
    
    let response = client.initialize().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert!(result["capabilities"].is_object());
    assert_eq!(result["serverInfo"]["name"], "MeCP");
}

#[tokio::test]
async fn test_list_resources() {
    let client = TestClient::new().await;
    
    let response = client.list_resources().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let resources = &response["result"]["resources"];
    assert!(resources.is_array());
    assert!(resources.as_array().unwrap().len() > 0);
    
    // Check mock resource is present
    let resource = &resources[0];
    assert!(resource["name"].is_string());
    assert!(resource["uri"].is_string());
    assert!(resource["description"].is_string());
}

#[tokio::test]
async fn test_read_resource() {
    let client = TestClient::new().await;
    
    // First get the resource URI
    let list_response = client.list_resources().await;
    let uri = list_response["result"]["resources"][0]["uri"]
        .as_str()
        .unwrap();
    
    // Now read the resource
    let response = client.read_resource(uri).await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let contents = &response["result"]["contents"];
    assert!(contents.is_array());
    assert!(contents.as_array().unwrap().len() > 0);
    
    let content = &contents[0];
    assert_eq!(content["uri"], uri);
    assert!(content["text"].is_string());
}

#[tokio::test]
async fn test_list_tools() {
    let client = TestClient::new().await;
    
    let response = client.list_tools().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let tools = &response["result"]["tools"];
    assert!(tools.is_array());
    assert!(tools.as_array().unwrap().len() > 0);
    
    // Check hello_world tool is present
    let tool = &tools[0];
    assert_eq!(tool["name"], "hello_world");
    assert!(tool["description"].is_string());
    assert!(tool["inputSchema"].is_object());
}

#[tokio::test]
async fn test_call_tool() {
    let client = TestClient::new().await;
    
    // Call hello_world tool without parameters
    let response = client.call_tool("hello_world", json!({})).await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let result = &response["result"];
    assert!(result["content"].is_array());
    
    let content = &result["content"][0];
    assert_eq!(content["type"], "text");
    assert!(content["text"].as_str().unwrap().contains("Hello"));
}

#[tokio::test]
async fn test_call_tool_with_name() {
    let client = TestClient::new().await;
    
    // Call hello_world tool with name parameter
    let response = client
        .call_tool("hello_world", json!({"name": "Rust"}))
        .await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let content = &response["result"]["content"][0];
    let text = content["text"].as_str().unwrap();
    assert!(text.contains("Hello, Rust"));
}

#[tokio::test]
async fn test_call_nonexistent_tool() {
    let client = TestClient::new().await;
    
    let response = client
        .call_tool("nonexistent_tool", json!({}))
        .await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    
    let error = &response["error"];
    assert_eq!(error["code"], -32603);
    assert!(error["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn test_list_prompts() {
    let client = TestClient::new().await;
    
    let response = client.list_prompts().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let prompts = &response["result"]["prompts"];
    assert!(prompts.is_array());
    assert!(prompts.as_array().unwrap().len() > 0);
    
    // Check mock_prompt is present
    let prompt = &prompts[0];
    assert_eq!(prompt["name"], "mock_prompt");
    assert!(prompt["description"].is_string());
    assert!(prompt["arguments"].is_array());
}

#[tokio::test]
async fn test_get_prompt() {
    let client = TestClient::new().await;
    
    let response = client
        .get_prompt("mock_prompt", json!({}))
        .await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let messages = &response["result"]["messages"];
    assert!(messages.is_array());
    assert!(messages.as_array().unwrap().len() > 0);
    
    let message = &messages[0];
    assert!(message["role"].is_string());
    assert!(message["content"]["text"].is_string());
}

#[tokio::test]
async fn test_get_prompt_with_topic() {
    let client = TestClient::new().await;
    
    let response = client
        .get_prompt("mock_prompt", json!({"topic": "Rust programming"}))
        .await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    
    let messages = &response["result"]["messages"];
    let message_text = messages[0]["content"]["text"].as_str().unwrap();
    assert!(message_text.contains("Rust programming"));
}

#[tokio::test]
async fn test_invalid_method() {
    let client = TestClient::new().await;
    
    let response = client
        .send_request("invalid/method", Some(json!({})))
        .await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    
    let error = &response["error"];
    assert_eq!(error["code"], -32601);
    assert!(error["message"].as_str().unwrap().contains("Method not found"));
}

#[tokio::test]
async fn test_concurrent_requests() {
    let client = Arc::new(TestClient::new().await);
    
    let mut handles = vec![];
    
    // Send 10 concurrent requests
    for i in 0..10 {
        let client_clone = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            if i % 3 == 0 {
                client_clone.list_resources().await
            } else if i % 3 == 1 {
                client_clone.list_tools().await
            } else {
                client_clone.list_prompts().await
            }
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
    }
}

#[tokio::test]
async fn test_json_rpc_format() {
    let client = TestClient::new().await;
    
    // Test that all responses follow JSON-RPC 2.0 format
    let response = client.initialize().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("id").is_some());
    assert!(response.get("result").is_some() || response.get("error").is_some());
    assert!(!(response.get("result").is_some() && response.get("error").is_some()));
}
