use crate::mcp_server::server::QdrantMCPServer;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler, prompt_handler, tool_handler};
use serde_json::json;

// Apply macros to generate handler implementations
#[tool_handler]
#[prompt_handler]
impl ServerHandler for QdrantMCPServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(format!(
                "Qdrant MCP Server - Semantic vector search with FastEmbed.\n\n\
                Connected to: {}\n\
                Collection: {}\n\
                Embedding Model: {}\n\n\
                PRIMARY TOOLS:\n\
                • search_text - Natural language semantic search (USE THIS FIRST)\n\
                • filter_search - Semantic search with metadata filters (username, filename, etc.)\n\n\
                ADDITIONAL TOOLS:\n\
                • scroll_points - Paginate through all points\n\
                • count_points - Get total number of points\n\
                • search_vectors - Advanced: search with pre-computed vectors\n\
                • get_collection_info - View collection statistics\n\n\
                USAGE:\n\
                Most users should use 'search_text' with natural language queries.\n\
                The server automatically handles text embedding using FastEmbed.\n\n\
                METADATA FIELDS:\n\
                • username - Filter by document owner\n\
                • filename - Filter by source document\n\
                • topics - Document topics\n\
                • entities - Named entities extracted from text\n\n\
                Prompts:\n\
                • vector_search_assistant - Get search guidance\n\n\
                Resources:\n\
                • qdrant://collection - Collection information",
                self.qdrant_url, self.collection_name, self.embedding_model_name
            )),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new(
                    "qdrant://collection",
                    format!("Qdrant Collection: {}", self.collection_name),
                )
                .no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri,meta: _ }: ReadResourceRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "qdrant://collection" => match self.get_collection_info().await {
                Ok(info_result) => {
                    let mut content_text = String::new();
                    for c in &info_result.content {
                        if let RawContent::Text(text_content) = &c.raw {
                            content_text.push_str(&text_content.text);
                        }
                    }

                    Ok(ReadResourceResult {
                        contents: vec![ResourceContents::text(&content_text, uri)],
                    })
                }
                Err(e) => Err(e),
            },
            _ => Err(McpError::resource_not_found(
                "Resource not found",
                Some(json!({ "uri": uri })),
            )),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
            meta: None,
        })
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!(
            collection = %self.collection_name,
            url = %self.qdrant_url,
            model = %self.embedding_model_name,
            "qdrant_mcp_server_initialized"
        );
        Ok(self.get_info())
    }
}
