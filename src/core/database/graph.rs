use async_trait::async_trait;
use anyhow::Result;
use super::types::{GraphNode, GraphEdge, GraphQueryResult, DatabaseConfig};

/// Graph Database trait - abstraction for graph database operations
/// Supports databases like Neo4j, ArangoDB, JanusGraph, Amazon Neptune, etc.
/// Also supports GraphQL endpoints for knowledge graphs
#[async_trait]
pub trait GraphDatabase: Send + Sync {
    /// Connect to the graph database
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()>;
    
    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Execute a graph query (Cypher, Gremlin, or GraphQL)
    async fn query(&self, query: &str) -> Result<GraphQueryResult>;
    
    /// Create a node
    async fn create_node(&self, node: GraphNode) -> Result<String>;
    
    /// Create an edge between nodes
    async fn create_edge(&self, edge: GraphEdge) -> Result<String>;
    
    /// Get a node by ID
    async fn get_node(&self, id: &str) -> Result<Option<GraphNode>>;
    
    /// Update node properties
    async fn update_node(
        &self,
        id: &str,
        properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()>;
    
    /// Delete a node
    async fn delete_node(&self, id: &str) -> Result<()>;
    
    /// Delete an edge
    async fn delete_edge(&self, id: &str) -> Result<()>;
    
    /// Find neighbors of a node
    async fn find_neighbors(
        &self,
        node_id: &str,
        direction: EdgeDirection,
        edge_label: Option<&str>,
    ) -> Result<Vec<GraphNode>>;
    
    /// Find shortest path between two nodes
    async fn shortest_path(&self, from: &str, to: &str) -> Result<Option<GraphQueryResult>>;
    
    /// Check connection status
    fn is_connected(&self) -> bool;
}

/// Direction for traversing edges
#[derive(Debug, Clone, Copy)]
pub enum EdgeDirection {
    Incoming,
    Outgoing,
    Both,
}

/// Mock implementation for testing
pub struct MockGraphDatabase {
    connected: bool,
}

impl MockGraphDatabase {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for MockGraphDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphDatabase for MockGraphDatabase {
    async fn connect(&mut self, _config: DatabaseConfig) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn query(&self, _query: &str) -> Result<GraphQueryResult> {
        Ok(GraphQueryResult {
            nodes: vec![],
            edges: vec![],
        })
    }

    async fn create_node(&self, node: GraphNode) -> Result<String> {
        Ok(node.id)
    }

    async fn create_edge(&self, edge: GraphEdge) -> Result<String> {
        Ok(edge.id)
    }

    async fn get_node(&self, id: &str) -> Result<Option<GraphNode>> {
        Ok(Some(GraphNode {
            id: id.to_string(),
            label: "MockNode".to_string(),
            properties: std::collections::HashMap::new(),
        }))
    }

    async fn update_node(
        &self,
        _id: &str,
        _properties: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        Ok(())
    }

    async fn delete_node(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn delete_edge(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn find_neighbors(
        &self,
        _node_id: &str,
        _direction: EdgeDirection,
        _edge_label: Option<&str>,
    ) -> Result<Vec<GraphNode>> {
        Ok(vec![])
    }

    async fn shortest_path(&self, _from: &str, _to: &str) -> Result<Option<GraphQueryResult>> {
        Ok(None)
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}
