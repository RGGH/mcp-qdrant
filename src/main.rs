use rmcp::transport::streamable_http_server::{
    StreamableHttpService,
    session::local::LocalSessionManager
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use std::path::Path;
use std::fs;
use mcp_server::QdrantMCPServer;

mod mcp_server;

const DEFAULT_PORT: u16 = 8766;
const ENV_FILE: &str = ".env";

/// Load port from .env file, creating it with default if it doesn't exist
fn load_or_create_port() -> anyhow::Result<u16> {
    let env_path = Path::new(ENV_FILE);
    
    // If .env doesn't exist, create it with default port
    if !env_path.exists() {
        let default_content = format!("PORT={}\n", DEFAULT_PORT);
        fs::write(env_path, default_content)?;
        println!("📝 Created {} with default port {}", ENV_FILE, DEFAULT_PORT);
        return Ok(DEFAULT_PORT);
    }
    
    // Load .env file
    dotenvy::dotenv().ok();
    
    // Try to read PORT from environment
    match std::env::var("PORT") {
        Ok(port_str) => {
            match port_str.parse::<u16>() {
                Ok(port) => {
                    println!("📖 Loaded port {} from {}", port, ENV_FILE);
                    Ok(port)
                }
                Err(_) => {
                    eprintln!("⚠️  Invalid PORT value in {}: '{}'. Using default {}", 
                        ENV_FILE, port_str, DEFAULT_PORT);
                    Ok(DEFAULT_PORT)
                }
            }
        }
        Err(_) => {
            println!("⚠️  No PORT found in {}. Using default {}", ENV_FILE, DEFAULT_PORT);
            Ok(DEFAULT_PORT)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mcp_qdrant=debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // Load port from .env
    let port = load_or_create_port()?;
    let bind_address = format!("127.0.0.1:{}", port);
    
    println!("📡 Binding to: {}", bind_address);
    println!("🔗 MCP endpoint: http://{}/mcp", bind_address);
    println!("🗄️  Qdrant collection: qc1");
    println!("🌐 Qdrant URL: http://localhost:6334");
    println!();
    
    // Create the HTTP service - wrap in Ok() for the factory
    let service = StreamableHttpService::new(
        || {
            Ok(tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    QdrantMCPServer::new()
                        .await
                        .expect("Failed to initialize Qdrant MCP Server")
                })
            }))
        },
        LocalSessionManager::default().into(),
        Default::default(),
    );
    
    // Create axum router and mount the MCP service at /mcp
    let router = axum::Router::new()
        .nest_service("/mcp", service);
    
    // Parse the bind address
    let addr: SocketAddr = bind_address.parse()?;
    
    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("✅ Server is running!");
    println!();
    println!("📋 Available tools:");
    println!("  • search_vectors - Semantic similarity search");
    println!("  • scroll_points - Paginate through points");
    println!("  • count_points - Count total points");
    println!("  • filter_search - Search with metadata filters");
    println!("  • get_collection_info - Collection statistics");
    println!();
    println!("📚 Available resources:");
    println!("  • qdrant://collection - Collection information");
    println!();
    println!("Press Ctrl+C to stop the server...");
    println!();
    
    // Serve with graceful shutdown
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
