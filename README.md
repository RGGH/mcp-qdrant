[![Rust](https://github.com/e21-ai/mcp-qdrant/actions/workflows/rust.yml/badge.svg)](https://github.com/e21-ai/mcp-qdrant/actions/workflows/rust.yml)
# MCP Qdrant 

## TL;DR

- Run as a server, preferably on same server as the Qdrant collection
- Use LLM + LLM CLient + mcp-qdrant to find stuff, just with semantic meaning
- Add / add more via Loader or use the Chrome extension, failing that, give me money to do it for you


Reference:

https://github.com/qdrant/mcp-server-qdrant

They have a "store" and a "find" endpoint, we will focus on "find" because "Loader" will do the "store" functions.

The "store" needs to be done with human in the loop, and wth additional metadata, so the idea of using LLM to call MCP to upsert data into Qdrant is likely a showcase rather than a valid solution.

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



