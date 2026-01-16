use async_trait::async_trait;
use anyhow::Result;
use super::types::{SqlQueryResult, DatabaseConfig};

/// SQL Database trait - abstraction for SQL database operations
/// Supports databases like MySQL, PostgreSQL, SQLite, SQL Server, etc.
#[async_trait]
pub trait SqlDatabase: Send + Sync {
    /// Connect to the SQL database
    async fn connect(&mut self, config: DatabaseConfig) -> Result<()>;
    
    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<()>;
    
    /// Execute a SQL query
    async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<SqlQueryResult>;
    
    /// Execute a SQL statement (INSERT, UPDATE, DELETE)
    async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64>;
    
    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<Box<dyn SqlTransaction>>;
    
    /// Check connection status
    fn is_connected(&self) -> bool;
    
    /// Get database type
    fn database_type(&self) -> DatabaseType;
}

/// SQL Transaction trait
#[async_trait]
pub trait SqlTransaction: Send + Sync {
    /// Commit the transaction
    async fn commit(&mut self) -> Result<()>;
    
    /// Rollback the transaction
    async fn rollback(&mut self) -> Result<()>;
    
    /// Execute a query within the transaction
    async fn query(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<SqlQueryResult>;
    
    /// Execute a statement within the transaction
    async fn execute(&self, sql: &str, params: Vec<serde_json::Value>) -> Result<u64>;
}

/// Database type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    MySQL,
    PostgreSQL,
    SQLite,
    SqlServer,
    Oracle,
    Other,
}

/// Mock implementation for testing
pub struct MockSqlDatabase {
    connected: bool,
    db_type: DatabaseType,
}

impl MockSqlDatabase {
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            connected: false,
            db_type,
        }
    }
}

impl Default for MockSqlDatabase {
    fn default() -> Self {
        Self::new(DatabaseType::MySQL)
    }
}

#[async_trait]
impl SqlDatabase for MockSqlDatabase {
    async fn connect(&mut self, _config: DatabaseConfig) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn query(&self, _sql: &str, _params: Vec<serde_json::Value>) -> Result<SqlQueryResult> {
        Ok(SqlQueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
            affected_rows: None,
        })
    }

    async fn execute(&self, _sql: &str, _params: Vec<serde_json::Value>) -> Result<u64> {
        Ok(1)
    }

    async fn begin_transaction(&self) -> Result<Box<dyn SqlTransaction>> {
        Ok(Box::new(MockSqlTransaction {}))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn database_type(&self) -> DatabaseType {
        self.db_type
    }
}

/// Mock transaction implementation
pub struct MockSqlTransaction {}

#[async_trait]
impl SqlTransaction for MockSqlTransaction {
    async fn commit(&mut self) -> Result<()> {
        Ok(())
    }

    async fn rollback(&mut self) -> Result<()> {
        Ok(())
    }

    async fn query(&self, _sql: &str, _params: Vec<serde_json::Value>) -> Result<SqlQueryResult> {
        Ok(SqlQueryResult {
            columns: vec![],
            rows: vec![],
            affected_rows: None,
        })
    }

    async fn execute(&self, _sql: &str, _params: Vec<serde_json::Value>) -> Result<u64> {
        Ok(0)
    }
}
