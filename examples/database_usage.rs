use mecp::core::database::{VectorDatabase, GraphDatabase, SqlDatabase, DatabaseConfig};
use mecp::core::database::vector::MockVectorDatabase;
use mecp::core::database::graph::MockGraphDatabase;
use mecp::core::database::sql::{MockSqlDatabase, DatabaseType};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Database Abstraction Examples ===\n");

    // Vector Database Example
    vector_database_example().await?;
    
    // Graph Database Example
    graph_database_example().await?;
    
    // SQL Database Example
    sql_database_example().await?;

    Ok(())
}

async fn vector_database_example() -> Result<()> {
    println!("1. Vector Database Example");
    println!("   (Supports: Milvus, Weaviate, Qdrant, etc.)\n");

    let mut db = MockVectorDatabase::new();
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 6333,
        database: "my_vectors".to_string(),
        username: None,
        password: None,
        options: std::collections::HashMap::new(),
    };

    db.connect(config).await?;
    println!("   ✓ Connected to vector database");

    // Search for similar vectors
    let query_vector = vec![0.1, 0.2, 0.3, 0.4];
    let results = db.search(query_vector, 5, None).await?;
    println!("   ✓ Found {} similar vectors", results.len());

    db.disconnect().await?;
    println!("   ✓ Disconnected\n");

    Ok(())
}

async fn graph_database_example() -> Result<()> {
    println!("2. Graph Database Example");
    println!("   (Supports: Neo4j, GraphQL, ArangoDB, etc.)\n");

    let mut db = MockGraphDatabase::new();
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 7687,
        database: "neo4j".to_string(),
        username: Some("neo4j".to_string()),
        password: Some("password".to_string()),
        options: std::collections::HashMap::new(),
    };

    db.connect(config).await?;
    println!("   ✓ Connected to graph database");

    // Execute a Cypher query
    let result = db.query("MATCH (n) RETURN n LIMIT 10").await?;
    println!("   ✓ Executed query, returned {} nodes", result.nodes.len());

    db.disconnect().await?;
    println!("   ✓ Disconnected\n");

    Ok(())
}

async fn sql_database_example() -> Result<()> {
    println!("3. SQL Database Example");
    println!("   (Supports: MySQL, PostgreSQL, SQLite, etc.)\n");

    let mut db = MockSqlDatabase::new(DatabaseType::MySQL);
    
    let config = DatabaseConfig {
        host: "localhost".to_string(),
        port: 3306,
        database: "mydb".to_string(),
        username: Some("user".to_string()),
        password: Some("password".to_string()),
        options: std::collections::HashMap::new(),
    };

    db.connect(config).await?;
    println!("   ✓ Connected to SQL database");

    // Execute a query
    let result = db.query("SELECT * FROM users LIMIT 10", vec![]).await?;
    println!("   ✓ Executed query, columns: {:?}", result.columns);

    // Start a transaction
    let mut tx = db.begin_transaction().await?;
    println!("   ✓ Started transaction");
    
    tx.execute("UPDATE users SET active = true WHERE id = ?", vec![serde_json::json!(1)]).await?;
    tx.commit().await?;
    println!("   ✓ Committed transaction");

    db.disconnect().await?;
    println!("   ✓ Disconnected\n");

    Ok(())
}
