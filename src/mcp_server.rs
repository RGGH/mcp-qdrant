// src/mcp_server.rs

use std::sync::Arc;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{SearchPoints, ScrollPoints, CountPoints, Filter, Condition, FieldCondition, Match};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::*,
    prompt, prompt_handler, prompt_router, schemars,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use tokio::sync::Mutex;
use fastembed::TextEmbedding;

// ==================== Request/Response Types ====================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchTextArgs {
    /// The text query to search for semantically similar content
    pub query: String,
    
    #[serde(default = "default_limit")]
    pub limit: u64,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchVectorsArgs {
    pub vector: Vec<f64>,
    
    #[serde(default = "default_limit")]
    pub limit: u64,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ScrollPointsArgs {
    #[serde(default = "default_limit_u32")]
    pub limit: u32,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CountPointsArgs {
    #[serde(default = "default_exact")]
    pub exact: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FilterSearchArgs {
    /// The text query to search for
    pub query: String,
    /// Field name to filter on (e.g., "username", "filename")
    pub filter_field: String,
    /// Value to match in the filter field
    pub filter_value: String,
    
    #[serde(default = "default_limit")]
    pub limit: u64,
}

fn default_limit() -> u64 { 10 }
fn default_limit_u32() -> u32 { 10 }
fn default_with_payload() -> bool { true }
fn default_exact() -> bool { true }

// ==================== Qdrant MCP Server ====================

#[derive(Clone)]
pub struct QdrantMCPServer {
    client: Arc<Mutex<Qdrant>>,
    embedding_model: Arc<Mutex<TextEmbedding>>,
    collection_name: String,
    pub tool_router: ToolRouter<QdrantMCPServer>,
    pub prompt_router: PromptRouter<QdrantMCPServer>,
}

#[tool_router]
impl QdrantMCPServer {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = Qdrant::from_url("http://localhost:6334").build()?;
        
        // Initialize the same embedding model used in your Loader
        tracing::info!("Initializing FastEmbed text embedding model...");
        let embedding_model = TextEmbedding::try_new(Default::default())?;
        
        tracing::info!(
            collection = "qc1",
            url = "http://localhost:6334",
            "initializing_qdrant_mcp_server"
        );

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            embedding_model: Arc::new(Mutex::new(embedding_model)),
            collection_name: "qc1".to_string(),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }

    /// Helper function to generate embeddings for a text query
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, McpError> {
        let mut model = self.embedding_model.lock().await;
        
        model.embed(vec![text], None)
            .map_err(|e| McpError::internal_error(
                format!("Failed to generate embedding: {}", e),
                None
            ))?
            .into_iter()
            .next()
            .ok_or_else(|| McpError::internal_error(
                "No embedding generated".to_string(),
                None
            ))
    }

    // ==================== Tools ====================

    #[tool(description = "Search for semantically similar text content in the Qdrant collection. This is the primary search method - provide natural language queries.")]
    pub async fn search_text(
        &self,
        Parameters(args): Parameters<SearchTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Generate embedding for the query text
        let query_vector = self.embed_text(&args.query).await?;
        
        let client = self.client.lock().await;
        
        let search_result = client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit: args.limit,
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(
                format!("Search failed: {}", e),
                None
            ))?;

        let results = json!({
            "query": args.query,
            "count": search_result.result.len(),
            "results": search_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&results).unwrap())
        ]))
    }

    #[tool(description = "Search using a pre-computed vector (advanced use - most users should use search_text instead)")]
    pub async fn search_vectors(
        &self,
        Parameters(args): Parameters<SearchVectorsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = self.client.lock().await;
        
        let search_result = client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: args.vector.iter().map(|&v| v as f32).collect(),
                limit: args.limit,
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(
                format!("Search failed: {}", e),
                None
            ))?;

        let results = json!({
            "count": search_result.result.len(),
            "results": search_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&results).unwrap())
        ]))
    }

    #[tool(description = "Scroll through all points in the collection with pagination support")]
    pub async fn scroll_points(
        &self,
        Parameters(args): Parameters<ScrollPointsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = self.client.lock().await;

        let scroll_result = client
            .scroll(ScrollPoints {
                collection_name: self.collection_name.clone(),
                limit: Some(args.limit),
                offset: args.offset.map(|id| id.into()),
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(
                format!("Scroll failed: {}", e),
                None
            ))?;

        let results = json!({
            "count": scroll_result.result.len(),
            "points": scroll_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "payload": p.payload,
                "vector": p.vectors.as_ref().map(|v| format!("{:?}", v)),
            })).collect::<Vec<_>>(),
            "next_offset": scroll_result.next_page_offset.map(|o| format!("{:?}", o))
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&results).unwrap())
        ]))
    }

    #[tool(description = "Count the total number of points in the collection")]
    pub async fn count_points(
        &self,
        Parameters(args): Parameters<CountPointsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let client = self.client.lock().await;

        let count_result = client
            .count(CountPoints {
                collection_name: self.collection_name.clone(),
                exact: Some(args.exact),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(
                format!("Count failed: {}", e),
                None
            ))?;

        let result = json!({
            "count": count_result.result.map(|r| r.count).unwrap_or(0)
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&result).unwrap())
        ]))
    }

    #[tool(description = "Search for semantically similar text with metadata filtering - find content matching your query AND specific criteria (e.g., by username or filename)")]
    pub async fn filter_search(
        &self,
        Parameters(args): Parameters<FilterSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        // Generate embedding for the query text
        let query_vector = self.embed_text(&args.query).await?;
        
        let client = self.client.lock().await;

        let filter = Filter {
            must: vec![Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: args.filter_field.clone(),
                        r#match: Some(Match {
                            match_value: Some(qdrant_client::qdrant::r#match::MatchValue::Keyword(
                                args.filter_value.clone()
                            )),
                        }),
                        ..Default::default()
                    }
                )),
            }],
            ..Default::default()
        };

        let search_result = client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit: args.limit,
                filter: Some(filter),
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(
                format!("Filtered search failed: {}", e),
                None
            ))?;

        let results = json!({
            "query": args.query,
            "count": search_result.result.len(),
            "filter": {
                "field": args.filter_field,
                "value": args.filter_value
            },
            "results": search_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&results).unwrap())
        ]))
    }

    #[tool(description = "Get collection information and statistics")]
    pub async fn get_collection_info(&self) -> Result<CallToolResult, McpError> {
        let client = self.client.lock().await;
        
        let collection_info = client
            .collection_info(&self.collection_name)
            .await
            .map_err(|e| McpError::internal_error(
                format!("Failed to get collection info: {}", e),
                None
            ))?;

        let info = json!({
            "collection_name": self.collection_name,
            "status": format!("{:?}", collection_info.result.as_ref().map(|r| r.status)),
            "segments_count": collection_info.result.as_ref().map(|r| r.segments_count),
            "points_count": collection_info.result.as_ref().and_then(|r| r.points_count),
            "indexed_vectors": collection_info.result.as_ref().and_then(|r| r.indexed_vectors_count),
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&info).unwrap())
        ]))
    }
}

// ==================== Prompts ====================

#[prompt_router]
impl QdrantMCPServer {
    #[prompt(name = "vector_search_assistant")]
    pub async fn vector_search_assistant(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "I'm your Qdrant vector search assistant. I can help you search through your vector database using natural language queries. Just tell me what you're looking for!",
            ),
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Help me find relevant information in my document collection.",
            ),
        ];

        Ok(GetPromptResult {
            description: Some("Semantic search assistance for your Qdrant collection".to_string()),
            messages,
        })
    }
}

// ==================== MCP Server Handler ====================

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
            instructions: Some(
                "Qdrant MCP Server - Semantic vector search with FastEmbed.\n\n\
                Connected to: http://localhost:6334\n\
                Collection: qc1\n\n\
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
                • qdrant://collection - Collection information".to_string()
            ),
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new(
                    "qdrant://collection",
                    format!("Qdrant Collection: {}", self.collection_name)
                ).no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParam { uri }: ReadResourceRequestParam,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match uri.as_str() {
            "qdrant://collection" => {
                match self.get_collection_info().await {
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
                }
            }
            _ => Err(McpError::resource_not_found(
                "Resource not found",
                Some(json!({ "uri": uri })),
            )),
        }
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            next_cursor: None,
            resource_templates: Vec::new(),
            meta: None,
        })
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("qdrant_mcp_server_initialized");
        Ok(self.get_info())
    }
}
