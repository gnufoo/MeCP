use async_trait::async_trait;
use anyhow::Result;
use super::types::{Vector, VectorSearchResult, DatabaseConfig};

/// Vector Database trait - abstraction for vector database operations
/// Supports databases like Milvus, Weaviate, Qdrant, etc.
#[async_trait]
pub trait VectorDatabase: Send + Sync {
    /// Connect to the vector database
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()>;
    
    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Insert a vector into the database
    async fn insert(&self, vector: Vector) -> Result<String>;
    
    /// Insert multiple vectors
    async fn batch_insert(&self, vectors: Vec<Vector>) -> Result<Vec<String>>;
    
    /// Search for similar vectors
    async fn search(
        &self,
        query_vector: Vec<f32>,
        top_k: usize,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<VectorSearchResult>>;
    
    /// Delete a vector by ID
    async fn delete(&self, id: &str) -> Result<()>;
    
    /// Update vector metadata
    async fn update_metadata(
        &self,
        id: &str,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()>;
    
    /// Get vector by ID
    async fn get(&self, id: &str) -> Result<Option<Vector>>;
    
    /// Create an index/collection
    async fn create_index(&self, name: &str, dimension: usize) -> Result<()>;
    
    /// Delete an index/collection
    async fn delete_index(&self, name: &str) -> Result<()>;
    
    /// Check connection status
    fn is_connected(&self) -> bool;
}

/// Mock implementation for testing
pub struct MockVectorDatabase {
    connected: bool,
}

impl MockVectorDatabase {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for MockVectorDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorDatabase for MockVectorDatabase {
    async fn connect(&mut self, _config: DatabaseConfig) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn insert(&self, vector: Vector) -> Result<String> {
        Ok(vector.id)
    }

    async fn batch_insert(&self, vectors: Vec<Vector>) -> Result<Vec<String>> {
        Ok(vectors.into_iter().map(|v| v.id).collect())
    }

    async fn search(
        &self,
        _query_vector: Vec<f32>,
        top_k: usize,
        _filter: Option<serde_json::Value>,
    ) -> Result<Vec<VectorSearchResult>> {
        // Mock implementation returns dummy results
        Ok((0..top_k)
            .map(|i| VectorSearchResult {
                id: format!("vec_{}", i),
                score: 0.9 - (i as f32 * 0.1),
                metadata: None,
            })
            .collect())
    }

    async fn delete(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn update_metadata(
        &self,
        _id: &str,
        _metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Vector>> {
        Ok(Some(Vector {
            id: id.to_string(),
            values: vec![0.1, 0.2, 0.3],
            metadata: None,
        }))
    }

    async fn create_index(&self, _name: &str, _dimension: usize) -> Result<()> {
        Ok(())
    }

    async fn delete_index(&self, _name: &str) -> Result<()> {
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
