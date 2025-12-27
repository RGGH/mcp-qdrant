[![Rust](https://github.com/e21-ai/mcp-qdrant/actions/workflows/rust.yml/badge.svg)](https://github.com/e21-ai/mcp-qdrant/actions/workflows/rust.yml)
# MCP Qdrant 

## TL;DR

- Run as a server, preferably on same server as the Qdrant collection
- Use LLM + LLM CLient + mcp-qdrant to find stuff, just with semantic meaning




## The flow:

```code
1. Client makes HTTP request
   ↓
2. Axum server receives TCP connection on port 8766
   ↓
3. Axum parses HTTP request
   ↓
4. Router checks path: "/mcp" → forward to service
   ↓
5. StreamableHttpService handles MCP protocol
   ↓
6. Your QdrantMCPServer executes the tool
   ↓
7. Response bubbles back up through service → router → server
   ↓
8. Axum server sends HTTP response over TCP
```


# Testing

https://modelcontextprotocol.io/docs/tools/inspector

<img width="1919" height="805" alt="image" src="https://github.com/user-attachments/assets/c9b7ac51-d734-47e1-a215-b263592e35c7" />

If you use Goose :
```bash
  mcp-qdrant:
    enabled: true
    type: streamable_http
    name: mcp-qdrant
    description: qdrant-mcp
    uri: http://localhost:8766/mcp
    envs: {}
    env_keys: []
    headers: {}
    timeout: 12
    bundled: null
    available_tools: []
```



