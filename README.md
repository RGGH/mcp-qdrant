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

---
---
---

# Qdrant MCP Server - 

A powerful Model Context Protocol (MCP) server that enables semantic search capabilities through Qdrant vector database with local FastEmbed text embedding. Connect Claude or any MCP-compatible client to your vector data for intelligent, context-aware search and retrieval.

## Features

- 🔍 **Semantic Text Search** - Natural language queries automatically embedded and searched
- 🎯 **Filtered Search** - Combine semantic search with metadata filtering (username, filename, etc.)
- 🔑 **Keyword Search** - Find semantically similar content that contains specific keywords
- 📊 **Vector Operations** - Direct vector search, scrolling, counting, and collection management
- ⚡ **Local Embeddings** - Fast, private text embedding using FastEmbed (no external API calls)
- 🌐 **Remote Access** - Serve over HTTP for connection from Claude.ai and other MCP clients
- 🔧 **Easy Configuration** - Simple `.env` file setup with sensible defaults

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Connecting to Claude](#connecting-to-claude)
- [Available Tools](#available-tools)
- [Architecture](#architecture)
- [Development](#development)
- [Troubleshooting](#troubleshooting)

## Installation

### Prerequisites

- Rust 1.75 or higher
- Qdrant instance running (local or remote)
- Basic familiarity with vector databases

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd qdrant-mcp-server

# Build the server
cargo build --release

# Run the server
cargo run --release
```

On first run, the server will automatically create a `.env` file with default configuration.

## Quick Start

### 1. Start Qdrant

If you don't have Qdrant running, start it with Docker:

```bash
docker run -p 6334:6334 qdrant/qdrant
```

### 2. Configure the Server

The server will create a `.env` file on first run. Edit it to match your setup:

```env
# Server binding (use 0.0.0.0 for remote access)
HOST=127.0.0.1
PORT=8766

# Qdrant connection
QDRANT_URL=http://localhost:6334
QDRANT_COLLECTION=qc1

# Embedding model
EMBEDDING_MODEL=BAAI/bge-small-en-v1.5

# Logging
RUST_LOG=info,mcp_qdrant=debug
```

### 3. Start the Server

```bash
cargo run --release
```

You should see:

```
✅ Server is running!

📋 Available tools:
  • search_text - Natural language semantic search
  • filter_search - Search with metadata filters
  • keyword_search - Semantic search with keyword filtering
  ...

🔗 MCP endpoint: http://127.0.0.1:8766/mcp
```

## Configuration

### Embedding Models

Choose from several pre-trained FastEmbed models:

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| `BAAI/bge-small-en-v1.5` | Small | Fast | Good | Default, balanced |
| `BAAI/bge-base-en-v1.5` | Medium | Moderate | Better | Higher quality |
| `BAAI/bge-large-en-v1.5` | Large | Slower | Best | Maximum accuracy |
| `sentence-transformers/all-MiniLM-L6-v2` | Small | Fast | Good | Alternative option |

Set your preferred model in `.env`:

```env
EMBEDDING_MODEL=BAAI/bge-base-en-v1.5
```

### Network Configuration

For **local-only** access (Claude Desktop, local clients):
```env
HOST=127.0.0.1
```

For **remote access** (Claude.ai, network clients):
```env
HOST=0.0.0.0
```

⚠️ **Security Warning**: When exposing to the network, ensure proper firewall rules and consider adding authentication.

## Connecting to Claude

### Option 1: Claude Desktop (Local)

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "qdrant": {
      "command": "/path/to/qdrant-mcp-server",
      "env": {
        "QDRANT_URL": "http://localhost:6334",
        "QDRANT_COLLECTION": "qc1"
      }
    }
  }
}
```

### Option 2: Claude.ai (Remote via Custom Connector)

1. **Configure for remote access** in `.env`:
   ```env
   HOST=0.0.0.0
   PORT=8766
   ```

2. **Start the server**:
   ```bash
   cargo run --release
   ```

3. **Add Custom Connector in Claude.ai**:
   - Open [Claude.ai](https://claude.ai)
   - Go to Settings → Connectors
   - Click "Add custom connector"
   - Enter your server URL: `http://your-server-ip:8766/mcp`
   - Complete any authentication if configured

4. **Use the connector**:
   - Click the paperclip icon in any conversation
   - Select resources/prompts from your Qdrant server
   - Ask Claude to search your vector database

For detailed instructions, see the [MCP Remote Servers documentation](https://modelcontextprotocol.io/docs/develop/connect-remote-servers).

## Available Tools

### Core Search Tools

#### `search_text`
Natural language semantic search - **use this first!**

```
Search for: "machine learning papers about transformers"
```

**Parameters:**
- `query` (string) - Your natural language search query
- `limit` (number, default: 10) - Maximum results to return
- `with_payload` (boolean, default: true) - Include metadata

#### `filter_search`
Semantic search with metadata filtering

```
Search for: "project updates"
Filter by: username = "alice"
```

**Parameters:**
- `query` (string) - Search query
- `filter_field` (string) - Metadata field name (e.g., "username", "filename")
- `filter_value` (string) - Value to match
- `limit` (number, default: 10)

#### `keyword_search`
Semantic search that must contain specific keywords

```
Query: "database performance"
Must contain: "postgresql indexing"
```

**Parameters:**
- `query` (string) - Semantic search query
- `must_contain_keywords` (string) - Space-separated keywords that must appear
- `limit` (number, default: 10)

### Utility Tools

#### `scroll_points`
Paginate through all points in the collection

#### `count_points`
Get the total number of points

#### `search_vectors`
Advanced: Search using pre-computed embedding vectors

#### `get_collection_info`
View collection statistics and configuration

### Resources

#### `qdrant://collection`
Provides information about the connected Qdrant collection

### Prompts

#### `vector_search_assistant`
Get guidance on using the vector search capabilities

## Architecture

```
┌─────────────────────┐
│   MCP Client        │
│  (Claude, etc.)     │
└──────────┬──────────┘
           │ HTTP/MCP Protocol
           │
┌──────────▼──────────┐
│  Qdrant MCP Server  │
│  ┌───────────────┐  │
│  │  HTTP Server  │  │
│  │  (Axum)       │  │
│  └───────┬───────┘  │
│          │          │
│  ┌───────▼───────┐  │
│  │ MCP Handler   │  │
│  │ (Tools/Prompts)│ │
│  └───────┬───────┘  │
│          │          │
│  ┌───────▼───────┐  │
│  │   FastEmbed   │  │
│  │  (Local Model)│  │
│  └───────────────┘  │
└──────────┬──────────┘
           │ gRPC
           │
┌──────────▼──────────┐
│  Qdrant Database    │
│  (Vector Storage)   │
└─────────────────────┘
```

**Key Components:**

- **Axum HTTP Server**: Serves the MCP endpoint
- **MCP Handler**: Implements MCP protocol (tools, resources, prompts)
- **FastEmbed**: Local text embedding (no external API calls)
- **Qdrant Client**: Communicates with Qdrant vector database
- **Thread-Safe Design**: `Arc<Mutex<>>` ensures safe concurrent access

## Development

### Project Structure

```
src/
├── config/
│   └── mod.rs          # Configuration management
├── mcp_server/
│   ├── handler.rs      # MCP protocol handlers
│   ├── server.rs       # Core server logic and tools
│   ├── types.rs        # Request/response types
│   └── mod.rs
└── main.rs             # Entry point
```

### Adding New Tools

1. Define the argument struct in `types.rs`:
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MyToolArgs {
    pub param: String,
}
```

2. Add the tool method in `server.rs` inside the `#[tool_router]` impl block:
```rust
#[tool(description = "My new tool")]
pub async fn my_tool(
    &self,
    Parameters(args): Parameters<MyToolArgs>,
) -> Result<CallToolResult, McpError> {
    // Implementation
    Ok(CallToolResult::success(vec![Content::text("result")]))
}
```

### Running Tests

```bash
cargo test
```

### Logging

Control log levels via `RUST_LOG`:

```env
# In .env file
RUST_LOG=info,mcp_qdrant=debug

# Or via environment variable
RUST_LOG=debug cargo run
```

## Troubleshooting

### Server won't start

**"Failed to bind to address"**
- Port 8766 might be in use
- Change `PORT` in `.env` file
- Check with: `lsof -i :8766` (macOS/Linux)

**"Failed to connect to Qdrant"**
- Ensure Qdrant is running: `docker ps`
- Check `QDRANT_URL` in `.env`
- Verify network connectivity

### Model download fails

**"Failed to download embedding model"**
- Check internet connection (first run only)
- Models are cached in `~/.cache/fastembed`
- Try a different model in `.env`

### Search returns no results

**Empty results from search**
- Verify collection exists: Use `get_collection_info` tool
- Check collection name in `.env` matches your data
- Ensure points have been indexed in Qdrant
- Try increasing `limit` parameter

### Claude can't connect

**Remote connection fails**
- Verify `HOST=0.0.0.0` for network access
- Check firewall rules allow port 8766
- Confirm server is accessible: `curl http://your-ip:8766/mcp`
- Review server logs for errors

### Performance issues

**Slow embedding generation**
- Embeddings are generated sequentially (thread-safe)
- Consider using a smaller model (`bge-small`)
- For high concurrency, implement model pooling
- Check Qdrant query performance separately

## Security Considerations

When exposing the server remotely:

1. **Use HTTPS**: Place behind a reverse proxy (nginx, Caddy) with TLS
2. **Add Authentication**: Implement API keys or OAuth
3. **Firewall Rules**: Restrict access to known IP addresses
4. **Rate Limiting**: Prevent abuse of embedding/search endpoints
5. **Monitor Logs**: Watch for suspicious activity

Example nginx configuration:

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location /mcp {
        proxy_pass http://127.0.0.1:8766;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
    }
}
```

## Performance Tips

1. **Choose the right model**: Smaller models are faster but less accurate
2. **Optimize limits**: Don't fetch more results than needed
3. **Use filters**: Narrow searches with metadata filters
4. **Batch operations**: Group related queries when possible
5. **Monitor Qdrant**: Ensure proper indexing and resource allocation

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `cargo fmt` and `cargo clippy` pass
5. Submit a pull request

## License

MIT

## Acknowledgments

- Built with [rmcp](https://github.com/jlowin/rmcp) - Rust MCP SDK
- Powered by [FastEmbed](https://github.com/Anush008/fastembed-rs) - Fast, local embeddings
- Uses [Qdrant](https://qdrant.tech/) - High-performance vector database
- Implements the [Model Context Protocol](https://modelcontextprotocol.io/)

## Support

- Report issues: [GitHub Issues](your-repo/issues)
- MCP Documentation: https://modelcontextprotocol.io/
- Qdrant Documentation: https://qdrant.tech/documentation/

---



