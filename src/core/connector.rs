//! MCP Connector module for MeCP
//! 
//! Provides base connector functionality and Cursor-specific implementation
//! for handling per-user MCP endpoints.

#![allow(dead_code)]

use anyhow::{Result, bail, Context};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

use crate::services::config::MySqlConfig;
use crate::tools::Tool;
use crate::resources::Resource;
use crate::core::types::{ToolResult, JsonValue, ResourceContent};
use crate::core::application::{Application, ApplicationManager};
use crate::core::user::{UserManager, UserInfo};
use crate::core::counter::CounterApplication;
use crate::core::app_loader::AppLoader;
use crate::core::wasm_runtime::{WasmRuntime, WasmApp};

/// Connector capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorCapabilities {
    pub resources: bool,
    pub resources_subscribe: bool,
    pub tools: bool,
    pub prompts: bool,
}

impl Default for ConnectorCapabilities {
    fn default() -> Self {
        Self {
            resources: true,
            resources_subscribe: true,
            tools: true,
            prompts: true,
        }
    }
}

/// Base MCP Connector trait
#[async_trait]
pub trait McpConnector: Send + Sync {
    /// Get connector ID
    fn connector_id(&self) -> &str;
    
    /// Get connector name
    fn name(&self) -> &str;
    
    /// Get connector capabilities
    fn capabilities(&self) -> ConnectorCapabilities;
    
    /// Initialize the connector for a user
    async fn initialize(&self, user: &UserInfo) -> Result<()>;
    
    /// Get tools available for the user
    async fn get_tools(&self, username: &str) -> Result<Vec<Box<dyn Tool>>>;
    
    /// Get resources available for the user
    async fn get_resources(&self, username: &str) -> Result<Vec<Box<dyn Resource>>>;
    
    /// Handle resource subscription
    async fn subscribe(&self, uri: &str, username: &str) -> Result<()>;
    
    /// Handle resource unsubscription
    async fn unsubscribe(&self, uri: &str, username: &str) -> Result<()>;
    
    /// Call a tool
    async fn call_tool(&self, username: &str, tool_name: &str, params: JsonValue) -> Result<ToolResult>;
    
    /// Read a resource
    async fn read_resource(&self, username: &str, uri: &str) -> Result<ResourceContent>;
}

/// Cursor MCP Connector implementation
/// 
/// This connector is specifically designed for Cursor AI agent integration.
/// It loads applications from the user's configuration and provides
/// per-user tool and resource access.
pub struct CursorMcpConnector {
    mysql_config: MySqlConfig,
    /// Cached user sessions with their loaded applications
    user_sessions: RwLock<HashMap<String, UserSession>>,
    /// WASM runtime for executing WASM applications (legacy core modules)
    wasm_runtime: Arc<WasmRuntime>,
    /// Wassette runtime for WebAssembly Components
    #[cfg(feature = "wassette")]
    wassette_runtime: Option<Arc<crate::core::wassette_runtime::WassetteRuntime>>,
    /// App loader for fetching WASM binaries from the database
    app_loader: Option<Arc<AppLoader>>,
    /// Component directory for Wassette
    #[cfg(feature = "wassette")]
    component_dir: Option<std::path::PathBuf>,
    /// Flag to track if initial notifications have been sent per user
    initial_notifications_sent: Arc<RwLock<HashMap<String, bool>>>,
    /// Notification broadcaster for sending resource updates
    notifications: Option<Arc<crate::core::notifications::NotificationBroadcaster>>,
}

/// User session containing loaded applications
struct UserSession {
    user: UserInfo,
    applications: Vec<Arc<dyn Application>>,
}

impl CursorMcpConnector {
    /// Create a new Cursor MCP Connector
    pub fn new(mysql_config: MySqlConfig) -> Self {
        let wasm_runtime = Arc::new(
            WasmRuntime::new().expect("Failed to initialize WASM runtime")
        );
        
        Self {
            mysql_config,
            user_sessions: RwLock::new(HashMap::new()),
            wasm_runtime,
            #[cfg(feature = "wassette")]
            wassette_runtime: None,
            app_loader: None,
            #[cfg(feature = "wassette")]
            component_dir: None,
            initial_notifications_sent: Arc::new(RwLock::new(HashMap::new())),
            notifications: None,
        }
    }
    
    /// Create a new Cursor MCP Connector with an app loader
    pub fn with_app_loader(mysql_config: MySqlConfig, app_loader: Arc<AppLoader>) -> Self {
        let wasm_runtime = Arc::new(
            WasmRuntime::new().expect("Failed to initialize WASM runtime")
        );
        
        Self {
            mysql_config,
            user_sessions: RwLock::new(HashMap::new()),
            wasm_runtime,
            #[cfg(feature = "wassette")]
            wassette_runtime: None,
            app_loader: Some(app_loader),
            #[cfg(feature = "wassette")]
            component_dir: None,
            initial_notifications_sent: Arc::new(RwLock::new(HashMap::new())),
            notifications: None,
        }
    }
    
    /// Set Wassette runtime for loading WebAssembly Components
    #[cfg(feature = "wassette")]
    pub async fn set_wassette_runtime(&mut self, component_dir: std::path::PathBuf) -> Result<()> {
        self.set_wassette_runtime_with_redis(component_dir, None).await
    }
    
    /// Set Wassette runtime for loading WebAssembly Components with Redis support
    #[cfg(feature = "wassette")]
    pub async fn set_wassette_runtime_with_redis(
        &mut self, 
        component_dir: std::path::PathBuf,
        redis_config: Option<crate::services::config::RedisConfig>,
    ) -> Result<()> {
        use crate::core::wassette_runtime::WassetteRuntime;
        
        let runtime = WassetteRuntime::new_with_redis(&component_dir, redis_config).await?;
        self.wassette_runtime = Some(Arc::new(runtime));
        self.component_dir = Some(component_dir);
        info!("🔧 Wassette runtime initialized for Components");
        
        Ok(())
    }
    
    /// Set the app loader (for lazy initialization)
    pub fn set_app_loader(&mut self, app_loader: Arc<AppLoader>) {
        self.app_loader = Some(app_loader);
    }
    
    /// Set the notification broadcaster for resource updates
    pub fn set_notifications(&mut self, notifications: Arc<crate::core::notifications::NotificationBroadcaster>) {
        self.notifications = Some(notifications);
        info!("📢 Notification broadcaster attached to connector");
    }

    /// Load applications for a user
    async fn load_user_applications(&self, username: &str, user_id: u64) -> Result<Vec<Arc<dyn Application>>> {
        let app_manager = ApplicationManager::new(&self.mysql_config).await?;
        let user_apps = app_manager.list_user_applications(username).await?;
        
        let mut applications: Vec<Arc<dyn Application>> = Vec::new();
        
        for app_info in user_apps {
            match app_info.app_id.as_str() {
                // Built-in applications
                "counter" => {
                    let counter = CounterApplication::from_config(&self.mysql_config).await?;
                    counter.set_user_context(username).await?;
                    applications.push(Arc::new(counter));
                    info!("📦 Loaded native app: counter for user {}", username);
                }
                
                // Skip the cursor_mcp_connector as it's a meta-application
                "cursor_mcp_connector" => {
                    continue;
                }
                
                // Try to load as WASM/Component application
                _ => {
                    if let Some(ref app_loader) = self.app_loader {
                        // First, get the binary to check if it's a Component
                        match app_loader.get_application(&app_info.app_id).await {
                            Ok(Some(app_data)) => {
                                if let Some(wasm_bytes) = &app_data.wasm_binary {
                                    let is_component = Self::is_wasm_component(wasm_bytes);
                                    
                                    if is_component {
                                        // Try to load as WebAssembly Component using Wassette
                                        #[cfg(feature = "wassette")]
                                        {
                                            match self.try_load_wassette_app(&app_info.app_id, user_id, username, wasm_bytes).await {
                                                Ok(app) => {
                                                    info!("📦 Loaded Wassette Component: {} for user {}", app_info.app_id, username);
                                                    applications.push(app);
                                                }
                                                Err(e) => {
                                                    warn!("⚠️ Failed to load Wassette Component '{}': {}", app_info.app_id, e);
                                                }
                                            }
                                        }
                                        #[cfg(not(feature = "wassette"))]
                                        {
                                            warn!("⚠️ App '{}' is a WebAssembly Component but Wassette feature is not enabled", app_info.app_id);
                                        }
                                    } else {
                                        // Load as legacy core WASM module
                                        match self.try_load_wasm_app(&app_info.app_id, user_id, username, app_loader).await {
                                            Ok(wasm_app) => {
                                                info!("📦 Loaded WASM app: {} for user {}", app_info.app_id, username);
                                                applications.push(Arc::new(wasm_app));
                                            }
                                            Err(e) => {
                                                warn!("⚠️ Failed to load WASM app '{}': {}", app_info.app_id, e);
                                            }
                                        }
                                    }
                                } else {
                                    warn!("⚠️ App '{}' has no WASM binary", app_info.app_id);
                                }
                            }
                            Ok(None) => {
                                warn!("⚠️ App '{}' not found in marketplace", app_info.app_id);
                            }
                            Err(e) => {
                                warn!("⚠️ Failed to get app '{}': {}", app_info.app_id, e);
                            }
                        }
                    } else {
                        warn!("⚠️ App loader not configured, cannot load WASM app: {}", app_info.app_id);
                    }
                }
            }
        }
        
        Ok(applications)
    }
    
    /// Check if WASM bytes are a WebAssembly Component (vs core module)
    fn is_wasm_component(wasm_bytes: &[u8]) -> bool {
        // WebAssembly Component binary starts with:
        // 0x00 0x61 0x73 0x6D (magic number)
        // 0x0D 0x00 0x01 0x00 (Component layer version)
        //
        // Core module starts with:
        // 0x00 0x61 0x73 0x6D (magic number)
        // 0x01 0x00 0x00 0x00 (core version 1)
        
        if wasm_bytes.len() < 8 {
            return false;
        }
        
        // Check magic number
        if &wasm_bytes[0..4] != b"\0asm" {
            return false;
        }
        
        // Check version - Component layer is 0x0D 0x00 0x01 0x00
        wasm_bytes[4] == 0x0D && wasm_bytes[5] == 0x00
    }
    
    /// Try to load a WASM application from the database (legacy core module)
    async fn try_load_wasm_app(
        &self,
        app_id: &str,
        user_id: u64,
        username: &str,
        app_loader: &AppLoader,
    ) -> Result<WasmApp> {
        // Get the WASM binary from the database
        let app_data = app_loader.get_application(app_id).await?
            .ok_or_else(|| anyhow::anyhow!("Application '{}' not found in marketplace", app_id))?;
        
        let wasm_bytes = app_data.wasm_binary
            .ok_or_else(|| anyhow::anyhow!("Application '{}' has no WASM binary", app_id))?;
        
        info!("🔧 Loading WASM module for '{}' ({} bytes)", app_id, wasm_bytes.len());
        
        // Compile and instantiate the WASM module with KV persistence
        let module = self.wasm_runtime.load_module(&wasm_bytes)?;
        // Use from_module_with_app_id to enable KV store during initialization
        let wasm_app = WasmApp::from_module_with_app_id(&module, user_id, username, app_id)?;
        
        Ok(wasm_app)
    }
    
    /// Try to load a WebAssembly Component using Wassette runtime
    #[cfg(feature = "wassette")]
    async fn try_load_wassette_app(
        &self,
        app_id: &str,
        user_id: u64,
        username: &str,
        wasm_bytes: &[u8],
    ) -> Result<Arc<dyn Application>> {
        use crate::core::wassette_app::WassetteApplication;
        
        let wassette_runtime = self.wassette_runtime.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Wassette runtime not initialized"))?;
        
        info!("🔧 Loading WebAssembly Component for '{}' ({} bytes)", app_id, wasm_bytes.len());
        
        // Save component to persistent directory (not temp)
        let component_path = if let Some(ref comp_dir) = self.component_dir {
            comp_dir.join(format!("{}.wasm", app_id))
        } else {
            std::path::PathBuf::from(format!("/tmp/mecp-components/{}.wasm", app_id))
        };
        
        // Ensure directory exists
        if let Some(parent) = component_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        // Check if component file already exists and is the same
        let needs_write = if component_path.exists() {
            match tokio::fs::read(&component_path).await {
                Ok(existing_bytes) => existing_bytes != wasm_bytes,
                Err(_) => true,
            }
        } else {
            true
        };
        
        // Only write if file doesn't exist or has changed
        if needs_write {
            tokio::fs::write(&component_path, wasm_bytes).await
                .context("Failed to write component to disk")?;
            info!("💾 Wrote component to: {}", component_path.display());
            
            // Load the component using library API (in-process, no subprocess)
            let uri = format!("file://{}", component_path.display());
            let _load_result = wassette_runtime.load_component(&uri).await
                .context("Failed to load component via Wassette")?;
        } else {
            info!("♻️  Component already on disk: {}", component_path.display());
        }
        
        // Use app_id as component_id (filename without .wasm)
        let component_id = app_id.to_string();
        
        info!("📦 Wassette component ready: {}", component_id);
        
        // Get app metadata from the marketplace
        let app_data = self.app_loader.as_ref()
            .and_then(|loader| {
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(loader.get_application(app_id))
                }).ok().flatten()
            });
        
        let app_name = app_data.as_ref().map(|d| d.name.clone()).unwrap_or_else(|| app_id.to_string());
        let app_description = app_data.and_then(|d| d.description).unwrap_or_else(|| format!("Application: {}", app_id));
        
        // Create WassetteApplication with notification support
        let wassette_app = WassetteApplication::new_with_notifications(
            Arc::clone(wassette_runtime),
            component_id,
            app_id.to_string(),
            app_name,
            app_description,
            user_id,
            username.to_string(),
            self.notifications.clone(),
        ).await?;
        
        Ok(Arc::new(wassette_app))
    }

    /// Get or create a user session
    async fn get_or_create_session(&self, username: &str) -> Result<()> {
        {
            let sessions = self.user_sessions.read().await;
            if sessions.contains_key(username) {
                return Ok(());
            }
        }

        // Load user info
        let user_manager = UserManager::new(&self.mysql_config).await?;
        let user = user_manager.get_user(username).await?
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", username))?;

        if !user.enabled {
            bail!("User '{}' is disabled", username);
        }

        // Load applications (including WASM apps from the database)
        let applications = self.load_user_applications(username, user.id).await?;

        // Initialize native applications (WASM apps are initialized during loading)
        for app in &applications {
            if let Err(e) = app.initialize(user.id).await {
                warn!("⚠️ Failed to initialize app for user {}: {}", username, e);
            }
        }

        info!("✅ Session created for user '{}' with {} applications", username, applications.len());

        // Store session
        let mut sessions = self.user_sessions.write().await;
        sessions.insert(username.to_string(), UserSession {
            user,
            applications,
        });

        Ok(())
    }

    /// Invalidate a user session
    pub async fn invalidate_session(&self, username: &str) {
        let mut sessions = self.user_sessions.write().await;
        sessions.remove(username);
    }
}

#[async_trait]
impl McpConnector for CursorMcpConnector {
    fn connector_id(&self) -> &str {
        "cursor_mcp_connector"
    }

    fn name(&self) -> &str {
        "Cursor MCP Connector"
    }

    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            resources: true,
            resources_subscribe: true,  // We support subscriptions for mailbox
            tools: true,
            prompts: true,
        }
    }

    async fn initialize(&self, user: &UserInfo) -> Result<()> {
        self.get_or_create_session(&user.username).await
    }

    async fn get_tools(&self, username: &str) -> Result<Vec<Box<dyn Tool>>> {
        self.get_or_create_session(username).await?;
        
        let sessions = self.user_sessions.read().await;
        let session = sessions.get(username)
            .ok_or_else(|| anyhow::anyhow!("Session not found for user '{}'", username))?;

        let mut tools: Vec<Box<dyn Tool>> = Vec::new();
        for app in &session.applications {
            tools.extend(app.get_tools().await);
        }

        Ok(tools)
    }

    async fn get_resources(&self, username: &str) -> Result<Vec<Box<dyn Resource>>> {
        self.get_or_create_session(username).await?;
        
        let sessions = self.user_sessions.read().await;
        let session = sessions.get(username)
            .ok_or_else(|| anyhow::anyhow!("Session not found for user '{}'", username))?;

        let mut resources: Vec<Box<dyn Resource>> = Vec::new();
        for app in &session.applications {
            resources.extend(app.get_resources().await);
        }

        Ok(resources)
    }

    async fn subscribe(&self, uri: &str, username: &str) -> Result<()> {
        self.get_or_create_session(username).await?;
        
        let sessions = self.user_sessions.read().await;
        let session = sessions.get(username)
            .ok_or_else(|| anyhow::anyhow!("Session not found for user '{}'", username))?;

        for app in &session.applications {
            app.subscribe(uri, session.user.id).await?;
        }

        Ok(())
    }

    async fn unsubscribe(&self, uri: &str, username: &str) -> Result<()> {
        self.get_or_create_session(username).await?;
        
        let sessions = self.user_sessions.read().await;
        let session = sessions.get(username)
            .ok_or_else(|| anyhow::anyhow!("Session not found for user '{}'", username))?;

        for app in &session.applications {
            app.unsubscribe(uri, session.user.id).await?;
        }

        Ok(())
    }

    async fn call_tool(&self, username: &str, tool_name: &str, params: JsonValue) -> Result<ToolResult> {
        let tools = self.get_tools(username).await?;
        
        for tool in tools {
            let metadata = tool.metadata().await?;
            if metadata.name == tool_name {
                return tool.execute(params).await;
            }
        }
        
        bail!("Tool '{}' not found for user '{}'", tool_name, username)
    }

    async fn read_resource(&self, username: &str, uri: &str) -> Result<ResourceContent> {
        let resources = self.get_resources(username).await?;
        
        for resource in resources {
            if resource.uri().await == uri {
                return resource.read().await;
            }
        }
        
        bail!("Resource '{}' not found for user '{}'", uri, username)
    }
}

/// Connector registry - holds registered connectors
pub struct ConnectorRegistry {
    connectors: HashMap<String, Arc<dyn McpConnector>>,
}

impl ConnectorRegistry {
    /// Create a new connector registry
    pub fn new() -> Self {
        Self {
            connectors: HashMap::new(),
        }
    }

    /// Register a connector
    pub fn register(&mut self, connector: Arc<dyn McpConnector>) {
        self.connectors.insert(connector.connector_id().to_string(), connector);
    }

    /// Get a connector by ID
    pub fn get(&self, connector_id: &str) -> Option<Arc<dyn McpConnector>> {
        self.connectors.get(connector_id).cloned()
    }

    /// Get the default connector (Cursor)
    pub fn get_default(&self) -> Option<Arc<dyn McpConnector>> {
        self.connectors.get("cursor_mcp_connector").cloned()
    }

    /// List all registered connector IDs
    pub fn list_connector_ids(&self) -> Vec<String> {
        self.connectors.keys().cloned().collect()
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}
