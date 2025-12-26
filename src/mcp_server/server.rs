use std::sync::Arc;
use qdrant_client::Qdrant;
use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use tokio::sync::Mutex;
use rmcp::{ErrorData as McpError, tool_router, prompt_router, tool, prompt, RoleServer};
use rmcp::handler::server::router::{prompt::PromptRouter, tool::ToolRouter};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, GetPromptResult, PromptMessage, PromptMessageRole};
use rmcp::service::RequestContext;
use serde_json::json;
use qdrant_client::qdrant::{SearchPoints, ScrollPoints, CountPoints, Filter, Condition, FieldCondition, Match};
use crate::mcp_server::types::*;

#[derive(Clone)]
pub struct QdrantMCPServer {
    pub client: Arc<Mutex<Qdrant>>,
    pub embedding_model: Arc<Mutex<TextEmbedding>>,
    pub collection_name: String,
    pub qdrant_url: String,
    pub embedding_model_name: String,
    pub tool_router: ToolRouter<Self>,
    pub prompt_router: PromptRouter<Self>,
}

// ALL tools and prompts MUST be in this impl block with the macros
#[tool_router]
#[prompt_router]
impl QdrantMCPServer {
pub async fn new(
    qdrant_url: String,
    collection_name: String,
    embedding_model_name: String,
) -> anyhow::Result<Self> {
        
        tracing::info!(
            url = %qdrant_url,
            collection = %collection_name,
            model = %embedding_model_name,
            "initializing_qdrant_mcp_server"
        );
        
        let client = Qdrant::from_url(&qdrant_url).build()?;
        
        let model = match embedding_model_name.as_str() {
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "BAAI/bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
            "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            _ => {
                tracing::warn!(
                    model = %embedding_model_name,
                    "Unknown embedding model, falling back to BGESmallENV15"
                );
                EmbeddingModel::BGESmallENV15
            }
        };
        
        tracing::info!(model = ?model, "initializing_fastembed_model");
        let embedding_model = TextEmbedding::try_new(
            InitOptions::new(model).with_show_download_progress(true)
        )?;
        
        tracing::info!("fastembed_model_initialized");

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            embedding_model: Arc::new(Mutex::new(embedding_model)),
            collection_name: collection_name.clone(),
            qdrant_url: qdrant_url.clone(),
            embedding_model_name,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, McpError> {
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

    // TOOLS - all must be in this impl block
    #[tool(description = "Search for semantically similar text content in the Qdrant collection. This is the primary search method - provide natural language queries.")]
    pub async fn search_text(
        &self,
        Parameters(args): Parameters<SearchTextArgs>,
    ) -> Result<CallToolResult, McpError> {
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
            "qdrant_url": self.qdrant_url,
            "embedding_model": self.embedding_model_name,
            "status": format!("{:?}", collection_info.result.as_ref().map(|r| r.status)),
            "segments_count": collection_info.result.as_ref().map(|r| r.segments_count),
            "points_count": collection_info.result.as_ref().and_then(|r| r.points_count),
            "indexed_vectors": collection_info.result.as_ref().and_then(|r| r.indexed_vectors_count),
        });

        Ok(CallToolResult::success(vec![
            Content::text(serde_json::to_string_pretty(&info).unwrap())
        ]))
    }

    // PROMPTS - must also be in this impl block
    #[prompt(name = "vector_search_assistant")]
    pub async fn vector_search_assistant(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        let messages = vec![
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                format!(
                    "I'm your Qdrant vector search assistant for the '{}' collection. \
                    I can help you search through your vector database using natural language queries. \
                    Just tell me what you're looking for!",
                    self.collection_name
                ),
            ),
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Help me find relevant information in my document collection.",
            ),
        ];

        Ok(GetPromptResult {
            description: Some(format!(
                "Semantic search assistance for your '{}' Qdrant collection",
                self.collection_name
            )),
            messages,
        })
    }
}