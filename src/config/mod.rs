use std::fs;
use std::path::Path;

const DEFAULT_PORT: u16 = 8766;
const DEFAULT_HOST: &str = "127.0.0.1"; 
const DEFAULT_COLLECTION: &str = "qc1";
const DEFAULT_QDRANT_URL: &str = "http://localhost:6334";
const DEFAULT_EMBEDDING_MODEL: &str = "BAAI/bge-m3";
const ENV_FILE: &str = ".env";

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub qdrant_url: String,
    pub collection_name: String,
    pub embedding_model: String,
    pub auth_token: Option<String>,
}

impl ServerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        
        let host = std::env::var("HOST")
            .unwrap_or_else(|_| DEFAULT_HOST.to_string());

        let qdrant_url =
            std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string());

        let collection_name =
            std::env::var("QDRANT_COLLECTION").unwrap_or_else(|_| DEFAULT_COLLECTION.to_string());

        let embedding_model = std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.to_string());

        let auth_token = std::env::var("MCP_AUTH_TOKEN").ok();

        Ok(Self {
            port,
            host,
            qdrant_url,
            collection_name,
            embedding_model,
            auth_token,
        })
    }
}

pub fn ensure_env_file() -> anyhow::Result<()> {
    let env_path = Path::new(ENV_FILE);
    
    if !env_path.exists() {
        let default_content = format!(
            "# MCP Qdrant Server Configuration\n\
            # HOST: Use 127.0.0.1 for local only, 0.0.0.0 for network access\n\
             HOST={}\n\
             PORT={}\n\
             QDRANT_URL={}\n\
             QDRANT_COLLECTION={}\n\
             EMBEDDING_MODEL={}\n\
             RUST_LOG=info,mcp_qdrant=debug\n\
             \n\
             # Authentication (optional - if not set, server allows unauthenticated access)\n\
             # Generate with: openssl rand -base64 32\n\
             # MCP_AUTH_TOKEN=your-token-here\n",
            DEFAULT_HOST,
            DEFAULT_PORT,
            DEFAULT_QDRANT_URL,
            DEFAULT_COLLECTION,
            DEFAULT_EMBEDDING_MODEL
        );
        fs::write(env_path, default_content)?;
        println!("📝 Created {} with default configuration", ENV_FILE);
    }
    
    Ok(())
}
