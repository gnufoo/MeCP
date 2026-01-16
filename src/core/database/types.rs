use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector representation for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub id: String,
    pub values: Vec<f32>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Query result for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Graph node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// SQL query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub affected_rows: Option<u64>,
}

/// Database connection configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub options: HashMap<String, String>,
}
