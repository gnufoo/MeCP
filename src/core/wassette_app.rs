//! Wassette Application Integration
//!
//! This module provides an Application trait implementation for Wassette components,
//! enabling them to be used as MeCP applications with full tool and resource support.
//!
//! # Resources
//! 
//! Wassette applications can expose resources that represent their persistent state.
//! For example, a mailbox application exposes:
//! - `mailbox://{username}/inbox` - User's inbox summary
//! 
//! These resources can be read to get the current state and are updated when
//! the application state changes (e.g., receiving a new message).

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::core::application::Application;
use crate::core::message_broker::InterAppMessage;
use crate::core::notifications::{McpNotification, NotificationBroadcaster};
use crate::core::types::{ToolResult as CoreToolResult, ResourceContent as CoreResourceContent, ResourceMetadata, ToolParameter};
use crate::core::wassette_runtime::WassetteRuntime;
use crate::tools::{Tool, ToolMetadata};
use crate::resources::Resource;

// =============================================================================
// WassetteApplication - Application trait implementation
// =============================================================================

/// A Wassette component wrapped as an MeCP Application
pub struct WassetteApplication {
    /// The underlying Wassette runtime
    runtime: Arc<WassetteRuntime>,
    /// Component ID
    component_id: String,
    /// Application manifest info
    app_id: String,
    name: String,
    description: String,
    /// Cached tools from the component
    tools: Vec<WassetteTool>,
    /// Resources exposed by this application
    resources: Vec<WassetteResource>,
    /// Notification broadcaster for sending resource updates
    notifications: Option<Arc<NotificationBroadcaster>>,
    /// User context
    user_id: u64,
    username: String,
}

impl WassetteApplication {
    /// Create a new WassetteApplication from a loaded component
    pub async fn new(
        runtime: Arc<WassetteRuntime>,
        component_id: String,
        app_id: String,
        name: String,
        description: String,
        user_id: u64,
        username: String,
    ) -> Result<Self> {
        Self::new_with_notifications(runtime, component_id, app_id, name, description, user_id, username, None).await
    }
    
    /// Create a new WassetteApplication with notification support
    pub async fn new_with_notifications(
        runtime: Arc<WassetteRuntime>,
        component_id: String,
        app_id: String,
        name: String,
        description: String,
        user_id: u64,
        username: String,
        notifications: Option<Arc<NotificationBroadcaster>>,
    ) -> Result<Self> {
        // Get tools from the component
        let tool_schemas = runtime.list_tools().await?;
        
        // Filter out Wassette built-in management tools
        let wassette_builtin_tools = vec![
            "load-component",
            "unload-component",
            "list-components",
            "search-components",
            "grant-network-permission",
            "revoke-network-permission",
            "grant-storage-permission",
            "revoke-storage-permission",
            "grant-environment-variable-permission",
            "revoke-environment-variable-permission",
            "reset-permission",
            "get-policy",
        ];
        
        let tools: Vec<WassetteTool> = tool_schemas
            .into_iter()
            .filter_map(|schema| {
                let tool_name = schema.get("name")?.as_str()?.to_string();
                
                // Skip Wassette built-in management tools
                if wassette_builtin_tools.contains(&tool_name.as_str()) {
                    return None;
                }
                
                let tool_description = schema.get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("No description")
                    .to_string();
                let input_schema = schema.get("inputSchema").cloned()
                    .unwrap_or(JsonValue::Object(Default::default()));
                
                Some(WassetteTool {
                    runtime: Arc::clone(&runtime),
                    app_id: app_id.clone(),
                    component_id: component_id.clone(),
                    username: username.clone(),
                    user_id,
                    name: tool_name,
                    description: tool_description,
                    input_schema,
                    notifications: notifications.clone(),
                })
            })
            .collect();
        
        // Create resources for this application
        // For mailbox-type apps, expose inbox resource
        let resources = Self::create_resources(&app_id, &username, &runtime);
        
        info!(
            "📦 Created WassetteApplication '{}' with {} tools and {} resources for user '{}' (notifications: {})",
            app_id,
            tools.len(),
            resources.len(),
            username,
            if notifications.is_some() { "enabled" } else { "disabled" }
        );
        
        Ok(Self {
            runtime,
            component_id,
            app_id,
            name,
            description,
            tools,
            resources,
            notifications,
            user_id,
            username,
        })
    }
    
    /// Create resources for the application based on app type
    fn create_resources(app_id: &str, username: &str, runtime: &Arc<WassetteRuntime>) -> Vec<WassetteResource> {
        let mut resources = Vec::new();
        
        // For mailbox applications, expose inbox resource
        if app_id.contains("mailbox") || app_id.contains("mail") {
            resources.push(WassetteResource {
                runtime: Arc::clone(runtime),
                app_id: app_id.to_string(),
                username: username.to_string(),
                resource_type: WassetteResourceType::Inbox,
            });
            
            info!("📦 Created mailbox resource: mailbox://{}/inbox", username);
        }
        
        // Generic app state resource for all apps
        resources.push(WassetteResource {
            runtime: Arc::clone(runtime),
            app_id: app_id.to_string(),
            username: username.to_string(),
            resource_type: WassetteResourceType::AppState,
        });
        
        resources
    }
}

#[async_trait]
impl Application for WassetteApplication {
    fn app_id(&self) -> &str {
        &self.app_id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    async fn initialize(&self, user_id: u64) -> Result<()> {
        debug!("Initializing WassetteApplication '{}' for user {}", self.app_id, user_id);
        Ok(())
    }
    
    async fn get_tools(&self) -> Vec<Box<dyn Tool>> {
        self.tools.iter()
            .map(|t| Box::new(t.clone()) as Box<dyn Tool>)
            .collect()
    }
    
    async fn get_resources(&self) -> Vec<Box<dyn Resource>> {
        self.resources.iter()
            .map(|r| Box::new(r.clone()) as Box<dyn Resource>)
            .collect()
    }
    
    /// Handle an incoming inter-app message
    /// 
    /// This method is called when a message arrives for this application instance.
    /// It will attempt to call the appropriate tool (e.g., `receive-mail` for mailbox apps).
    pub async fn on_message(&self, message: &InterAppMessage) -> Result<bool> {
        info!(
            "📬 MESSAGE DELIVERY: app='{}', user='{}', type='{}', from='{}:{}'",
            self.app_id, self.username, message.message_type, message.from_app, message.from_user
        );
        
        // For mailbox apps, call the receive-mail tool
        if self.app_id.contains("mailbox") || self.app_id.contains("mail") {
            // Extract message data from payload
            let payload = &message.payload;
            let sender = payload.get("sender")
                .and_then(|v| v.as_str())
                .unwrap_or(&message.from_user)
                .to_string();
            let recipient = payload.get("recipient")
                .and_then(|v| v.as_str())
                .unwrap_or(&self.username)
                .to_string();
            let subject = payload.get("subject").and_then(|v| v.as_str()).map(|s| s.to_string());
            let content = payload.get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let timestamp = message.created_at.to_rfc3339();
            
            // Call the receive-mail tool
            let params = serde_json::json!({
                "sender": sender,
                "recipient": recipient,
                "subject": subject,
                "content": content,
                "timestamp": timestamp,
            });
            
            match self.runtime.call_tool_with_user("receive-mail", &params, Some(&self.component_id), Some(self.user_id)).await {
                Ok(result_str) => {
                    // Parse result to check if it was successful
                    let result: JsonValue = serde_json::from_str(&result_str)
                        .unwrap_or_else(|_| JsonValue::String(result_str));
                    
                    // Check if the result indicates success
                    let success = result.get("result")
                        .and_then(|r| r.as_bool())
                        .unwrap_or(false);
                    
                    if success {
                        info!("✅ Message delivered successfully to wassette component");
                    } else {
                        info!("⚠️ Message delivery returned false");
                    }
                    
                    Ok(success)
                }
                Err(e) => {
                    warn!("❌ Failed to deliver message to wassette component: {}", e);
                    Ok(false)
                }
            }
        } else {
            // For other app types, log but don't handle
            debug!("Message type '{}' not handled by app '{}'", message.message_type, self.app_id);
            Ok(false)
        }
    }
}

// =============================================================================
// WassetteTool - Tool implementation for Wassette tools
// =============================================================================

/// A tool exported by a Wassette component
#[derive(Clone)]
pub struct WassetteTool {
    runtime: Arc<WassetteRuntime>,
    app_id: String,
    component_id: String,
    username: String,
    user_id: u64,
    name: String,
    description: String,
    input_schema: JsonValue,
    notifications: Option<Arc<NotificationBroadcaster>>,
}

impl WassetteTool {
    /// Check if this tool triggers a resource update when executed
    fn triggers_resource_update(&self) -> bool {
        // These mailbox tools modify state that should trigger a resource update
        matches!(self.name.as_str(), 
            "send-message" | "delete-message" | "mark-as-read" | 
            "clear-inbox" | "receive-message")
    }
    
    /// Get the resource URI that would be updated by this tool
    fn get_updated_resource_uri(&self, params: &JsonValue) -> Option<String> {
        match self.name.as_str() {
            "send-message" => {
                // When sending a message, the recipient's inbox is updated
                if let Some(recipient) = params.get("recipient").and_then(|v| v.as_str()) {
                    Some(format!("mailbox://{}/inbox", recipient))
                } else {
                    None
                }
            }
            "delete-message" | "mark-as-read" | "clear-inbox" | "receive-message" => {
                // These modify the current user's inbox
                Some(format!("mailbox://{}/inbox", self.username))
            }
            _ => None
        }
    }
    
    /// Send resource update notification
    async fn notify_resource_update(&self, uri: &str, recipient_username: Option<&str>) {
        if let Some(ref broadcaster) = self.notifications {
            let notification = McpNotification::ResourceUpdated { uri: uri.to_string() };
            
            // Notify the recipient if different from current user
            if let Some(recipient) = recipient_username {
                if recipient != self.username {
                    info!(
                        "📢 RESOURCE UPDATE: Notifying '{}' about resource change: {}",
                        recipient, uri
                    );
                    broadcaster.broadcast_to_user(recipient, notification.clone()).await;
                }
            }
            
            // Also notify current user if their resource changed
            if uri.contains(&self.username) {
                info!(
                    "📢 RESOURCE UPDATE: Notifying '{}' about resource change: {}",
                    self.username, uri
                );
                broadcaster.broadcast_to_user(&self.username, notification).await;
            }
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
    
    async fn execute(&self, params: JsonValue) -> Result<CoreToolResult> {
        info!(
            "🔧 TOOL EXECUTE: app='{}', user='{}', tool='{}', params={}",
            self.app_id, self.username, self.name, 
            serde_json::to_string(&params).unwrap_or_default()
        );
        
        let should_notify = self.triggers_resource_update();
        let resource_uri = self.get_updated_resource_uri(&params);
        let recipient = params.get("recipient").and_then(|v| v.as_str()).map(|s| s.to_string());
        
        // Call tool with user context to enable KV storage
        match self.runtime.call_tool_with_user(&self.name, &params, Some(&self.component_id), Some(self.user_id)).await {
            Ok(result_str) => {
                // Parse the result - Wassette returns JSON
                let output: JsonValue = serde_json::from_str(&result_str)
                    .unwrap_or_else(|_| JsonValue::String(result_str.clone()));
                
                info!(
                    "✅ TOOL SUCCESS: app='{}', tool='{}', result={}",
                    self.app_id, self.name, result_str
                );
                
                // Send resource update notification if applicable
                if should_notify {
                    if let Some(ref uri) = resource_uri {
                        self.notify_resource_update(uri, recipient.as_deref()).await;
                    }
                }
                
                Ok(CoreToolResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(e) => {
                info!(
                    "❌ TOOL ERROR: app='{}', tool='{}', error={}",
                    self.app_id, self.name, e
                );
                
                Ok(CoreToolResult {
                    success: false,
                    output: JsonValue::Null,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Extract parameters from JSON Schema
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
// WassetteResource - Resource implementation for Wassette apps
// =============================================================================

/// Type of Wassette resource
#[derive(Clone, Debug)]
pub enum WassetteResourceType {
    /// User's inbox (for mailbox apps)
    Inbox,
    /// Generic app state
    AppState,
}

/// A resource exposed by a Wassette application
#[derive(Clone)]
pub struct WassetteResource {
    runtime: Arc<WassetteRuntime>,
    app_id: String,
    username: String,
    resource_type: WassetteResourceType,
}

impl WassetteResource {
    /// Get the resource URI for this resource
    fn get_uri(&self) -> String {
        match self.resource_type {
            WassetteResourceType::Inbox => format!("mailbox://{}/inbox", self.username),
            WassetteResourceType::AppState => format!("{}://{}/state", self.app_id, self.username),
        }
    }
    
    /// Get the resource name
    fn get_name(&self) -> String {
        match self.resource_type {
            WassetteResourceType::Inbox => format!("{}'s Inbox", self.username),
            WassetteResourceType::AppState => format!("{} - {} State", self.app_id, self.username),
        }
    }
    
    /// Get the resource description
    fn get_description(&self) -> String {
        match self.resource_type {
            WassetteResourceType::Inbox => format!("Messages inbox for user '{}'", self.username),
            WassetteResourceType::AppState => format!("Application state for '{}' owned by '{}'", self.app_id, self.username),
        }
    }
}

#[async_trait]
impl Resource for WassetteResource {
    async fn metadata(&self) -> Result<ResourceMetadata> {
        Ok(ResourceMetadata {
            uri: self.get_uri(),
            name: self.get_name(),
            description: self.get_description(),
            mime_type: Some("application/json".to_string()),
        })
    }
    
    async fn read(&self) -> Result<CoreResourceContent> {
        let uri = self.get_uri();
        
        // Read resource content based on type
        let content = match self.resource_type {
            WassetteResourceType::Inbox => {
                // Query inbox by calling the get-inbox-count or query-messages tool
                match self.runtime.call_tool("get-inbox-count", &serde_json::json!({})).await {
                    Ok(result) => {
                        // Parse result and format as inbox summary
                        let count: JsonValue = serde_json::from_str(&result)
                            .unwrap_or(JsonValue::Null);
                        
                        serde_json::json!({
                            "type": "inbox",
                            "user": self.username,
                            "message_count": count,
                            "uri": uri
                        })
                    }
                    Err(e) => {
                        info!("📭 Inbox resource read error (may be empty): {}", e);
                        serde_json::json!({
                            "type": "inbox",
                            "user": self.username,
                            "message_count": 0,
                            "error": e.to_string(),
                            "uri": uri
                        })
                    }
                }
            }
            WassetteResourceType::AppState => {
                // Generic app state - for now return basic info
                serde_json::json!({
                    "type": "app_state",
                    "app_id": self.app_id,
                    "user": self.username,
                    "uri": uri,
                    "status": "active"
                })
            }
        };
        
        info!("📖 Resource read: {} -> {} bytes", uri, content.to_string().len());
        
        // Build metadata
        let mut metadata = HashMap::new();
        metadata.insert("mime_type".to_string(), "application/json".to_string());
        metadata.insert("name".to_string(), self.get_name());
        
        Ok(CoreResourceContent {
            uri,
            content,
            metadata: Some(metadata),
        })
    }
    
    async fn uri(&self) -> String {
        self.get_uri()
    }
}
