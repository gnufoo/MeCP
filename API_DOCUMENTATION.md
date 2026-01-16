# MeCP API Documentation

## Overview

MeCP exposes a JSON-RPC 2.0 compliant API over HTTP for Model Context Protocol (MCP) operations.

## Base URL

```
http://127.0.0.1:3000
```

Default port is 3000, configurable via `MCP_PORT` environment variable.

## Endpoints

### Health Check

**GET** `/health`

Returns server health status.

**Response:**
```json
{
  "status": "healthy",
  "service": "mecp",
  "version": "0.1.0"
}
```

### MCP Operations

**POST** `/mcp`

All MCP operations use this single endpoint with JSON-RPC 2.0 protocol.

**Request Format:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "method_name",
  "params": {
    // method-specific parameters
  }
}
```

**Response Format (Success):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    // method-specific result
  }
}
```

**Response Format (Error):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32600,
    "message": "Error description"
  }
}
```

## Methods

### 1. initialize

Initialize an MCP session.

**Method:** `initialize`

**Parameters:**
```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {},
  "clientInfo": {
    "name": "client-name",
    "version": "1.0.0"
  }
}
```

**Result:**
```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "resources": {
      "subscribe": false,
      "listChanged": false
    },
    "tools": {
      "listChanged": false
    },
    "prompts": {
      "listChanged": false
    }
  },
  "serverInfo": {
    "name": "MeCP",
    "version": "0.1.0"
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

### 2. resources/list

List all available resources.

**Method:** `resources/list`

**Parameters:** None

**Result:**
```json
{
  "resources": [
    {
      "uri": "mock://example/resource",
      "name": "mock_resource",
      "description": "A mock resource for demonstration purposes",
      "mimeType": "application/json"
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "resources/list"
  }'
```

### 3. resources/read

Read a specific resource by URI.

**Method:** `resources/read`

**Parameters:**
```json
{
  "uri": "mock://example/resource"
}
```

**Result:**
```json
{
  "contents": [
    {
      "uri": "mock://example/resource",
      "mimeType": "application/json",
      "text": "{\"message\":\"This is mock resource data\",\"items\":[\"item1\",\"item2\",\"item3\"]}",
      "blob": null
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "resources/read",
    "params": {
      "uri": "mock://example/resource"
    }
  }'
```

### 4. tools/list

List all available tools.

**Method:** `tools/list`

**Parameters:** None

**Result:**
```json
{
  "tools": [
    {
      "name": "hello_world",
      "description": "A simple hello world tool that greets users",
      "inputSchema": {
        "type": "object",
        "properties": {
          "name": {
            "type": "string",
            "description": "Name to greet"
          }
        },
        "required": []
      }
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 4,
    "method": "tools/list"
  }'
```

### 5. tools/call

Execute a tool with given arguments.

**Method:** `tools/call`

**Parameters:**
```json
{
  "name": "hello_world",
  "arguments": {
    "name": "Alice"
  }
}
```

**Result:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "{\"message\":\"Hello, Alice! Welcome to MeCP.\",\"timestamp\":\"2026-01-16T15:30:00Z\"}"
    }
  ],
  "isError": false
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 5,
    "method": "tools/call",
    "params": {
      "name": "hello_world",
      "arguments": {
        "name": "Alice"
      }
    }
  }'
```

### 6. prompts/list

List all available prompts.

**Method:** `prompts/list`

**Parameters:** None

**Result:**
```json
{
  "prompts": [
    {
      "name": "mock_prompt",
      "description": "A mock prompt that generates a conversation starter",
      "arguments": [
        {
          "name": "topic",
          "description": "Topic for the conversation",
          "required": false
        }
      ]
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 6,
    "method": "prompts/list"
  }'
```

### 7. prompts/get

Generate a prompt with given arguments.

**Method:** `prompts/get`

**Parameters:**
```json
{
  "name": "mock_prompt",
  "arguments": {
    "topic": "Rust programming"
  }
}
```

**Result:**
```json
{
  "messages": [
    {
      "role": "system",
      "content": {
        "type": "text",
        "text": "You are a helpful assistant discussing Rust programming. Provide clear and concise responses."
      }
    },
    {
      "role": "user",
      "content": {
        "type": "text",
        "text": "Let's start a conversation about Rust programming."
      }
    }
  ]
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 7,
    "method": "prompts/get",
    "params": {
      "name": "mock_prompt",
      "arguments": {
        "topic": "Rust programming"
      }
    }
  }'
```

## Error Codes

MeCP follows JSON-RPC 2.0 error code conventions:

| Code | Message | Description |
|------|---------|-------------|
| -32700 | Parse error | Invalid JSON was received |
| -32600 | Invalid Request | The JSON sent is not a valid Request object |
| -32601 | Method not found | The method does not exist |
| -32602 | Invalid params | Invalid method parameter(s) |
| -32603 | Internal error | Internal JSON-RPC error |

**Error Response Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found: invalid_method"
  }
}
```

## Client Libraries

### Rust

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let response = client
        .post("http://127.0.0.1:3000/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .send()
        .await?;
    
    let result: serde_json::Value = response.json().await?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    
    Ok(())
}
```

### Python

```python
import requests
import json

def call_mcp(method, params=None):
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method
    }
    if params:
        payload["params"] = params
    
    response = requests.post(
        "http://127.0.0.1:3000/mcp",
        json=payload
    )
    return response.json()

# Example: List tools
result = call_mcp("tools/list")
print(json.dumps(result, indent=2))

# Example: Call tool
result = call_mcp("tools/call", {
    "name": "hello_world",
    "arguments": {"name": "Python User"}
})
print(json.dumps(result, indent=2))
```

### JavaScript/Node.js

```javascript
const axios = require('axios');

async function callMCP(method, params = null) {
  const payload = {
    jsonrpc: '2.0',
    id: 1,
    method: method
  };
  
  if (params) {
    payload.params = params;
  }
  
  const response = await axios.post(
    'http://127.0.0.1:3000/mcp',
    payload
  );
  
  return response.data;
}

// Example: List resources
callMCP('resources/list')
  .then(result => console.log(JSON.stringify(result, null, 2)))
  .catch(error => console.error(error));

// Example: Read resource
callMCP('resources/read', { uri: 'mock://example/resource' })
  .then(result => console.log(JSON.stringify(result, null, 2)))
  .catch(error => console.error(error));
```

## Rate Limiting

Currently, no rate limiting is implemented. For production use, consider adding rate limiting middleware.

## CORS

CORS is enabled with permissive settings for development. For production, configure appropriate CORS policies.

## Authentication

Currently, no authentication is required. For production use, implement appropriate authentication/authorization.

## Versioning

API version is indicated in the server info returned by the `initialize` method.

Current version: `0.1.0`

## Testing

Use the provided integration tests to verify API functionality:

```bash
cargo test --test integration_test
```

## Support

For issues or questions:
- Review the [TESTING.md](TESTING.md) documentation
- Check the [ARCHITECTURE.md](ARCHITECTURE.md) for design details
- Refer to the [MCP Protocol](https://modelcontextprotocol.io/) specification

---

**Last Updated**: January 16, 2026  
**API Version**: 0.1.0  
**Protocol Version**: 2024-11-05
