use std::fs;
use std::path::Path;

const DEFAULT_PORT: u16 = 8766;
const DEFAULT_COLLECTION: &str = "qc1";
const DEFAULT_QDRANT_URL: &str = "http://localhost:6334";
const DEFAULT_EMBEDDING_MODEL: &str = "BAAI/bge-small-en-v1.5";
const ENV_FILE: &str = ".env";

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub qdrant_url: String,
    pub collection_name: String,
    pub embedding_model: String,
}

impl ServerConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);

        let qdrant_url =
            std::env::var("QDRANT_URL").unwrap_or_else(|_| DEFAULT_QDRANT_URL.to_string());

        let collection_name =
            std::env::var("QDRANT_COLLECTION").unwrap_or_else(|_| DEFAULT_COLLECTION.to_string());

        let embedding_model = std::env::var("EMBEDDING_MODEL")
            .unwrap_or_else(|_| DEFAULT_EMBEDDING_MODEL.to_string());

        Ok(Self {
            port,
            qdrant_url,
            collection_name,
            embedding_model,
        })
    }
}

pub fn ensure_env_file() -> anyhow::Result<()> {
    let env_path = Path::new(ENV_FILE);

    if !env_path.exists() {
        let default_content = format!(
            "# MCP Qdrant Server Configuration\n\
             PORT={}\n\
             QDRANT_URL={}\n\
             QDRANT_COLLECTION={}\n\
             EMBEDDING_MODEL={}\n\
             RUST_LOG=info,mcp_qdrant=debug\n",
            DEFAULT_PORT, DEFAULT_QDRANT_URL, DEFAULT_COLLECTION, DEFAULT_EMBEDDING_MODEL
        );
        fs::write(env_path, default_content)?;
        println!("📝 Created {} with default configuration", ENV_FILE);
    }

    Ok(())
}
