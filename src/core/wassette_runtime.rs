//! Wassette-Compatible Runtime Integration for MeCP
//!
//! This module provides a WebAssembly Component Model runtime for executing
//! WebAssembly Components directly using wasmtime, inspired by Microsoft's Wassette.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     MeCP Host Runtime                           │
//! │  ┌─────────────────┐    ┌─────────────────┐                    │
//! │  │ WassetteRuntime │───▶│ WassetteApp     │                    │
//! │  │ (Library)       │    │ (Component)     │                    │
//! │  └─────────────────┘    └─────────────────┘                    │
//! │         │                      │                               │
//! │   Direct wasmtime API    Tool calls via Component Model        │
//! │         ▼                      ▼                               │
//! │  ┌─────────────────────────────────────────────────────────────┐ │
//! │  │         wasmtime Component Model Runtime                    │ │
//! │  │  - In-process WebAssembly execution                         │ │
//! │  │  - WASI support via wasmtime-wasi                           │ │
//! │  │  - Component persistence & caching                          │ │
//! │  └─────────────────────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - Load WebAssembly Components directly
//! - Auto-discover tools from WIT interfaces
//! - Execute tool calls with JSON parameters
//! - Component lifecycle management

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use wasmtime::{Engine, Config, Store};
use wasmtime::component::{Component, Linker, Val};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, ResourceTable};

use crate::core::types::{ToolResult, ToolParameter};
use crate::core::message_broker::AppKvStore;
use crate::services::config::RedisConfig;
use crate::tools::{Tool, ToolMetadata};

// =============================================================================
// WASI State for Component Execution
// =============================================================================

/// WASI state for WebAssembly Component execution
pub struct WasiState {
    ctx: WasiCtx,
    table: ResourceTable,
    /// KV storage for this component instance (optional)
    kv_store: Option<Arc<AppKvStore>>,
}

impl WasiState {
    fn new() -> Self {
        Self {
            ctx: WasiCtxBuilder::new()
                .inherit_stdio()
                .build(),
            table: ResourceTable::new(),
            kv_store: None,
        }
    }
    
    fn with_kv_store(kv_store: Arc<AppKvStore>) -> Self {
        Self {
            ctx: WasiCtxBuilder::new()
                .inherit_stdio()
                .build(),
            table: ResourceTable::new(),
            kv_store: Some(kv_store),
        }
    }
}

impl WasiView for WasiState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

// =============================================================================
// Component Instance
// =============================================================================

/// A loaded WebAssembly Component (compiled, ready to instantiate)
/// 
/// Each tool call creates a fresh instance, ensuring:
/// - User isolation (tony's mailbox separate from bob's)
/// - Stateless execution (state should be in external storage like Redis/MessageBroker)
struct LoadedComponent {
    /// Compiled component (can be instantiated multiple times)
    component: Component,
    /// Tool metadata discovered from the component
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Clone)]
struct ToolInfo {
    name: String,
    description: String,
    input_schema: JsonValue,
    /// Interface and function path for calling
    interface_name: Option<String>,
    function_name: String,
    /// Parameter names (extracted from WIT or generated)
    param_names: Vec<String>,
}

// =============================================================================
// Wassette Runtime Engine
// =============================================================================

/// Wassette-compatible runtime for WebAssembly Components
///
/// This runtime uses wasmtime directly to execute WebAssembly Components
/// that follow the Component Model (WIT interfaces).
pub struct WassetteRuntime {
    /// Wasmtime engine configured for components
    engine: Engine,
    /// Component directory for storing downloaded/compiled components
    component_dir: PathBuf,
    /// Loaded components by ID
    components: Arc<RwLock<HashMap<String, LoadedComponent>>>,
    /// Tool to component mapping
    tool_to_component: Arc<RwLock<HashMap<String, String>>>,
    /// Redis config for KV storage (optional)
    redis_config: Option<RedisConfig>,
}

impl WassetteRuntime {
    /// Create a new Wassette-compatible runtime
    ///
    /// # Arguments
    /// * `component_dir` - Directory for storing WebAssembly components
    pub async fn new(component_dir: impl AsRef<Path>) -> Result<Self> {
        Self::new_with_redis(component_dir, None).await
    }
    
    /// Create a new Wassette-compatible runtime with Redis support for KV storage
    ///
    /// # Arguments
    /// * `component_dir` - Directory for storing WebAssembly components
    /// * `redis_config` - Optional Redis configuration for KV storage
    pub async fn new_with_redis(component_dir: impl AsRef<Path>, redis_config: Option<RedisConfig>) -> Result<Self> {
        let component_dir = component_dir.as_ref().to_path_buf();
        
        // Ensure the component directory exists
        tokio::fs::create_dir_all(&component_dir).await
            .context("Failed to create component directory")?;
        
        // Configure wasmtime engine for components
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        
        let engine = Engine::new(&config)
            .context("Failed to create wasmtime engine")?;
        
        let runtime = Self {
            engine,
            component_dir,
            components: Arc::new(RwLock::new(HashMap::new())),
            tool_to_component: Arc::new(RwLock::new(HashMap::new())),
            redis_config,
        };
        
        // Load existing components from the directory
        runtime.load_existing_components().await?;
        
        info!("🔧 Wassette Runtime initialized (library mode)");
        info!("   Component dir: {}", runtime.component_dir.display());
        if redis_config.is_some() {
            info!("   Redis KV storage: enabled");
        } else {
            info!("   Redis KV storage: disabled (in-memory only)");
        }
        
        Ok(runtime)
    }
    
    /// Create a new Wassette-compatible runtime without loading existing components
    pub async fn new_unloaded(component_dir: impl AsRef<Path>) -> Result<Self> {
        Self::new_unloaded_with_redis(component_dir, None).await
    }
    
    /// Create a new Wassette-compatible runtime without loading existing components, with Redis support
    pub async fn new_unloaded_with_redis(component_dir: impl AsRef<Path>, redis_config: Option<RedisConfig>) -> Result<Self> {
        let component_dir = component_dir.as_ref().to_path_buf();
        
        // Ensure the component directory exists
        tokio::fs::create_dir_all(&component_dir).await
            .context("Failed to create component directory")?;
        
        // Configure wasmtime engine for components
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        
        let engine = Engine::new(&config)
            .context("Failed to create wasmtime engine")?;
        
        info!("🔧 Wassette Runtime initialized (library mode, lazy loading)");
        info!("   Component dir: {}", component_dir.display());
        if redis_config.is_some() {
            info!("   Redis KV storage: enabled");
        } else {
            info!("   Redis KV storage: disabled (in-memory only)");
        }
        
        Ok(Self {
            engine,
            component_dir,
            components: Arc::new(RwLock::new(HashMap::new())),
            tool_to_component: Arc::new(RwLock::new(HashMap::new())),
            redis_config,
        })
    }
    
    /// Create a KV store for a component instance
    pub async fn create_kv_store(&self, component_id: &str, user_id: u64) -> Result<Arc<AppKvStore>> {
        if let Some(ref config) = self.redis_config {
            let kv_store = AppKvStore::new(config, component_id, &user_id.to_string()).await
                .context("Failed to create KV store")?;
            Ok(Arc::new(kv_store))
        } else {
            // Return a dummy KV store that does nothing (in-memory mode)
            // AppKvStore will handle this gracefully
            let dummy_config = RedisConfig::default();
            let kv_store = AppKvStore::new(&dummy_config, component_id, &user_id.to_string()).await
                .context("Failed to create KV store")?;
            Ok(Arc::new(kv_store))
        }
    }
    
    /// Get the component directory path
    pub fn component_dir(&self) -> &Path {
        &self.component_dir
    }
    
    /// Load existing components from the component directory
    async fn load_existing_components(&self) -> Result<()> {
        let mut entries = tokio::fs::read_dir(&self.component_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            // Only process .wasm files
            if path.extension().and_then(|e| e.to_str()) != Some("wasm") {
                continue;
            }
            
            // Extract component ID from filename
            let component_id = path.file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
                .unwrap_or_default();
            
            if component_id.is_empty() {
                continue;
            }
            
            // Try to load the component
            match self.load_component_from_path(&path, &component_id).await {
                Ok(_) => info!("📦 Auto-loaded component: {}", component_id),
                Err(e) => warn!("⚠️ Failed to auto-load component {}: {}", component_id, e),
            }
        }
        
        Ok(())
    }
    
    /// Load a component from a URI
    ///
    /// # Arguments
    /// * `uri` - URI to load the component from (file://, https://)
    pub async fn load_component(&self, uri: &str) -> Result<LoadResult> {
        // Parse URI and get bytes
        let (component_id, wasm_bytes) = if uri.starts_with("file://") {
            let path = PathBuf::from(uri.strip_prefix("file://").unwrap());
            let bytes = tokio::fs::read(&path).await
                .with_context(|| format!("Failed to read component from: {}", path.display()))?;
            
            let id = path.file_stem()
                .and_then(|s| s.to_str())
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string());
            
            (id, bytes)
        } else if uri.starts_with("http://") || uri.starts_with("https://") {
            let response = reqwest::get(uri).await
                .with_context(|| format!("Failed to download component from: {}", uri))?;
            
            let bytes = response.bytes().await
                .context("Failed to read component bytes")?
                .to_vec();
            
            // Extract ID from URL path
            let id = uri.rsplit('/').next()
                .and_then(|s| s.strip_suffix(".wasm"))
                .map(String::from)
                .unwrap_or_else(|| "downloaded".to_string());
            
            (id, bytes)
        } else {
            bail!("Unsupported URI scheme: {}", uri);
        };
        
        // Check if component already exists
        let status = {
            let components = self.components.read().await;
            if components.contains_key(&component_id) {
                LoadStatus::Replaced
            } else {
                LoadStatus::New
            }
        };
        
        // Save component to disk
        let component_path = self.component_dir.join(format!("{}.wasm", component_id));
        tokio::fs::write(&component_path, &wasm_bytes).await
            .context("Failed to save component to disk")?;
        
        // Load and compile the component
        let tools = self.load_component_from_bytes(&wasm_bytes, &component_id).await?;
        
        let tool_names = tools.iter().map(|t| t.name.clone()).collect();
        
        info!("📦 Loaded component: {} ({:?})", component_id, status);
        
        Ok(LoadResult {
            component_id,
            status,
            tool_names,
        })
    }
    
    /// Load a component from a file path
    async fn load_component_from_path(&self, path: &Path, component_id: &str) -> Result<Vec<ToolInfo>> {
        let wasm_bytes = tokio::fs::read(path).await
            .with_context(|| format!("Failed to read component from: {}", path.display()))?;
        
        self.load_component_from_bytes(&wasm_bytes, component_id).await
    }
    
    /// Load a component from bytes
    async fn load_component_from_bytes(&self, wasm_bytes: &[u8], component_id: &str) -> Result<Vec<ToolInfo>> {
        // Compile the component
        let component = Component::from_binary(&self.engine, wasm_bytes)
            .context("Failed to compile WebAssembly Component")?;
        
        // Extract tools from the component's exports
        let tools = self.extract_tools_from_component(&component, component_id)?;
        
        // Store the compiled component (can be instantiated multiple times)
        {
            let mut components = self.components.write().await;
            components.insert(component_id.to_string(), LoadedComponent {
                component,
                tools: tools.clone(),
            });
        }
        
        // Update tool to component mapping
        {
            let mut mapping = self.tool_to_component.write().await;
            for tool in &tools {
                mapping.insert(tool.name.clone(), component_id.to_string());
            }
        }
        
        Ok(tools)
    }
    
    /// Extract tool information from a component's exports
    fn extract_tools_from_component(&self, component: &Component, component_id: &str) -> Result<Vec<ToolInfo>> {
        let mut tools = Vec::new();
        
        // Get component type to inspect exports
        let component_type = component.component_type();
        
        // Iterate over exports
        for (name, export) in component_type.exports(&self.engine) {
            match export {
                wasmtime::component::types::ComponentItem::ComponentFunc(func_type) => {
                    // Direct function export
                    let tool = self.create_tool_info_from_func(
                        &name,
                        None,
                        &name,
                        &func_type,
                        component_id,
                    );
                    tools.push(tool);
                }
                wasmtime::component::types::ComponentItem::ComponentInstance(instance_type) => {
                    // Interface export - iterate over its functions
                    for (func_name, item) in instance_type.exports(&self.engine) {
                        if let wasmtime::component::types::ComponentItem::ComponentFunc(func_type) = item {
                            // Create normalized tool name
                            let normalized_name = Self::normalize_tool_name(&name, &func_name);
                            
                            let tool = self.create_tool_info_from_func(
                                &normalized_name,
                                Some(name.to_string()),
                                &func_name,
                                &func_type,
                                component_id,
                            );
                            tools.push(tool);
                        }
                    }
                }
                _ => {} // Skip other export types (types, resources, etc.)
            }
        }
        
        debug!("Extracted {} tools from component {}", tools.len(), component_id);
        Ok(tools)
    }
    
    /// Create tool info from a function type
    fn create_tool_info_from_func(
        &self,
        name: &str,
        interface_name: Option<String>,
        function_name: &str,
        func_type: &wasmtime::component::types::ComponentFunc,
        _component_id: &str,
    ) -> ToolInfo {
        // Build JSON schema for input parameters
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        let mut param_names = Vec::new();
        
        // Get parameter types and generate names
        for (idx, param_type) in func_type.params().enumerate() {
            // Generate parameter name (param0, param1, etc. since wasmtime v27 doesn't provide names)
            let param_name = format!("param{}", idx);
            let schema = self.component_type_to_json_schema(&param_type);
            properties.insert(param_name.clone(), schema);
            required.push(JsonValue::String(param_name.clone()));
            param_names.push(param_name);
        }
        
        let input_schema = serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required,
        });
        
        ToolInfo {
            name: name.to_string(),
            description: format!("Function exported from WebAssembly Component"),
            input_schema,
            interface_name,
            function_name: function_name.to_string(),
            param_names,
        }
    }
    
    /// Convert a component type to JSON schema
    fn component_type_to_json_schema(&self, val_type: &wasmtime::component::types::Type) -> JsonValue {
        use wasmtime::component::types::Type;
        
        match val_type {
            Type::Bool => serde_json::json!({"type": "boolean"}),
            Type::S8 | Type::S16 | Type::S32 | Type::S64 |
            Type::U8 | Type::U16 | Type::U32 | Type::U64 => {
                serde_json::json!({"type": "integer"})
            }
            Type::Float32 | Type::Float64 => serde_json::json!({"type": "number"}),
            Type::Char | Type::String => serde_json::json!({"type": "string"}),
            Type::List(inner) => serde_json::json!({
                "type": "array",
                "items": self.component_type_to_json_schema(&inner.ty())
            }),
            Type::Option(inner) => {
                // Option types are nullable
                let inner_schema = self.component_type_to_json_schema(&inner.ty());
                serde_json::json!({
                    "oneOf": [inner_schema, {"type": "null"}]
                })
            }
            Type::Record(record) => {
                let mut props = serde_json::Map::new();
                let mut req = Vec::new();
                for field in record.fields() {
                    props.insert(field.name.to_string(), self.component_type_to_json_schema(&field.ty));
                    req.push(JsonValue::String(field.name.to_string()));
                }
                serde_json::json!({
                    "type": "object",
                    "properties": props,
                    "required": req,
                })
            }
            _ => serde_json::json!({"type": "string"}) // Fallback
        }
    }
    
    /// Normalize tool name (interface + function name)
    fn normalize_tool_name(interface: &str, function: &str) -> String {
        // Convert from "local:package/interface" to "local_package_interface_function"
        let normalized_interface = interface
            .replace(':', "_")
            .replace('/', "_")
            .replace('-', "-");
        format!("{}_{}", normalized_interface, function)
    }
    
    /// Unload a component by its ID
    pub async fn unload_component(&self, component_id: &str) -> Result<()> {
        // Remove from components
        let tool_names = {
            let mut components = self.components.write().await;
            if let Some(instance) = components.remove(component_id) {
                instance.tools.iter().map(|t| t.name.clone()).collect::<Vec<_>>()
            } else {
                bail!("Component not found: {}", component_id);
            }
        };
        
        // Remove tool mappings
        {
            let mut mapping = self.tool_to_component.write().await;
            for name in tool_names {
                mapping.remove(&name);
            }
        }
        
        // Remove from disk
        let component_path = self.component_dir.join(format!("{}.wasm", component_id));
        if component_path.exists() {
            tokio::fs::remove_file(&component_path).await?;
        }
        
        info!("📦 Unloaded component: {}", component_id);
        Ok(())
    }
    
    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<JsonValue>> {
        let components = self.components.read().await;
        
        let mut tools = Vec::new();
        for instance in components.values() {
            for tool in &instance.tools {
                tools.push(serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "inputSchema": tool.input_schema,
                }));
            }
        }
        
        Ok(tools)
    }
    
    /// List all loaded components
    pub async fn list_components(&self) -> Vec<String> {
        let components = self.components.read().await;
        components.keys().cloned().collect()
    }
    
    /// Call a tool by name
    pub async fn call_tool(&self, tool_name: &str, arguments: &JsonValue) -> Result<String> {
        self.call_tool_with_user(tool_name, arguments, None, None).await
    }
    
    /// Call a tool by name with user context (enables KV storage)
    pub async fn call_tool_with_user(
        &self,
        tool_name: &str,
        arguments: &JsonValue,
        component_id: Option<&str>,
        user_id: Option<u64>,
    ) -> Result<String> {
        // Find the component and tool info
        let (component_id, tool_info) = {
            let mapping = self.tool_to_component.read().await;
            let comp_id = component_id
                .map(String::from)
                .or_else(|| mapping.get(tool_name).cloned())
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;
            
            let components = self.components.read().await;
            let instance = components.get(&comp_id)
                .ok_or_else(|| anyhow::anyhow!("Component not found: {}", comp_id))?;
            
            let tool = instance.tools.iter()
                .find(|t| t.name == tool_name)
                .ok_or_else(|| anyhow::anyhow!("Tool not found in component: {}", tool_name))?
                .clone();
            
            (comp_id, tool)
        };
        
        // Execute the tool with optional KV store
        let kv_store = if let Some(uid) = user_id {
            Some(self.create_kv_store(&component_id, uid).await?)
        } else {
            None
        };
        
        self.execute_tool(&component_id, &tool_info, arguments, kv_store).await
    }
    
    /// Execute a tool on a component
    /// 
    /// Each call creates a fresh instance to ensure:
    /// - User isolation (different users have separate state)
    /// - Stateless execution (state should be persisted externally via MessageBroker/Redis)
    async fn execute_tool(
        &self,
        component_id: &str,
        tool_info: &ToolInfo,
        arguments: &JsonValue,
        kv_store: Option<Arc<AppKvStore>>,
    ) -> Result<String> {
        // Get the loaded component
        let components = self.components.read().await;
        let loaded = components.get(component_id)
            .ok_or_else(|| anyhow::anyhow!("Component not found: {}", component_id))?;
        
        // Create a fresh instance for this call (ensures user isolation)
        debug!("Creating fresh instance for component: {} (KV store: {})", 
            component_id, if kv_store.is_some() { "enabled" } else { "disabled" });
        
        let mut linker: Linker<WasiState> = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker_async(&mut linker)
            .context("Failed to add WASI to linker")?;
        
        // Add KV storage host functions if KV store is available
        if let Some(ref kv) = kv_store {
            Self::add_kv_storage_to_linker(&mut linker, Arc::clone(kv))
                .context("Failed to add KV storage to linker")?;
        }
        
        let wasi_state = if let Some(kv) = kv_store {
            WasiState::with_kv_store(kv)
        } else {
            WasiState::new()
        };
        
        let mut store = Store::new(&self.engine, wasi_state);
        
        // Instantiate the component
        let instance = linker.instantiate_async(&mut store, &loaded.component).await
            .context("Failed to instantiate component")?;
        
        // Get the function - for interface exports, we need to navigate through the interface
        let func = if let Some(ref interface_name) = tool_info.interface_name {
            // For interface exports like "local:mailbox/mailbox-ops", we need to:
            // 1. Get the interface export index
            // 2. Get the function export index from within that interface
            // 3. Get the actual function using the function export index
            let interface_export = instance.get_export(&mut store, None, interface_name)
                .ok_or_else(|| anyhow::anyhow!("Interface not found: {}", interface_name))?;
            
            // Get the function export from the interface
            let func_export = instance.get_export(&mut store, Some(&interface_export), &tool_info.function_name)
                .ok_or_else(|| anyhow::anyhow!("Function '{}' not found in interface '{}'", 
                    tool_info.function_name, interface_name))?;
            
            // Now get the actual function using the export index
            instance.get_func(&mut store, func_export)
                .ok_or_else(|| anyhow::anyhow!("Failed to get function '{}' from export", 
                    tool_info.function_name))?
        } else {
            // Get direct function export (no interface) - use function name as the lookup
            instance.get_func(&mut store, &tool_info.function_name as &str)
                .ok_or_else(|| anyhow::anyhow!("Function not found: {}", tool_info.function_name))?
        };
        
        // Convert JSON arguments to component values
        let func_param_types = func.params(&store);
        let mut params = Vec::new();
        
        for (idx, param_type) in func_param_types.iter().enumerate() {
            let param_name = &tool_info.param_names[idx];
            let value = arguments.get(param_name)
                .ok_or_else(|| anyhow::anyhow!("Missing parameter: {}", param_name))?;
            
            let component_val = self.json_to_component_val(value, param_type)?;
            params.push(component_val);
        }
        
        // Create result storage
        let results_count = func.results(&store).len();
        let mut results = vec![Val::Bool(false); results_count];
        
        // Call the function
        func.call_async(&mut store, &params, &mut results).await
            .context("Failed to call function")?;
        func.post_return_async(&mut store).await
            .context("Failed to post-return")?;
        
        // Convert results to JSON
        let result_json = if results.len() == 1 {
            self.component_val_to_json(&results[0])?
        } else if results.is_empty() {
            JsonValue::Null
        } else {
            let arr: Vec<JsonValue> = results.iter()
                .map(|v| self.component_val_to_json(v))
                .collect::<Result<Vec<_>>>()?;
            JsonValue::Array(arr)
        };
        
        // Wrap in result object (compatible with Wassette format)
        let wrapped = serde_json::json!({ "result": result_json });
        Ok(serde_json::to_string(&wrapped)?)
    }
    
    /// Convert JSON value to component value based on expected type
    fn json_to_component_val(
        &self,
        value: &JsonValue,
        val_type: &wasmtime::component::types::Type,
    ) -> Result<Val> {
        use wasmtime::component::types::Type;
        
        match val_type {
            // Boolean type
            Type::Bool => match value {
                JsonValue::Bool(b) => Ok(Val::Bool(*b)),
                JsonValue::String(s) => {
                    // Handle string-to-bool conversion
                    let lower = s.to_lowercase();
                    match lower.as_str() {
                        "true" | "1" | "yes" => Ok(Val::Bool(true)),
                        "false" | "0" | "no" | "" => Ok(Val::Bool(false)),
                        _ => bail!("Cannot convert string '{}' to bool", s),
                    }
                }
                JsonValue::Number(n) => Ok(Val::Bool(n.as_i64().unwrap_or(0) != 0)),
                _ => bail!("Cannot convert {:?} to bool", value),
            },
            
            // Signed integers
            Type::S8 => self.json_to_s64(value).map(|n| Val::S8(n as i8)),
            Type::S16 => self.json_to_s64(value).map(|n| Val::S16(n as i16)),
            Type::S32 => self.json_to_s64(value).map(|n| Val::S32(n as i32)),
            Type::S64 => self.json_to_s64(value).map(Val::S64),
            
            // Unsigned integers
            Type::U8 => self.json_to_u64(value).map(|n| Val::U8(n as u8)),
            Type::U16 => self.json_to_u64(value).map(|n| Val::U16(n as u16)),
            Type::U32 => self.json_to_u64(value).map(|n| Val::U32(n as u32)),
            Type::U64 => self.json_to_u64(value).map(Val::U64),
            
            // Floats
            Type::Float32 => self.json_to_f64(value).map(|f| Val::Float32(f as f32)),
            Type::Float64 => self.json_to_f64(value).map(Val::Float64),
            
            // Char - take first character of string
            Type::Char => match value {
                JsonValue::String(s) => s.chars().next()
                    .map(Val::Char)
                    .ok_or_else(|| anyhow::anyhow!("Empty string cannot be converted to char")),
                _ => bail!("Cannot convert {:?} to char", value),
            },
            
            // String
            Type::String => match value {
                JsonValue::String(s) => Ok(Val::String(s.clone())),
                JsonValue::Number(n) => Ok(Val::String(n.to_string())),
                JsonValue::Bool(b) => Ok(Val::String(b.to_string())),
                JsonValue::Null => Ok(Val::String(String::new())),
                _ => bail!("Cannot convert {:?} to string", value),
            },
            
            // Option type
            Type::Option(inner) => {
                // Treat null and empty strings as None
                let is_none = value.is_null() || 
                    matches!(value, JsonValue::String(s) if s.is_empty());
                
                if is_none {
                    // None case
                    Ok(Val::Option(None))
                } else {
                    // Some case - recursively convert the inner value
                    let inner_val = self.json_to_component_val(value, &inner.ty())?;
                    Ok(Val::Option(Some(Box::new(inner_val))))
                }
            },
            
            // Result type
            Type::Result(result_type) => {
                // Check if value is an object with "ok" or "err" fields
                if let JsonValue::Object(obj) = value {
                    if let Some(ok_val) = obj.get("ok") {
                        if let Some(ok_type) = result_type.ok() {
                            let inner_val = self.json_to_component_val(ok_val, &ok_type)?;
                            return Ok(Val::Result(Ok(Some(Box::new(inner_val)))));
                        }
                        return Ok(Val::Result(Ok(None)));
                    }
                    if let Some(err_val) = obj.get("err") {
                        if let Some(err_type) = result_type.err() {
                            let inner_val = self.json_to_component_val(err_val, &err_type)?;
                            return Ok(Val::Result(Err(Some(Box::new(inner_val)))));
                        }
                        return Ok(Val::Result(Err(None)));
                    }
                }
                // Default: treat the entire value as Ok
                if let Some(ok_type) = result_type.ok() {
                    let inner_val = self.json_to_component_val(value, &ok_type)?;
                    Ok(Val::Result(Ok(Some(Box::new(inner_val)))))
                } else {
                    Ok(Val::Result(Ok(None)))
                }
            },
            
            // List type
            Type::List(list_type) => {
                if let JsonValue::Array(arr) = value {
                    let inner_ty = list_type.ty();
                    let items: Result<Vec<Val>> = arr.iter()
                        .map(|v| self.json_to_component_val(v, &inner_ty))
                        .collect();
                    Ok(Val::List(items?))
                } else {
                    bail!("Expected array for list type, got {:?}", value)
                }
            },
            
            // Record type
            Type::Record(record_type) => {
                if let JsonValue::Object(obj) = value {
                    let mut fields = Vec::new();
                    for field in record_type.fields() {
                        let field_value = obj.get(field.name)
                            .ok_or_else(|| anyhow::anyhow!("Missing field: {}", field.name))?;
                        let field_val = self.json_to_component_val(field_value, &field.ty)?;
                        fields.push((field.name.to_string(), field_val));
                    }
                    Ok(Val::Record(fields))
                } else {
                    bail!("Expected object for record type, got {:?}", value)
                }
            },
            
            // Tuple type
            Type::Tuple(tuple_type) => {
                if let JsonValue::Array(arr) = value {
                    let types: Vec<_> = tuple_type.types().collect();
                    if arr.len() != types.len() {
                        bail!("Tuple length mismatch: expected {}, got {}", types.len(), arr.len());
                    }
                    let items: Result<Vec<Val>> = arr.iter().zip(types.iter())
                        .map(|(v, t)| self.json_to_component_val(v, t))
                        .collect();
                    Ok(Val::Tuple(items?))
                } else {
                    bail!("Expected array for tuple type, got {:?}", value)
                }
            },
            
            // Variant and Enum types
            Type::Variant(variant_type) => {
                // Handle variant as object with single key
                if let JsonValue::Object(obj) = value {
                    if obj.len() == 1 {
                        let (case_name, case_value) = obj.iter().next().unwrap();
                        for case in variant_type.cases() {
                            if case.name == case_name {
                                if let Some(case_ty) = case.ty {
                                    let inner_val = self.json_to_component_val(case_value, &case_ty)?;
                                    return Ok(Val::Variant(case_name.clone(), Some(Box::new(inner_val))));
                                } else {
                                    return Ok(Val::Variant(case_name.clone(), None));
                                }
                            }
                        }
                        bail!("Unknown variant case: {}", case_name);
                    }
                }
                // Also handle as a simple string for enums
                if let JsonValue::String(s) = value {
                    for case in variant_type.cases() {
                        if case.name == s.as_str() {
                            return Ok(Val::Variant(s.clone(), None));
                        }
                    }
                    bail!("Unknown variant case: {}", s);
                }
                bail!("Cannot convert {:?} to variant", value)
            },
            
            Type::Enum(enum_type) => {
                if let JsonValue::String(s) = value {
                    for name in enum_type.names() {
                        if name == s.as_str() {
                            return Ok(Val::Enum(s.clone()));
                        }
                    }
                    bail!("Unknown enum value: {}", s);
                }
                bail!("Expected string for enum type, got {:?}", value)
            },
            
            Type::Flags(flags_type) => {
                // Flags can be an array of strings or a single string
                let flag_names: Vec<String> = match value {
                    JsonValue::Array(arr) => arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect(),
                    JsonValue::String(s) => s.split(',').map(|s| s.trim().to_string()).collect(),
                    _ => bail!("Expected array or string for flags type"),
                };
                
                // Validate flag names
                let valid_names: Vec<_> = flags_type.names().collect();
                for name in &flag_names {
                    if !valid_names.contains(&name.as_str()) {
                        bail!("Unknown flag: {}", name);
                    }
                }
                
                Ok(Val::Flags(flag_names))
            },
            
            // Borrow and Own - treat as the inner type
            Type::Borrow(_) | Type::Own(_) => {
                bail!("Resource types (borrow/own) not supported in JSON conversion")
            },
        }
    }
    
    /// Helper to convert JSON to i64
    fn json_to_s64(&self, value: &JsonValue) -> Result<i64> {
        match value {
            JsonValue::Number(n) => n.as_i64()
                .ok_or_else(|| anyhow::anyhow!("Number out of i64 range")),
            JsonValue::String(s) => s.parse::<i64>()
                .map_err(|_| anyhow::anyhow!("Cannot parse '{}' as integer", s)),
            JsonValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => bail!("Cannot convert {:?} to integer", value),
        }
    }
    
    /// Helper to convert JSON to u64
    fn json_to_u64(&self, value: &JsonValue) -> Result<u64> {
        match value {
            JsonValue::Number(n) => n.as_u64()
                .ok_or_else(|| anyhow::anyhow!("Number out of u64 range")),
            JsonValue::String(s) => s.parse::<u64>()
                .map_err(|_| anyhow::anyhow!("Cannot parse '{}' as unsigned integer", s)),
            JsonValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            _ => bail!("Cannot convert {:?} to unsigned integer", value),
        }
    }
    
    /// Helper to convert JSON to f64
    fn json_to_f64(&self, value: &JsonValue) -> Result<f64> {
        match value {
            JsonValue::Number(n) => n.as_f64()
                .ok_or_else(|| anyhow::anyhow!("Cannot convert number to f64")),
            JsonValue::String(s) => s.parse::<f64>()
                .map_err(|_| anyhow::anyhow!("Cannot parse '{}' as float", s)),
            _ => bail!("Cannot convert {:?} to float", value),
        }
    }
    
    /// Convert component value to JSON
    fn component_val_to_json(&self, val: &Val) -> Result<JsonValue> {
        match val {
            Val::Bool(b) => Ok(JsonValue::Bool(*b)),
            Val::S8(n) => Ok(JsonValue::Number((*n as i64).into())),
            Val::S16(n) => Ok(JsonValue::Number((*n as i64).into())),
            Val::S32(n) => Ok(JsonValue::Number((*n as i64).into())),
            Val::S64(n) => Ok(JsonValue::Number((*n).into())),
            Val::U8(n) => Ok(JsonValue::Number((*n as u64).into())),
            Val::U16(n) => Ok(JsonValue::Number((*n as u64).into())),
            Val::U32(n) => Ok(JsonValue::Number((*n as u64).into())),
            Val::U64(n) => Ok(JsonValue::Number((*n).into())),
            Val::Float32(f) => Ok(serde_json::Number::from_f64(*f as f64)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)),
            Val::Float64(f) => Ok(serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)),
            Val::Char(c) => Ok(JsonValue::String(c.to_string())),
            Val::String(s) => Ok(JsonValue::String(s.clone())),
            
            // Option type - None becomes null, Some(x) becomes the inner value
            Val::Option(opt) => match opt {
                None => Ok(JsonValue::Null),
                Some(inner) => self.component_val_to_json(inner),
            },
            
            // Result type - convert to {"ok": value} or {"error": value}
            Val::Result(res) => match res {
                Ok(Some(inner)) => {
                    let inner_json = self.component_val_to_json(inner)?;
                    Ok(serde_json::json!({"ok": inner_json}))
                }
                Ok(None) => Ok(serde_json::json!({"ok": null})),
                Err(Some(inner)) => {
                    let inner_json = self.component_val_to_json(inner)?;
                    Ok(serde_json::json!({"error": inner_json}))
                }
                Err(None) => Ok(serde_json::json!({"error": null})),
            },
            
            // List type - convert to JSON array
            Val::List(items) => {
                let arr: Result<Vec<JsonValue>> = items.iter()
                    .map(|v| self.component_val_to_json(v))
                    .collect();
                Ok(JsonValue::Array(arr?))
            },
            
            // Record type - convert to JSON object
            Val::Record(fields) => {
                let mut obj = serde_json::Map::new();
                for (name, value) in fields {
                    // Convert kebab-case to snake_case for JSON
                    let json_name = name.replace('-', "_");
                    obj.insert(json_name, self.component_val_to_json(value)?);
                }
                Ok(JsonValue::Object(obj))
            },
            
            // Tuple type - convert to JSON array
            Val::Tuple(items) => {
                let arr: Result<Vec<JsonValue>> = items.iter()
                    .map(|v| self.component_val_to_json(v))
                    .collect();
                Ok(JsonValue::Array(arr?))
            },
            
            // Variant type - convert to {"variant_name": value} or just "variant_name"
            Val::Variant(name, payload) => match payload {
                Some(inner) => {
                    let inner_json = self.component_val_to_json(inner)?;
                    Ok(serde_json::json!({name: inner_json}))
                }
                None => Ok(JsonValue::String(name.clone())),
            },
            
            // Enum type - just the name as string
            Val::Enum(name) => Ok(JsonValue::String(name.clone())),
            
            // Flags type - array of flag names
            Val::Flags(flags) => {
                let arr: Vec<JsonValue> = flags.iter()
                    .map(|f| JsonValue::String(f.clone()))
                    .collect();
                Ok(JsonValue::Array(arr))
            },
            
            // Resource types - not directly convertible
            Val::Resource(_) => Ok(serde_json::json!({"resource": "opaque"})),
        }
    }
    
    /// Add KV storage host functions to the linker
    /// 
    /// This adds the mecp:kv-storage interface functions to the linker
    /// so components can import and use KV storage.
    fn add_kv_storage_to_linker(linker: &mut Linker<WasiState>, kv_store: Arc<AppKvStore>) -> Result<()> {
        use wasmtime::component::Resource;
        
        // Note: For wasmtime component model, we need to manually implement
        // the host functions. However, wasmtime's component model API doesn't
        // directly support adding arbitrary host functions like the core module API.
        // 
        // For now, we'll store the KV store in WasiState and components can access it
        // through a different mechanism. The proper way would be to:
        // 1. Define the WIT interface (done in wit/mecp-kv-storage.wit)
        // 2. Generate bindings using wit-bindgen
        // 3. Implement the host functions using wasmtime's component API
        //
        // For the immediate implementation, we'll use a workaround where components
        // can call KV functions through a special tool interface, or we can
        // implement it properly using wasmtime's component linker API.
        //
        // TODO: Implement proper WIT interface binding for KV storage
        
        // For now, the KV store is stored in WasiState and can be accessed
        // by components through a future proper WIT interface implementation
        Ok(())
    }
    
    /// Shutdown the runtime (no-op for library mode)
    pub async fn shutdown(&self) -> Result<()> {
        info!("Wassette runtime shutdown");
        Ok(())
    }
}

// =============================================================================
// Result Types
// =============================================================================

/// Result of loading a component
#[derive(Debug, Clone)]
pub struct LoadResult {
    pub component_id: String,
    pub status: LoadStatus,
    pub tool_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadStatus {
    New,
    Replaced,
}

// =============================================================================
// WassetteApp - Wrapper for tools from a loaded component
// =============================================================================

/// Represents a loaded Wassette component
pub struct WassetteApp {
    runtime: Arc<WassetteRuntime>,
    component_id: String,
    tool_names: Vec<String>,
}

impl WassetteApp {
    /// Create a new WassetteApp
    pub fn new(
        runtime: Arc<WassetteRuntime>,
        component_id: String,
        tool_names: Vec<String>,
    ) -> Self {
        Self {
            runtime,
            component_id,
            tool_names,
        }
    }
    
    /// Get the component ID
    pub fn component_id(&self) -> &str {
        &self.component_id
    }
    
    /// Get available tool names
    pub fn tool_names(&self) -> &[String] {
        &self.tool_names
    }
    
    /// Call a tool
    pub async fn call_tool(&self, tool_name: &str, params: JsonValue) -> Result<ToolResult> {
        match self.runtime.call_tool(tool_name, &params).await {
            Ok(result_str) => {
                let output = serde_json::from_str(&result_str)
                    .unwrap_or_else(|_| JsonValue::String(result_str));
                
                Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    output: JsonValue::Null,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

// =============================================================================
// WassetteTool - Tool trait implementation
// =============================================================================

/// Wrapper that implements the Tool trait for Wassette tools
pub struct WassetteTool {
    runtime: Arc<WassetteRuntime>,
    name: String,
    description: String,
    input_schema: JsonValue,
}

impl WassetteTool {
    /// Create a new WassetteTool
    pub fn new(
        runtime: Arc<WassetteRuntime>,
        name: String,
        description: String,
        input_schema: JsonValue,
    ) -> Self {
        Self {
            runtime,
            name,
            description,
            input_schema,
        }
    }
}

#[async_trait]
impl Tool for WassetteTool {
    async fn metadata(&self) -> Result<ToolMetadata> {
        let parameters = extract_parameters_from_schema(&self.input_schema);
        
        Ok(ToolMetadata {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters,
        })
    }
    
    async fn execute(&self, params: JsonValue) -> Result<ToolResult> {
        match self.runtime.call_tool(&self.name, &params).await {
            Ok(result_str) => {
                let output = serde_json::from_str(&result_str)
                    .unwrap_or_else(|_| JsonValue::String(result_str));
                
                Ok(ToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(e) => {
                Ok(ToolResult {
                    success: false,
                    output: JsonValue::Null,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

/// Extract parameters from a JSON Schema
fn extract_parameters_from_schema(schema: &JsonValue) -> Vec<ToolParameter> {
    let mut params = Vec::new();
    
    if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
        let required: Vec<&str> = schema.get("required")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        
        for (name, prop) in properties {
            let description = prop.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let param_type = prop.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();
            
            params.push(ToolParameter {
                name: name.clone(),
                description,
                required: required.contains(&name.as_str()),
                param_type,
            });
        }
    }
    
    params
}

// =============================================================================
// WassetteAppLoader - High-level loader
// =============================================================================

/// Loader for Wassette-based applications
pub struct WassetteAppLoader {
    runtime: Arc<WassetteRuntime>,
}

impl WassetteAppLoader {
    /// Create a new Wassette app loader
    pub async fn new(component_dir: impl AsRef<Path>) -> Result<Self> {
        Self::new_with_redis(component_dir, None).await
    }
    
    /// Create a new Wassette app loader with Redis support
    pub async fn new_with_redis(component_dir: impl AsRef<Path>, redis_config: Option<RedisConfig>) -> Result<Self> {
        let runtime = WassetteRuntime::new_with_redis(component_dir, redis_config).await?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }
    
    /// Create a new Wassette app loader without eager loading
    pub async fn new_unloaded(component_dir: impl AsRef<Path>) -> Result<Self> {
        Self::new_unloaded_with_redis(component_dir, None).await
    }
    
    /// Create a new Wassette app loader without eager loading, with Redis support
    pub async fn new_unloaded_with_redis(component_dir: impl AsRef<Path>, redis_config: Option<RedisConfig>) -> Result<Self> {
        let runtime = WassetteRuntime::new_unloaded_with_redis(component_dir, redis_config).await?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }
    
    /// Get a reference to the underlying runtime
    pub fn runtime(&self) -> &Arc<WassetteRuntime> {
        &self.runtime
    }
    
    /// Load a component and return a WassetteApp
    pub async fn load_app(&self, uri: &str) -> Result<WassetteApp> {
        let result = self.runtime.load_component(uri).await?;
        
        Ok(WassetteApp::new(
            Arc::clone(&self.runtime),
            result.component_id,
            result.tool_names,
        ))
    }
    
    /// Get all tools as trait objects
    pub async fn get_all_tools(&self) -> Result<Vec<Box<dyn Tool>>> {
        let tools = self.runtime.list_tools().await?;
        
        let mut result: Vec<Box<dyn Tool>> = Vec::new();
        
        for tool in tools {
            let name = tool.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            let description = tool.get("description").and_then(|d| d.as_str()).unwrap_or("").to_string();
            let input_schema = tool.get("inputSchema").cloned().unwrap_or(JsonValue::Object(Default::default()));
            
            if !name.is_empty() {
                result.push(Box::new(WassetteTool::new(
                    Arc::clone(&self.runtime),
                    name,
                    description,
                    input_schema,
                )));
            }
        }
        
        Ok(result)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_parameters_from_schema() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "description": "Amount to increment"
                },
                "name": {
                    "type": "string",
                    "description": "Name parameter"
                }
            },
            "required": ["amount"]
        });
        
        let params = extract_parameters_from_schema(&schema);
        assert_eq!(params.len(), 2);
        
        let amount_param = params.iter().find(|p| p.name == "amount").unwrap();
        assert!(amount_param.required);
        assert_eq!(amount_param.param_type, "integer");
        
        let name_param = params.iter().find(|p| p.name == "name").unwrap();
        assert!(!name_param.required);
        assert_eq!(name_param.param_type, "string");
    }
    
    #[test]
    fn test_normalize_tool_name() {
        assert_eq!(
            WassetteRuntime::normalize_tool_name("local:counter/counter-ops", "get"),
            "local_counter_counter-ops_get"
        );
    }
}
