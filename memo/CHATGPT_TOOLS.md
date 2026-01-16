# ChatGPT Connector Tools - Search and Fetch

## Overview

Your MeCP server now implements the two required tools for ChatGPT Connectors and deep research:

1. **`search`** - Search for information and return results with URLs
2. **`fetch`** - Fetch content from a URL

Both tools are implemented with mock data and are ready to use.

## Tool Details

### 1. Search Tool

**Name:** `search`

**Description:** Search for information and return relevant results with URLs. Required for ChatGPT Connectors and deep research.

**Parameters:**
- `query` (string, required) - Search query string
- `max_results` (number, optional) - Maximum number of results to return (default: 10)

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "Rust programming",
      "max_results": 5
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"query\":\"Rust programming\",\"total_results\":5,\"results\":[{\"title\":\"Result 1: Information about Rust programming\",\"url\":\"https://example.com/result1\",\"snippet\":\"...\",\"relevance_score\":0.95},...],\"timestamp\":\"2026-01-16T15:28:39Z\"}"
      }
    ],
    "isError": false
  }
}
```

### 2. Fetch Tool

**Name:** `fetch`

**Description:** Fetch content from a URL. Required for ChatGPT Connectors and deep research.

**Parameters:**
- `url` (string, required) - URL to fetch content from

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "fetch",
    "arguments": {
      "url": "https://example.com/result1"
    }
  }
}
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\"url\":\"https://example.com/result1\",\"title\":\"Result 1 Content\",\"content\":\"This is mock content...\",\"content_type\":\"text/html\",\"content_length\":234,\"fetched_at\":\"2026-01-16T15:28:39Z\",\"status\":\"success\"}"
      }
    ],
    "isError": false
  }
}
```

## Testing the Tools

### List All Tools

```bash
curl -k -X POST https://mecp.io/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list"
  }'
```

### Test Search Tool

```bash
curl -k -X POST https://mecp.io/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "search",
      "arguments": {
        "query": "Rust programming",
        "max_results": 3
      }
    }
  }'
```

### Test Fetch Tool

```bash
curl -k -X POST https://mecp.io/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "fetch",
      "arguments": {
        "url": "https://example.com/result1"
      }
    }
  }'
```

## Mock Data

Currently, both tools return mock data:

- **Search** returns 5 predefined mock results with different relevance scores
- **Fetch** returns mock content based on the URL pattern

## Next Steps: Real Implementation

To replace mock data with real functionality:

### Search Tool
- Integrate with a search API (Google, Bing, etc.)
- Or implement your own search index
- Or connect to a database with search capabilities

### Fetch Tool
- Use `reqwest` or similar HTTP client to fetch real URLs
- Parse HTML content
- Extract text and metadata
- Handle errors and timeouts

## ChatGPT Connector Setup

Now that you have `search` and `fetch` tools:

1. **Update ChatGPT Connector:**
   - URL: `https://mecp.io`
   - SSE endpoint: `https://mecp.io/sse`
   - MCP endpoint: `https://mecp.io/mcp`

2. **ChatGPT will automatically:**
   - Detect the `search` and `fetch` tools
   - Use them for deep research
   - Enable connector functionality

3. **Test in ChatGPT:**
   - Try asking questions that require research
   - ChatGPT will use `search` to find information
   - Then use `fetch` to get detailed content

## Implementation Location

- **Code:** `src/tools/mock.rs`
- **Registration:** `src/main.rs` (lines 26-28)

## Current Tools

Your server now has 3 tools:
1. `hello_world` - Simple greeting tool
2. `search` - Search tool (required for ChatGPT)
3. `fetch` - Fetch tool (required for ChatGPT)

All tools are registered and ready to use!
