// ============================================================================
// ./src/mcp_server/server.rs
// ============================================================================
use crate::mcp_server::types::*;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    Condition, CountPoints, CreateCollectionBuilder, Distance, FieldCondition, Filter, Match,
    PointStruct, ScrollPoints, SearchPoints, VectorParamsBuilder,
};
use qdrant_client::qdrant::UpsertPointsBuilder;
use qdrant_client::qdrant::Value;
use rmcp::handler::server::router::{prompt::PromptRouter, tool::ToolRouter};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, GetPromptResult, PromptMessage, PromptMessageRole};
use rmcp::service::RequestContext;
use rmcp::{prompt, prompt_router, tool, tool_router, ErrorData as McpError, RoleServer};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct QdrantMCPServer {
    pub client: Arc<Qdrant>,
    pub embedding_model: Arc<Mutex<TextEmbedding>>,
    pub collection_name: String,
    pub qdrant_url: String,
    pub embedding_model_name: String,
    pub tool_router: ToolRouter<Self>,
    pub prompt_router: PromptRouter<Self>,
}

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

        // Create client
        let client = Qdrant::from_url(&qdrant_url).build()?;

        // Check Qdrant connection
        println!("🔍 Checking Qdrant connection...");
        match client.health_check().await {
            Ok(_) => {
                println!("✅ Successfully connected to Qdrant at {}", qdrant_url);
                tracing::info!("qdrant_connection_successful");
            }
            Err(e) => {
                eprintln!("❌ Failed to connect to Qdrant: {}", e);
                eprintln!();
                eprintln!("💡 Please start Qdrant first:");
                eprintln!("   docker run -p 6334:6334 qdrant/qdrant");
                eprintln!();
                eprintln!(
                    "   Or check that QDRANT_URL in .env is correct: {}",
                    qdrant_url
                );
                tracing::error!(error = %e, "qdrant_connection_failed");
                return Err(anyhow::anyhow!("Failed to connect to Qdrant: {}", e));
            }
        }

        let model = match embedding_model_name.as_str() {
            // BGE models
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "BAAI/bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
            "BAAI/bge-small-zh-v1.5" => EmbeddingModel::BGESmallZHV15,
            "BAAI/bge-large-zh-v1.5" => EmbeddingModel::BGELargeZHV15,
            "BAAI/bge-m3" => EmbeddingModel::BGEM3,

            // Sentence Transformers models
            "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            "sentence-transformers/all-MiniLM-L12-v2" => EmbeddingModel::AllMiniLML12V2,
            "sentence-transformers/all-mpnet-base-v2" => EmbeddingModel::AllMpnetBaseV2,
            "sentence-transformers/paraphrase-MiniLM-L12-v2" => {
                EmbeddingModel::ParaphraseMLMiniLML12V2
            }
            "sentence-transformers/paraphrase-multilingual-mpnet-base-v2" => {
                EmbeddingModel::ParaphraseMLMpnetBaseV2
            }

            // Nomic models
            "nomic-ai/nomic-embed-text-v1" => EmbeddingModel::NomicEmbedTextV1,
            "nomic-ai/nomic-embed-text-v1.5" => EmbeddingModel::NomicEmbedTextV15,

            // Multilingual E5 models
            "intfloat/multilingual-e5-small" => EmbeddingModel::MultilingualE5Small,
            "intfloat/multilingual-e5-base" => EmbeddingModel::MultilingualE5Base,
            "intfloat/multilingual-e5-large" => EmbeddingModel::MultilingualE5Large,

            // MixedBread model
            "mixedbread-ai/mxbai-embed-large-v1" => EmbeddingModel::MxbaiEmbedLargeV1,

            // Alibaba GTE models
            "Alibaba-NLP/gte-base-en-v1.5" => EmbeddingModel::GTEBaseENV15,
            "Alibaba-NLP/gte-large-en-v1.5" => EmbeddingModel::GTELargeENV15,



            // Jina models
            "jinaai/jina-embeddings-v2-base-code" => EmbeddingModel::JinaEmbeddingsV2BaseCode,
            "jinaai/jina-embeddings-v2-base-en" => EmbeddingModel::JinaEmbeddingsV2BaseEN,

            // Google Gemma model
            "google/embeddinggemma-300m" => EmbeddingModel::EmbeddingGemma300M,

            // Snowflake Arctic models
            "snowflake/snowflake-arctic-embed-xs" => EmbeddingModel::SnowflakeArcticEmbedXS,
            "snowflake/snowflake-arctic-embed-s" => EmbeddingModel::SnowflakeArcticEmbedS,
            "snowflake/snowflake-arctic-embed-m" => EmbeddingModel::SnowflakeArcticEmbedM,
            "snowflake/snowflake-arctic-embed-m-long" => EmbeddingModel::SnowflakeArcticEmbedMLong,
            "snowflake/snowflake-arctic-embed-l" => EmbeddingModel::SnowflakeArcticEmbedL,

            // Quantized versions
            "BAAI/bge-small-en-v1.5-q" | "BAAI/bge-small-en-v1.5Q" => {
                EmbeddingModel::BGESmallENV15Q
            }
            "BAAI/bge-base-en-v1.5-q" | "BAAI/bge-base-en-v1.5Q" => EmbeddingModel::BGEBaseENV15Q,
            "sentence-transformers/all-MiniLM-L6-v2-q" | "sentence-transformers/all-MiniLM-L6-v2Q" => {
                EmbeddingModel::AllMiniLML6V2Q
            }

            _ => {
                tracing::warn!(
                    model = %embedding_model_name,
                    "Unknown embedding model, falling back to AllMiniLML12V2"
                );
                eprintln!(
                    "⚠️  Unknown model '{}', using default AllMiniLML12V2",
                    embedding_model_name
                );
                EmbeddingModel::AllMiniLML12V2
            }
        };

        println!(
            "🤖 Loading embedding model: {}...",
            embedding_model_name
        );
        tracing::info!(model = ?model, "initializing_fastembed_model");
        let mut embedding_model =
            TextEmbedding::try_new(InitOptions::new(model).with_show_download_progress(true))?;

        println!("✅ Embedding model loaded");
        tracing::info!("fastembed_model_initialized");
        println!();

        // Check if collection exists, create with synthetic data if not
        println!("🔍 Checking collection '{}'...", collection_name);
        match client.collection_exists(&collection_name).await {
            Ok(exists) => {
                if exists {
                    println!("✅ Collection '{}' found", collection_name);
                    tracing::info!(collection = %collection_name, "collection_exists");
                } else {
                    println!(
                        "⚠️  Collection '{}' does not exist - creating with synthetic data...",
                        collection_name
                    );
                    tracing::warn!(collection = %collection_name, "collection_not_found_creating");

                    // Get embedding dimension from a test embedding
                    let test_embedding = embedding_model
                        .embed(vec!["test"], None)
                        .map_err(|e| anyhow::anyhow!("Failed to get embedding dimension: {}", e))?
                        .into_iter()
                        .next()
                        .ok_or_else(|| anyhow::anyhow!("No test embedding generated"))?;
                    let vector_size = test_embedding.len() as u64;

                    println!("   📏 Vector dimension: {}", vector_size);

                    // Create collection
                    client
                        .create_collection(
                            CreateCollectionBuilder::new(&collection_name)
                                .vectors_config(VectorParamsBuilder::new(vector_size, Distance::Cosine)),
                        )
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to create collection: {}", e))?;

                    println!("   ✅ Collection created");

                    // Generate and insert synthetic data
                    Self::populate_with_synthetic_data(
                        &client,
                        &collection_name,
                        &mut embedding_model,
                    )
                    .await?;
                }
            }
            Err(e) => {
                eprintln!("⚠️  Could not check collection: {}", e);
                tracing::warn!(error = %e, "collection_check_failed");
            }
        }
        println!();

        Ok(Self {
            client: Arc::new(client),
            embedding_model: Arc::new(Mutex::new(embedding_model)),
            collection_name: collection_name.clone(),
            qdrant_url: qdrant_url.clone(),
            embedding_model_name,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }

    async fn populate_with_synthetic_data(
        client: &Qdrant,
        collection_name: &str,
        embedding_model: &mut TextEmbedding,
        
    ) -> anyhow::Result<()> {
        println!("   📝 Generating synthetic data...");

        let synthetic_docs = vec![
            (
                "Rust is a systems programming language that focuses on safety, speed, and concurrency.",
                "rust_intro",
                "alice",
                vec!["programming", "rust", "systems"],
            ),
            (
                "Vector databases enable semantic search by storing embeddings of documents.",
                "vector_db_intro",
                "bob",
                vec!["databases", "vectors", "search"],
            ),
            (
                "Machine learning models can be used to generate embeddings from text data.",
                "ml_embeddings",
                "alice",
                vec!["machine-learning", "embeddings", "nlp"],
            ),
            (
                "Qdrant is a vector similarity search engine with a convenient API.",
                "qdrant_intro",
                "charlie",
                vec!["qdrant", "search", "api"],
            ),
            (
                "Semantic search understands the meaning behind queries, not just keywords.",
                "semantic_search",
                "bob",
                vec!["search", "semantics", "nlp"],
            ),
            (
                "MCP servers enable AI assistants to access external tools and data sources.",
                "mcp_intro",
                "alice",
                vec!["mcp", "ai", "integration"],
            ),
            (
                "FastEmbed provides efficient text embedding models for various languages.",
                "fastembed_intro",
                "charlie",
                vec!["embeddings", "models", "nlp"],
            ),
            (
                "Tokio is an asynchronous runtime for Rust, powering high-performance applications.",
                "tokio_intro",
                "bob",
                vec!["rust", "async", "performance"],
            ),
            (
                "Cosine similarity measures the angle between vectors in high-dimensional space.",
                "cosine_similarity",
                "alice",
                vec!["mathematics", "vectors", "similarity"],
            ),
            (
                "JSON is a lightweight data interchange format that is easy to read and write.",
                "json_intro",
                "charlie",
                vec!["data", "json", "format"],
            ),
        ];

        // Generate embeddings for all documents
        let texts: Vec<&str> = synthetic_docs.iter().map(|(text, _, _, _)| *text).collect();
        let embeddings = embedding_model
            .embed(texts.clone(), None)
            .map_err(|e| anyhow::anyhow!("Failed to generate embeddings: {}", e))?;

        // Create points
        let points: Vec<PointStruct> = synthetic_docs
            .into_iter()
            .zip(embeddings.into_iter())
            .enumerate()
            .map(|(id, ((text, filename, username, topics), embedding))| {
                let mut payload: HashMap<String, Value> = HashMap::new();
            payload.insert("text".to_string(), Value::from(text));
            payload.insert("filename".to_string(), Value::from(filename));
            payload.insert("username".to_string(), Value::from(username));
            payload.insert(
                "topics".to_string(),
                Value::from(
                    topics
                        .into_iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>(),
                ),
            );


                PointStruct::new(id as u64, embedding, payload)
            })
            .collect();

        // Insert points
        client
            .upsert_points(
                UpsertPointsBuilder::new(collection_name, points)
                    .wait(true),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to insert synthetic data: {}", e))?;


        println!("   ✅ Inserted 10 synthetic documents");
        println!("   📊 Sample documents:");
        println!("      • Rust programming introduction");
        println!("      • Vector database concepts");
        println!("      • Machine learning embeddings");
        println!("      • Qdrant search engine");
        println!("      • And more...");

        Ok(())
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, McpError> {
        let mut model = self.embedding_model.lock().await;

        model
            .embed(vec![text], None)
            .map_err(|e| {
                McpError::internal_error(format!("Failed to generate embedding: {}", e), None)
            })?
            .into_iter()
            .next()
            .ok_or_else(|| McpError::internal_error("No embedding generated".to_string(), None))
    }

    // TOOLS
    #[tool(
        description = "Search for semantically similar text content in the Qdrant collection. This is the primary search method - provide natural language queries."
    )]
    pub async fn search_text(
        &self,
        Parameters(args): Parameters<SearchTextArgs>,
    ) -> Result<CallToolResult, McpError> {
        let query_vector = self.embed_text(&args.query).await?;

        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit: args.limit,
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        let results = json!({
            "query": args.query,
            "count": search_result.result.len(),
            "results": search_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results).unwrap(),
        )]))
    }

    #[tool(
        description = "Search using a pre-computed vector (advanced use - most users should use search_text instead)"
    )]
    pub async fn search_vectors(
        &self,
        Parameters(args): Parameters<SearchVectorsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: args.vector.iter().map(|&v| v as f32).collect(),
                limit: args.limit,
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        let results = json!({
            "count": search_result.result.len(),
            "results": search_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results).unwrap(),
        )]))
    }

    #[tool(description = "Scroll through all points in the collection with pagination support")]
    pub async fn scroll_points(
        &self,
        Parameters(args): Parameters<ScrollPointsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let scroll_result = self
            .client
            .scroll(ScrollPoints {
                collection_name: self.collection_name.clone(),
                limit: Some(args.limit),
                offset: args.offset.map(|id| id.into()),
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(format!("Scroll failed: {}", e), None))?;

        let results = json!({
            "count": scroll_result.result.len(),
            "points": scroll_result.result.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "payload": p.payload,
                "vector": p.vectors.as_ref().map(|v| format!("{:?}", v)),
            })).collect::<Vec<_>>(),
            "next_offset": scroll_result.next_page_offset.map(|o| format!("{:?}", o))
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results).unwrap(),
        )]))
    }

    #[tool(description = "Count the total number of points in the collection")]
    pub async fn count_points(
        &self,
        Parameters(args): Parameters<CountPointsArgs>,
    ) -> Result<CallToolResult, McpError> {
        let count_result = self
            .client
            .count(CountPoints {
                collection_name: self.collection_name.clone(),
                exact: Some(args.exact),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(format!("Count failed: {}", e), None))?;

        let result = json!({
            "count": count_result.result.map(|r| r.count).unwrap_or(0)
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap(),
        )]))
    }

    #[tool(
        description = "Search for semantically similar text with metadata filtering - find content matching your query AND specific criteria (e.g., by username or filename)"
    )]
    pub async fn filter_search(
        &self,
        Parameters(args): Parameters<FilterSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let query_vector = self.embed_text(&args.query).await?;

        let filter = Filter {
            must: vec![Condition {
                condition_one_of: Some(qdrant_client::qdrant::condition::ConditionOneOf::Field(
                    FieldCondition {
                        key: args.filter_field.clone(),
                        r#match: Some(Match {
                            match_value: Some(
                                qdrant_client::qdrant::r#match::MatchValue::Keyword(
                                    args.filter_value.clone(),
                                ),
                            ),
                        }),
                        ..Default::default()
                    },
                )),
            }],
            ..Default::default()
        };

        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit: args.limit,
                filter: Some(filter),
                with_payload: Some(true.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Filtered search failed: {}", e), None)
            })?;

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

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results).unwrap(),
        )]))
    }

    #[tool(description = "Get collection information and statistics")]
    pub async fn get_collection_info(&self) -> Result<CallToolResult, McpError> {
        let collection_info = self
            .client
            .collection_info(&self.collection_name)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to get collection info: {}", e), None)
            })?;

        let info = json!({
            "collection_name": self.collection_name,
            "qdrant_url": self.qdrant_url,
            "embedding_model": self.embedding_model_name,
            "status": format!("{:?}", collection_info.result.as_ref().map(|r| r.status)),
            "segments_count": collection_info.result.as_ref().map(|r| r.segments_count),
            "points_count": collection_info.result.as_ref().and_then(|r| r.points_count),
            "indexed_vectors": collection_info.result.as_ref().and_then(|r| r.indexed_vectors_count),
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&info).unwrap(),
        )]))
    }

    #[tool(
        description = "Semantic search with optional keyword filtering - finds semantically similar content, optionally filtered to contain specific keywords"
    )]
    pub async fn keyword_search(
        &self,
        Parameters(args): Parameters<KeywordSearchArgs>,
    ) -> Result<CallToolResult, McpError> {
        let query_vector = self.embed_text(&args.query).await?;

        let keywords: Vec<String> = args
            .must_contain_keywords
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        let initial_limit = if keywords.is_empty() {
            args.limit
        } else {
            args.limit * 5
        };

        let search_result = self
            .client
            .search_points(SearchPoints {
                collection_name: self.collection_name.clone(),
                vector: query_vector,
                limit: initial_limit,
                with_payload: Some(args.with_payload.into()),
                ..Default::default()
            })
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        if keywords.is_empty() {
            let results = json!({
                "query": args.query,
                "keywords": [],
                "keyword_filtering": false,
                "count": search_result.result.len(),
                "results": search_result.result.iter().map(|p| json!({
                    "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                    "score": p.score,
                    "payload": p.payload,
                })).collect::<Vec<_>>()
            });

            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&results).unwrap(),
            )]));
        }

        let filtered_results: Vec<_> = search_result
            .result
            .iter()
            .filter(|point| {
                let text_fields = ["text", "content", "body", "description"];

                for field_name in &text_fields {
                    if let Some(value) = point.payload.get(*field_name) {
                        if let Some(text) = value.as_str() {
                            let text_lower = text.to_lowercase();
                            let contains_all = keywords.iter().all(|kw| text_lower.contains(kw));
                            if contains_all {
                                return true;
                            }
                        }
                    }
                }
                false
            })
            .take(args.limit as usize)
            .collect();

        let results = json!({
            "query": args.query,
            "keywords": keywords,
            "keyword_filtering": true,
            "total_semantic_matches": search_result.result.len(),
            "keyword_filtered_count": filtered_results.len(),
            "results": filtered_results.iter().map(|p| json!({
                "id": p.id.as_ref().map(|id| format!("{:?}", id)),
                "score": p.score,
                "payload": p.payload,
            })).collect::<Vec<_>>()
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&results).unwrap(),
        )]))
    }

    // PROMPTS
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