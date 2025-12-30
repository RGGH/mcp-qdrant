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

![Architecture](architecture.svg)

<svg viewBox="0 0 800 900" xmlns="http://www.w3.org/2000/svg">
  <!-- Define styles -->
  <defs>
    <style>
      .component-box { fill: #ffffff; stroke: #2c3e50; stroke-width: 2; }
      .client-box { fill: #3498db; stroke: #2980b9; stroke-width: 2; }
      .server-box { fill: #e74c3c; stroke: #c0392b; stroke-width: 2; }
      .embed-box { fill: #f39c12; stroke: #d68910; stroke-width: 2; }
      .db-box { fill: #27ae60; stroke: #229954; stroke-width: 2; }
      .inner-box { fill: #ecf0f1; stroke: #95a5a6; stroke-width: 1.5; }
      .text-title { fill: #ffffff; font-family: Arial, sans-serif; font-size: 18px; font-weight: bold; }
      .text-subtitle { fill: #ffffff; font-family: Arial, sans-serif; font-size: 14px; }
      .text-inner { fill: #2c3e50; font-family: Arial, sans-serif; font-size: 14px; font-weight: bold; }
      .text-label { fill: #7f8c8d; font-family: Arial, sans-serif; font-size: 12px; font-style: italic; }
      .arrow { stroke: #34495e; stroke-width: 2; fill: none; marker-end: url(#arrowhead); }
      .protocol-label { fill: #7f8c8d; font-family: Arial, sans-serif; font-size: 13px; font-weight: bold; }
    </style>
    <marker id="arrowhead" markerWidth="10" markerHeight="10" refX="9" refY="3" orient="auto">
      <polygon points="0 0, 10 3, 0 6" fill="#34495e" />
    </marker>
  </defs>

  <!-- Title -->
  <text x="400" y="30" text-anchor="middle" style="font-family: Arial, sans-serif; font-size: 24px; font-weight: bold; fill: #2c3e50;">
    Qdrant MCP Server Architecture
  </text>

  <!-- MCP Client -->
  <rect x="250" y="70" width="300" height="80" rx="8" class="client-box"/>
  <text x="400" y="100" text-anchor="middle" class="text-title">MCP Client</text>
  <text x="400" y="125" text-anchor="middle" class="text-subtitle">(Claude, Desktop Apps, etc.)</text>

  <!-- Arrow 1: Client to Server -->
  <line x1="400" y1="150" x2="400" y2="200" class="arrow"/>
  <text x="420" y="180" class="protocol-label">HTTP/MCP</text>
  <text x="420" y="195" class="protocol-label">Protocol</text>

  <!-- Main Server Container -->
  <rect x="150" y="210" width="500" height="380" rx="8" class="server-box"/>
  <text x="400" y="240" text-anchor="middle" class="text-title">Qdrant MCP Server</text>

  <!-- HTTP Server (Axum) -->
  <rect x="175" y="260" width="450" height="60" rx="6" class="inner-box"/>
  <text x="400" y="285" text-anchor="middle" class="text-inner">HTTP Server</text>
  <text x="400" y="305" text-anchor="middle" class="text-label">(Axum)</text>

  <!-- Internal Arrow 1 -->
  <line x1="400" y1="320" x2="400" y2="345" class="arrow"/>

  <!-- MCP Handler -->
  <rect x="175" y="345" width="450" height="60" rx="6" class="inner-box"/>
  <text x="400" y="370" text-anchor="middle" class="text-inner">MCP Handler</text>
  <text x="400" y="390" text-anchor="middle" class="text-label">(Tools, Resources, Prompts)</text>

  <!-- Internal Arrow 2 - Split -->
  <line x1="400" y1="405" x2="400" y2="430" class="arrow"/>
  <line x1="400" y1="430" x2="280" y2="460" class="arrow"/>
  <line x1="400" y1="430" x2="520" y2="460" class="arrow"/>

  <!-- FastEmbed Box -->
  <rect x="175" y="460" width="200" height="100" rx="6" class="embed-box"/>
  <text x="275" y="495" text-anchor="middle" class="text-title">FastEmbed</text>
  <text x="275" y="520" text-anchor="middle" class="text-subtitle">Text Embedding</text>
  <text x="275" y="540" text-anchor="middle" class="text-label">Local Model</text>
  <text x="275" y="555" text-anchor="middle" class="text-label">Arc&lt;Mutex&lt;&gt;&gt;</text>

  <!-- Qdrant Client Box -->
  <rect x="425" y="460" width="200" height="100" rx="6" class="inner-box"/>
  <text x="525" y="495" text-anchor="middle" class="text-inner">Qdrant Client</text>
  <text x="525" y="520" text-anchor="middle" class="text-label">Search, Filter,</text>
  <text x="525" y="535" text-anchor="middle" class="text-label">Scroll, Count</text>
  <text x="525" y="550" text-anchor="middle" class="text-label">Operations</text>

  <!-- Arrow out of server box -->
  <line x1="525" y1="590" x2="525" y2="650" class="arrow"/>
  <text x="545" y="625" class="protocol-label">gRPC</text>

  <!-- Qdrant Database -->
  <rect x="325" y="660" width="400" height="180" rx="8" class="db-box"/>
  <text x="525" y="695" text-anchor="middle" class="text-title">Qdrant Database</text>
  <text x="525" y="720" text-anchor="middle" class="text-subtitle">(Vector Storage)</text>
  
  <!-- Database Features -->
  <text x="370" y="755" class="text-subtitle">✓ Vector Indexing</text>
  <text x="370" y="780" class="text-subtitle">✓ Similarity Search</text>
  <text x="370" y="805" class="text-subtitle">✓ Metadata Filtering</text>
  <text x="370" y="830" class="text-subtitle">✓ Scalable Storage</text>

  <!-- Side annotations -->
  <text x="50" y="110" class="text-label" style="font-size: 11px;">User</text>
  <text x="50" y="125" class="text-label" style="font-size: 11px;">Interface</text>
  
  <text x="50" y="400" class="text-label" style="font-size: 11px;">Request</text>
  <text x="50" y="415" class="text-label" style="font-size: 11px;">Processing</text>
  
  <text x="50" y="750" class="text-label" style="font-size: 11px;">Data</text>
  <text x="50" y="765" class="text-label" style="font-size: 11px;">Storage</text>

  <!-- Legend -->
  <rect x="50" y="860" width="700" height="30" rx="4" fill="#f8f9fa" stroke="#dee2e6" stroke-width="1"/>
  <text x="70" y="880" style="font-family: Arial, sans-serif; font-size: 12px; fill: #495057;">
    <tspan font-weight="bold">Flow:</tspan> Client sends MCP request → Server embeds text → Searches Qdrant → Returns results
  </text>
</svg>


