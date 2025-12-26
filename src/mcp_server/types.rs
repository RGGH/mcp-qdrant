use rmcp::schemars;
use serde::Deserialize;
use rmcp::schemars::JsonSchema;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchTextArgs {
    /// The text query to search for semantically similar content
    pub query: String,
    
    #[serde(default = "default_limit")]
    pub limit: u64,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchVectorsArgs {
    pub vector: Vec<f64>,
    
    #[serde(default = "default_limit")]
    pub limit: u64,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScrollPointsArgs {
    #[serde(default = "default_limit_u32")]
    pub limit: u32,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    
    #[serde(default = "default_with_payload")]
    pub with_payload: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CountPointsArgs {
    #[serde(default = "default_exact")]
    pub exact: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
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
