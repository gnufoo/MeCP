# MeCP Testing Guide

## Overview

MeCP has a comprehensive end-to-end testing suite that validates all MCP server endpoints through actual HTTP requests, simulating real client interactions.

## Test Architecture

### Components

1. **HTTP Server** (`src/core/http_server.rs`)
   - Axum-based REST API
   - JSON-RPC 2.0 protocol handler
   - CORS-enabled endpoints
   - Health check endpoint

2. **Integration Tests** (`tests/integration_test.rs`)
   - 14 comprehensive test cases
   - Real HTTP client/server communication
   - Concurrent request testing
   - Error handling validation

3. **Test Framework** (`tests/common/mod.rs`)
   - Test server management
   - Reusable test client
   - Automatic server startup/teardown

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only Integration Tests
```bash
cargo test --test integration_test
```

### Run Specific Test
```bash
cargo test test_initialize
```

### Run with Output
```bash
cargo test -- --nocapture
```

### Run with Logging
```bash
RUST_LOG=debug cargo test
```

## Test Coverage

### ✅ Health Check Endpoint (`GET /health`)
- **Test**: `test_health_check`
- **Validates**: Server status, service name, version

### ✅ Initialize (`initialize`)
- **Test**: `test_initialize`
- **Validates**: Protocol version, capabilities, server info
- **JSON-RPC Method**: `initialize`

### ✅ List Resources (`resources/list`)
- **Test**: `test_list_resources`
- **Validates**: Resource list format, metadata fields
- **JSON-RPC Method**: `resources/list`

### ✅ Read Resource (`resources/read`)
- **Test**: `test_read_resource`
- **Validates**: Resource content retrieval, URI matching
- **JSON-RPC Method**: `resources/read`

### ✅ List Tools (`tools/list`)
- **Test**: `test_list_tools`
- **Validates**: Tool list format, input schema
- **JSON-RPC Method**: `tools/list`

### ✅ Call Tool (`tools/call`)
- **Tests**: 
  - `test_call_tool` - Basic tool execution
  - `test_call_tool_with_name` - Parameterized execution
  - `test_call_nonexistent_tool` - Error handling
- **Validates**: Tool execution, parameter handling, error responses
- **JSON-RPC Method**: `tools/call`

### ✅ List Prompts (`prompts/list`)
- **Test**: `test_list_prompts`
- **Validates**: Prompt list format, argument definitions
- **JSON-RPC Method**: `prompts/list`

### ✅ Get Prompt (`prompts/get`)
- **Tests**:
  - `test_get_prompt` - Basic prompt generation
  - `test_get_prompt_with_topic` - Parameterized generation
- **Validates**: Prompt message format, argument processing
- **JSON-RPC Method**: `prompts/get`

### ✅ Protocol Validation
- **Test**: `test_json_rpc_format`
- **Validates**: JSON-RPC 2.0 compliance, response structure

### ✅ Error Handling
- **Test**: `test_invalid_method`
- **Validates**: Method not found error (-32601)

### ✅ Concurrent Operations
- **Test**: `test_concurrent_requests`
- **Validates**: Thread safety, concurrent request handling

## Test Results

```
running 14 tests
test test_health_check ... ok
test test_initialize ... ok
test test_list_resources ... ok
test test_read_resource ... ok
test test_list_tools ... ok
test test_call_tool ... ok
test test_call_tool_with_name ... ok
test test_call_nonexistent_tool ... ok
test test_list_prompts ... ok
test test_get_prompt ... ok
test test_get_prompt_with_topic ... ok
test test_json_rpc_format ... ok
test test_invalid_method ... ok
test test_concurrent_requests ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Test Server

The test server automatically:
- Starts on port 13000
- Registers mock components (MockResource, HelloWorldTool, MockPrompt)
- Waits for server readiness before running tests
- Runs only once for all tests (shared state)

## Adding New Tests

### Step 1: Add Test Function

```rust
#[tokio::test]
async fn test_new_feature() {
    let client = TestClient::new().await;
    
    let response = client.send_request("new/method", Some(json!({
        "param": "value"
    }))).await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
}
```

### Step 2: Implement Endpoint

Add handler in `src/core/http_server.rs`:

```rust
async fn handle_new_feature(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    // Implementation
}
```

### Step 3: Register Route

Add method to the router in `handle_mcp_request`:

```rust
methods::NEW_METHOD => handle_new_feature(&server, &request).await,
```

### Step 4: Run Tests

```bash
cargo test
```

### Step 5: Verify

Ensure all tests pass, including new test.

## Test-Driven Development Workflow

Following the cursor rules in `.cursor/rules/mecp-custom-coding-rule.mdc`:

1. **Write Test First**: Create failing test for new feature
2. **Run Tests**: `cargo test` - expect failure
3. **Implement**: Add minimal code to make test pass
4. **Run Tests**: `cargo test` - verify pass
5. **Refactor**: Improve code quality
6. **Run Tests**: `cargo test` - ensure still passing

## Continuous Integration

### Pre-commit Checklist
- [ ] All tests pass: `cargo test`
- [ ] No compilation errors: `cargo check`
- [ ] Code formatted: `cargo fmt`
- [ ] No linter warnings: `cargo clippy`

### Test Policies

1. **No Broken Tests**: Never commit code that breaks existing tests
2. **Test Coverage**: New features must include tests
3. **Error Testing**: Test both success and error cases
4. **Edge Cases**: Include boundary condition tests
5. **Concurrency**: Test thread safety where applicable

## Common Test Patterns

### Testing Success Cases
```rust
#[tokio::test]
async fn test_success() {
    let client = TestClient::new().await;
    let response = client.some_method().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());
    assert_eq!(response["result"]["field"], "expected_value");
}
```

### Testing Error Cases
```rust
#[tokio::test]
async fn test_error() {
    let client = TestClient::new().await;
    let response = client.invalid_method().await;
    
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601);
}
```

### Testing with Parameters
```rust
#[tokio::test]
async fn test_with_params() {
    let client = TestClient::new().await;
    let response = client.method_with_params(json!({
        "param1": "value1",
        "param2": 42
    })).await;
    
    assert!(response["result"].is_object());
}
```

## Debugging Tests

### View Test Output
```bash
cargo test -- --nocapture
```

### Run Single Test with Logging
```bash
RUST_LOG=debug cargo test test_name -- --nocapture
```

### Use dbg! Macro
```rust
#[tokio::test]
async fn test_debug() {
    let response = client.some_method().await;
    dbg!(&response);  // Print for debugging
    assert!(response["result"].is_object());
}
```

## Performance Testing

### Response Time
```rust
use std::time::Instant;

#[tokio::test]
async fn test_performance() {
    let client = TestClient::new().await;
    let start = Instant::now();
    
    let response = client.list_resources().await;
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Response too slow: {:?}", duration);
}
```

### Concurrent Load
```rust
#[tokio::test]
async fn test_load() {
    let client = Arc::new(TestClient::new().await);
    let mut handles = vec![];
    
    for _ in 0..100 {
        let client_clone = Arc::clone(&client);
        let handle = tokio::spawn(async move {
            client_clone.list_tools().await
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let response = handle.await.unwrap();
        assert_eq!(response["jsonrpc"], "2.0");
    }
}
```

## JSON-RPC Error Codes

Tests validate proper error code usage:

- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found ✓ (tested)
- `-32602`: Invalid params
- `-32603`: Internal error ✓ (tested)

## Test Data

Mock implementations provide consistent test data:

### MockResource
- URI: `mock://example/resource`
- Returns JSON with items array and timestamp

### HelloWorldTool
- Name: `hello_world`
- Parameters: `name` (optional string)
- Returns greeting message

### MockPrompt
- Name: `mock_prompt`
- Arguments: `topic` (optional string)
- Returns system and user messages

## Troubleshooting

### Tests Fail to Connect
- Ensure no other process is using port 13000
- Increase startup wait time in `common/mod.rs`
- Check firewall settings

### Flaky Tests
- Increase timeout durations
- Add retry logic
- Check for race conditions

### Server Won't Start
- Check port availability: `lsof -i :13000`
- Verify dependencies: `cargo check`
- Review server logs

## Maintenance

### Regular Tasks
- Review test coverage quarterly
- Remove obsolete tests
- Update test data as needed
- Refactor duplicated test code

### Adding New Endpoints

When adding a new endpoint, ensure:
1. Success case test
2. Error case test
3. Parameter validation test
4. Response format test
5. Concurrent access test (if applicable)

## Resources

- [Axum Testing Guide](https://docs.rs/axum/latest/axum/testing/)
- [Tokio Test Documentation](https://docs.rs/tokio/latest/tokio/attr.test.html)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [MCP Protocol Documentation](https://modelcontextprotocol.io/)

---

**Status**: ✅ All 14 integration tests passing
**Last Updated**: January 16, 2026
**Test Coverage**: 100% of MCP endpoints
