mod core;
mod resources;
mod tools;
mod prompts;
mod services;

use anyhow::Result;
use std::sync::Arc;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("MeCP - Modular Context Protocol Server");
    println!("=======================================\n");

    // Initialize the MCP server
    let server = Arc::new(crate::core::server::McpServer::new());
    
    // Register resources
    server.register_resource(Box::new(resources::mock::MockResource::new())).await;
    
    // Register tools
    server.register_tool(Box::new(tools::mock::HelloWorldTool::new())).await;
    
    // Register prompts
    server.register_prompt(Box::new(prompts::mock::MockPrompt::new())).await;
    
    println!("Server initialized successfully!");
    println!("\nRegistered components:");
    println!("  - Resources: {}", server.resource_count().await);
    println!("  - Tools: {}", server.tool_count().await);
    println!("  - Prompts: {}", server.prompt_count().await);
    
    // Load configuration for MySQL metrics
    let config = services::config::ServiceConfig::load("config.toml")
        .unwrap_or_else(|_| {
            println!("⚠️  Could not load config.toml, using defaults");
            services::config::ServiceConfig::default()
        });
    
    // Initialize metrics collector with MySQL backend if enabled
    let metrics = if config.mysql.enabled {
        println!("📊 Enabling MySQL metrics backend...");
        let mysql_writer = Arc::new(crate::core::metrics::MySqlMetricsWriter::new(
            &config.mysql.host,
            config.mysql.port,
            &config.mysql.database,
            &config.mysql.username,
            &config.mysql.password,
        ));
        Arc::new(crate::core::metrics::MetricsCollector::with_mysql_writer(mysql_writer))
    } else {
        println!("⚠️  MySQL metrics disabled, using in-memory only");
        Arc::new(crate::core::metrics::MetricsCollector::new())
    };
    
    // Get port from environment or use default
    let port: u16 = env::var("MCP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);
    
    println!("\nStarting HTTP server on port {}...", port);
    println!("API endpoint: http://127.0.0.1:{}/mcp", port);
    println!("Health check: http://127.0.0.1:{}/health", port);
    println!("Dashboard: http://127.0.0.1:{}/dashboard\n", port);
    
    // Initialize authentication if configured
    let auth_service = if let Some(auth_config) = &config.auth {
        if auth_config.enabled {
            println!("🔐 Web3 Authentication enabled");
            println!("   Allowed address: {}", auth_config.allowed_address);
            println!("   Session duration: {}s ({}h)", auth_config.session_duration, auth_config.session_duration / 3600);
            
            let auth_config_for_service = crate::core::auth::AuthConfig {
                enabled: auth_config.enabled,
                allowed_address: auth_config.allowed_address.clone(),
                jwt_secret: auth_config.jwt_secret.clone(),
                session_duration: auth_config.session_duration,
            };
            Some(Arc::new(crate::core::auth::AuthService::new(auth_config_for_service)))
        } else {
            println!("🔓 Web3 Authentication disabled in config");
            None
        }
    } else {
        println!("🔓 Web3 Authentication not configured");
        None
    };
    
    // Start the HTTP server with metrics and auth
    let mut http_server = crate::core::http_server::HttpServer::with_metrics(server.clone(), metrics, port);
    if let Some(auth) = auth_service {
        http_server = http_server.with_auth(auth);
    }
    http_server.start().await?;
    
    Ok(())
}
