
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod mcp_server;
mod middleware;

use config::{ServerConfig, ensure_env_file};
use mcp_server::QdrantMCPServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mcp_qdrant=debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    ensure_env_file()?;
    let config = ServerConfig::from_env()?;

    println!("🚀 Starting MCP Qdrant Server");
    println!();
    println!("🔧 Configuration:");
    println!("  📡 Binding to: {}:{}", config.host, config.port);
    println!("  🔗 MCP endpoint: http://{}:{}/mcp", config.host, config.port);
    println!("  🗄️  Qdrant collection: {}", config.collection_name);
    println!("  🌐 Qdrant URL: {}", config.qdrant_url);
    println!("  🤖 Embedding model: {}", config.embedding_model);
    
    if config.auth_token.is_some() {
        println!("  🔐 Authentication: ENABLED (Bearer token required)");
    } else {
        println!("  ⚠️  Authentication: DISABLED (set MCP_AUTH_TOKEN in .env to enable)");
    }
    println!();

    println!("⚠️  Prerequisites:");
    println!("   • Qdrant must be running at {}", config.qdrant_url);
    println!("   • Collection '{}' will be auto-created with sample data if missing", config.collection_name);
    println!();

    let bind_address = format!("{}:{}", config.host, config.port);
    let server_config = config.clone();

    // Initialize MCP server (includes Qdrant connection checks)
    let mcp_server = QdrantMCPServer::new(
        server_config.qdrant_url.clone(),
        server_config.collection_name.clone(),
        server_config.embedding_model.clone(),
    )
    .await?;

    let service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // Create router with authentication middleware
    let auth_token = config.auth_token.clone();
    let router = axum::Router::new()
        .nest_service("/mcp", service)
        .layer(axum::middleware::from_fn(move |req, next| {
            crate::middleware::auth_middleware(auth_token.clone(), req, next)
        }));

    let addr: SocketAddr = bind_address.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("✅ Server is running!");
    println!();
    
    if config.auth_token.is_some() {
        println!("🔐 Authentication is ENABLED");
        println!("   Clients must include: Authorization: Bearer <token>");
        println!("   Token is set in MCP_AUTH_TOKEN environment variable");
        println!();
    }
    
    println!("📋 Available tools:");
    println!("  • search_text - Natural language semantic search");
    println!("  • filter_search - Search with metadata filters");
    println!("  • keyword_search - Semantic search with keyword filtering");
    println!("  • search_vectors - Search with pre-computed vectors");
    println!("  • scroll_points - Paginate through points");
    println!("  • count_points - Count total points");
    println!("  • get_collection_info - Collection statistics");
    println!();
    println!("📚 Available resources:");
    println!("  • qdrant://collection - Collection information");
    println!();
    println!("🔗 Connect from:");
    println!("  • Claude Desktop: Add to config file");
    println!("  • Claude.ai: Use Custom Connector with http://{}:{}/mcp", config.host, config.port);
    println!("  • MCP Inspector: http://{}:{}/mcp", config.host, config.port);
    println!();
    println!("Press Ctrl+C to stop the server...");
    println!();

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen for ctrl-c");
            println!("\n🛑 Shutting down server...");
        })
        .await?;

    println!("✅ Server stopped");
    Ok(())
}
