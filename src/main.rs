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
    // Required tools for ChatGPT Connectors and deep research
    server.register_tool(Box::new(tools::mock::SearchTool::new())).await;
    server.register_tool(Box::new(tools::mock::FetchTool::new())).await;
    
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
    
    // Initialize app loader for marketplace FIRST (needed by connector)
    let app_loader = if config.mysql.enabled {
        println!("📦 Initializing Application Marketplace...");
        match crate::core::app_loader::AppLoader::new(&config.mysql).await {
            Ok(loader) => {
                // Initialize tables
                if let Err(e) = loader.initialize_tables().await {
                    println!("⚠️  Failed to initialize app loader tables: {}", e);
                }
                Some(Arc::new(loader))
            }
            Err(e) => {
                println!("⚠️  Failed to create app loader: {}", e);
                None
            }
        }
    } else {
        println!("⚠️  Application Marketplace disabled (MySQL required)");
        None
    };
    
    // Initialize the WASM KV Store MySQL persistence if MySQL is enabled
    if config.mysql.enabled {
        if let Err(e) = crate::core::wasm_runtime::WasmKvStore::init_mysql_pool(&config.mysql).await {
            println!("⚠️  WASM KV Store MySQL initialization failed: {}", e);
            println!("   (WASM apps will use in-memory storage - data will not persist across restarts)");
        }
    }
    
    // Create the notification broadcaster for resource updates
    // This needs to be created before the connector so it can be shared
    let notifications = Arc::new(crate::core::notifications::NotificationBroadcaster::new());
    println!("📢 Notification broadcaster initialized");
    
    // Initialize the Cursor MCP Connector if MySQL is enabled
    // Note: Connector needs app_loader for WASM application support
    let connector = if config.mysql.enabled {
        println!("🔌 Initializing Cursor MCP Connector...");
        
        let mut connector = if let Some(ref loader) = app_loader {
            println!("   ✅ WASM Runtime enabled with App Loader");
            crate::core::connector::CursorMcpConnector::with_app_loader(
                config.mysql.clone(),
                Arc::clone(loader),
            )
        } else {
            println!("   ⚠️  WASM Runtime disabled (no App Loader)");
            crate::core::connector::CursorMcpConnector::new(config.mysql.clone())
        };
        
        // Set the notification broadcaster for resource updates
        connector.set_notifications(Arc::clone(&notifications));
        
        // Initialize Wassette runtime if feature is enabled
        #[cfg(feature = "wassette")]
        {
            use std::path::PathBuf;
            let component_dir = PathBuf::from("./wassette-components");
            println!("   🔧 Initializing Wassette runtime...");
            println!("      Component dir: {}", component_dir.display());
            
            match connector.set_wassette_runtime_with_redis(component_dir, Some(config.redis.clone())).await {
                Ok(()) => {
                    println!("   ✅ Wassette runtime enabled for Components");
                }
                Err(e) => {
                    eprintln!("   ⚠️  Wassette runtime initialization failed: {}", e);
                    eprintln!("      Components will not be loadable");
                }
            }
        }
        
        Some(Arc::new(connector))
    } else {
        println!("⚠️  Per-user MCP endpoints disabled (MySQL required)");
        None
    };
    
    // Get port from environment or use config
    // Railway uses PORT, but we also support MCP_PORT for local development
    let port: u16 = env::var("PORT")
        .or_else(|_| env::var("MCP_PORT"))
        .unwrap_or_else(|_| config.server.port.to_string())
        .parse()
        .unwrap_or(config.server.port);
    
    // Get host from config
    let host = config.server.host.clone();
    
    println!("\nStarting HTTP server on {}:{}...", host, port);
    println!("API endpoint: http://{}:{}/mcp", host, port);
    println!("Health check: http://{}:{}/health", host, port);
    println!("Dashboard: http://{}:{}/dashboard", host, port);
    println!("Marketplace: http://{}:{}/marketplace", host, port);
    
    if connector.is_some() {
        println!("\n📍 Per-user MCP endpoints enabled:");
        println!("   http://{}:{}/u/{{username}}/mcp", host, port);
    }
    
    println!();
    
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
    
    // Initialize Vector Database (Milvus) for similarity search
    let vector_db = if config.milvus.enabled {
        println!("🔍 Initializing Vector Database for similarity search...");
        let milvus_config = core::database::MilvusConfig {
            host: config.milvus.host.clone(),
            port: config.milvus.port,
            collection_name: config.milvus.collection_name.clone(),
            dimension: config.milvus.dimension,
            metric: config.milvus.metric.clone(),
        };
        let client = core::database::MilvusClient::new(milvus_config);
        
        // Check if Milvus is available
        if client.check_connection().await {
            println!("   ✅ Milvus connected at {}:{}", config.milvus.host, config.milvus.port);
        } else {
            println!("   ⚠️  Milvus not available, using in-memory fallback");
        }
        
        Some(Arc::new(client))
    } else {
        println!("🔍 Initializing Vector Database (in-memory mode)...");
        // Always initialize vector DB in memory for fuzzy search
        let client = core::database::MilvusClient::with_defaults();
        Some(Arc::new(client))
    };
    
    // Start the HTTP server with metrics, auth, connector, app loader, and vector DB
    let mut http_server = crate::core::http_server::HttpServer::with_metrics(server.clone(), metrics, host, port)
        .with_mysql_config(config.mysql.clone())
        .with_notifications(notifications);  // Share the same notification broadcaster
    
    if let Some(auth) = auth_service {
        http_server = http_server.with_auth(auth);
    }
    
    if let Some(conn) = connector {
        http_server = http_server.with_connector(conn);
    }
    
    if let Some(loader) = app_loader {
        http_server = http_server.with_app_loader(loader);
    }
    
    if let Some(vdb) = vector_db {
        http_server = http_server.with_vector_db(vdb);
    }
    
    http_server.start().await?;
    
    Ok(())
}
