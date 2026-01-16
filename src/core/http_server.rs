use axum::{
    extract::{State, Request},
    http::{StatusCode, header, HeaderMap},
    response::{IntoResponse, Response, sse::Event, Sse},
    routing::{post, get},
    Json, Router,
    middleware::{self, Next},
};
use futures::stream::{self, Stream};
use std::convert::Infallible;
use serde_json::json;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use chrono::Utc;
use std::time::Instant;

use super::protocol::*;
use super::server::McpServer;
use super::metrics::{MetricsCollector, ApiCallLog};
use super::auth::{AuthService, ChallengeRequest, VerifyRequest};
use crate::core::types::JsonValue;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    mcp_server: Arc<McpServer>,
    metrics: Arc<MetricsCollector>,
    auth: Option<Arc<AuthService>>,
}

/// HTTP Server for MCP
pub struct HttpServer {
    mcp_server: Arc<McpServer>,
    metrics: Arc<MetricsCollector>,
    auth: Option<Arc<AuthService>>,
    host: String,
    port: u16,
}

impl HttpServer {
    pub fn new(mcp_server: Arc<McpServer>, host: String, port: u16) -> Self {
        Self { 
            mcp_server,
            metrics: Arc::new(MetricsCollector::new()),
            auth: None,
            host,
            port 
        }
    }

    pub fn with_metrics(mcp_server: Arc<McpServer>, metrics: Arc<MetricsCollector>, host: String, port: u16) -> Self {
        Self { 
            mcp_server,
            metrics,
            auth: None,
            host,
            port 
        }
    }

    pub fn with_auth(mut self, auth: Arc<AuthService>) -> Self {
        self.auth = Some(auth);
        self
    }

    pub async fn start(self) -> anyhow::Result<()> {
        let state = AppState {
            mcp_server: self.mcp_server,
            metrics: self.metrics,
            auth: self.auth.clone(),
        };

        // Create protected API routes (not the dashboard HTML itself)
        let protected_api_routes = Router::new()
            .route("/api/metrics", get(get_metrics))
            .route("/api/logs", get(get_logs))
            .route("/api/errors", get(get_errors))
            .route("/api/stats", get(get_stats));

        // Apply auth middleware to API routes if auth is enabled
        let protected_api_routes = if let Some(ref auth_service) = self.auth {
            if auth_service.is_enabled() {
                info!("Web3 authentication enabled for dashboard APIs");
                protected_api_routes.layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
            } else {
                info!("Web3 authentication disabled");
                protected_api_routes
            }
        } else {
            info!("Web3 authentication not configured");
            protected_api_routes
        };

        let app = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/sse", get(handle_sse_stream))
            .route("/health", get(health_check))
            // Dashboard HTML (public - auth checked by JavaScript)
            .route("/dashboard", get(serve_dashboard))
            // Login page (public)
            .route("/login", get(serve_login))
            // Auth endpoints (public)
            .route("/api/auth/challenge", post(get_auth_challenge))
            .route("/api/auth/verify", post(verify_auth_signature))
            // Merge protected API routes
            .merge(protected_api_routes)
            .layer(CorsLayer::permissive())
            .with_state(state);

        let addr = format!("{}:{}", self.host, self.port);
        info!("MCP HTTP Server starting on {}", addr);
        info!("Dashboard available at http://{}/dashboard", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "service": "mecp",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// SSE endpoint for MCP streaming
/// This endpoint supports Server-Sent Events for ChatGPT and other MCP clients
async fn handle_sse_stream(
    State(_state): State<AppState>,
    _headers: HeaderMap,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("SSE connection established");
    
    // Create a stream that sends initial connection event and periodic heartbeats
    let stream = stream::unfold(true, |first| async move {
        if first {
            // Send initial connection event
            let event = Event::default()
                .event("connected")
                .data(json!({
                    "status": "connected",
                    "service": "mecp",
                    "version": env!("CARGO_PKG_VERSION"),
                    "protocol": "sse"
                }).to_string());
            Some((
                Ok(event),
                false
            ))
        } else {
            // Send periodic heartbeat to keep connection alive
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            let heartbeat = Event::default()
                .event("heartbeat")
                .data(json!({"timestamp": Utc::now().to_rfc3339()}).to_string());
            Some((
                Ok(heartbeat),
                false
            ))
        }
    });
    
    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
}

async fn handle_mcp_request(
    State(state): State<AppState>,
    Json(request): Json<JsonRpcRequest>,
) -> Response {
    let start_time = Instant::now();
    let method = request.method.clone();
    let request_params = serde_json::to_string(&request.params).ok();
    
    info!("Received MCP request: method={}", method);

    let response = match request.method.as_str() {
        methods::INITIALIZE => handle_initialize(&request).await,
        methods::LIST_RESOURCES => handle_list_resources(&state.mcp_server, &request).await,
        methods::READ_RESOURCE => handle_read_resource(&state.mcp_server, &request).await,
        methods::LIST_TOOLS => handle_list_tools(&state.mcp_server, &request).await,
        methods::CALL_TOOL => handle_call_tool(&state.mcp_server, &request).await,
        methods::LIST_PROMPTS => handle_list_prompts(&state.mcp_server, &request).await,
        methods::GET_PROMPT => handle_get_prompt(&state.mcp_server, &request).await,
        _ => JsonRpcResponse::error(
            request.id.clone(),
            -32601,
            format!("Method not found: {}", request.method),
        ),
    };

    // Record metrics
    let duration_ms = start_time.elapsed().as_millis() as u64;
    let (status, error_msg) = if response.error.is_some() {
        ("error".to_string(), response.error.as_ref().map(|e| e.message.clone()))
    } else {
        ("success".to_string(), None)
    };

    // Capture response data (serialize the response)
    let response_data = serde_json::to_string(&response).ok();

    let log = ApiCallLog {
        id: None,
        method: method.clone(),
        endpoint: "/mcp".to_string(),
        request_params,
        response_data,
        response_status: status,
        error_message: error_msg,
        duration_ms,
        timestamp: Utc::now(),
        client_info: None,
    };

    // Don't block on metrics recording
    let metrics = state.metrics.clone();
    tokio::spawn(async move {
        if let Err(e) = metrics.record_call(log).await {
            error!("Failed to record metrics: {}", e);
        }
    });

    (StatusCode::OK, Json(response)).into_response()
}

async fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    let result = InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            resources: Some(ResourcesCapability {
                subscribe: false,
                list_changed: false,
            }),
            tools: Some(ToolsCapability {
                list_changed: false,
            }),
            prompts: Some(PromptsCapability {
                list_changed: false,
            }),
        },
        server_info: ServerInfo {
            name: "MeCP".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    JsonRpcResponse::success(
        request.id.clone(),
        serde_json::to_value(result).unwrap(),
    )
}

async fn handle_list_resources(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    match server.list_resources().await {
        Ok(resources) => {
            let resource_infos: Vec<ResourceInfo> = resources
                .into_iter()
                .map(|r| ResourceInfo {
                    uri: r.uri.clone(),
                    name: r.name.clone(),
                    description: Some(r.description.clone()),
                    mime_type: r.mime_type.clone(),
                })
                .collect();

            let result = ResourceListResult {
                resources: resource_infos,
            };

            JsonRpcResponse::success(
                request.id.clone(),
                serde_json::to_value(result).unwrap(),
            )
        }
        Err(e) => {
            error!("Failed to list resources: {}", e);
            JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
        }
    }
}

async fn handle_read_resource(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params: Result<ReadResourceParams, _> = serde_json::from_value(
        request.params.clone().unwrap_or(json!({})),
    );

    match params {
        Ok(params) => match server.read_resource(&params.uri).await {
            Ok(content) => {
                let resource_content = ResourceContent {
                    uri: content.uri.clone(),
                    mime_type: Some("application/json".to_string()),
                    text: Some(content.content.to_string()),
                    blob: None,
                };

                let result = ReadResourceResult {
                    contents: vec![resource_content],
                };

                JsonRpcResponse::success(
                    request.id.clone(),
                    serde_json::to_value(result).unwrap(),
                )
            }
            Err(e) => {
                error!("Failed to read resource: {}", e);
                JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
            }
        },
        Err(e) => JsonRpcResponse::error(
            request.id.clone(),
            -32602,
            format!("Invalid params: {}", e),
        ),
    }
}

async fn handle_list_tools(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    match server.list_tools().await {
        Ok(tools) => {
            let tool_infos: Vec<ToolInfo> = tools
                .into_iter()
                .map(|t| ToolInfo {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    input_schema: json!({
                        "type": "object",
                        "properties": t.parameters.iter().map(|p| {
                            (p.name.clone(), json!({
                                "type": p.param_type,
                                "description": p.description
                            }))
                        }).collect::<serde_json::Map<String, JsonValue>>(),
                        "required": t.parameters.iter()
                            .filter(|p| p.required)
                            .map(|p| p.name.clone())
                            .collect::<Vec<_>>()
                    }),
                })
                .collect();

            let result = ToolListResult { tools: tool_infos };

            JsonRpcResponse::success(
                request.id.clone(),
                serde_json::to_value(result).unwrap(),
            )
        }
        Err(e) => {
            error!("Failed to list tools: {}", e);
            JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
        }
    }
}

async fn handle_call_tool(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params: Result<CallToolParams, _> = serde_json::from_value(
        request.params.clone().unwrap_or(json!({})),
    );

    match params {
        Ok(params) => {
            let args = params.arguments.unwrap_or(json!({}));
            match server.call_tool(&params.name, args).await {
                Ok(result) => {
                    let content = ToolContent {
                        content_type: "text".to_string(),
                        text: result.output.to_string(),
                    };

                    let call_result = CallToolResult {
                        content: vec![content],
                        is_error: Some(!result.success),
                    };

                    JsonRpcResponse::success(
                        request.id.clone(),
                        serde_json::to_value(call_result).unwrap(),
                    )
                }
                Err(e) => {
                    error!("Failed to call tool: {}", e);
                    JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
                }
            }
        }
        Err(e) => JsonRpcResponse::error(
            request.id.clone(),
            -32602,
            format!("Invalid params: {}", e),
        ),
    }
}

async fn handle_list_prompts(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    match server.list_prompts().await {
        Ok(prompts) => {
            let prompt_infos: Vec<PromptInfo> = prompts
                .into_iter()
                .map(|p| PromptInfo {
                    name: p.name.clone(),
                    description: Some(p.description.clone()),
                    arguments: Some(
                        p.arguments
                            .into_iter()
                            .map(|a| PromptArgument {
                                name: a.name.clone(),
                                description: Some(a.description.clone()),
                                required: Some(a.required),
                            })
                            .collect(),
                    ),
                })
                .collect();

            let result = PromptListResult {
                prompts: prompt_infos,
            };

            JsonRpcResponse::success(
                request.id.clone(),
                serde_json::to_value(result).unwrap(),
            )
        }
        Err(e) => {
            error!("Failed to list prompts: {}", e);
            JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
        }
    }
}

async fn handle_get_prompt(
    server: &Arc<McpServer>,
    request: &JsonRpcRequest,
) -> JsonRpcResponse {
    let params: Result<GetPromptParams, _> = serde_json::from_value(
        request.params.clone().unwrap_or(json!({})),
    );

    match params {
        Ok(params) => {
            let args = params.arguments.unwrap_or(json!({}));
            match server.get_prompt(&params.name, args).await {
                Ok(result) => {
                    let messages: Vec<PromptMessage> = result
                        .messages
                        .into_iter()
                        .map(|m| PromptMessage {
                            role: m.role.clone(),
                            content: PromptContent {
                                content_type: "text".to_string(),
                                text: m.content.clone(),
                            },
                        })
                        .collect();

                    let prompt_result = GetPromptResult {
                        messages,
                        description: None,
                    };

                    JsonRpcResponse::success(
                        request.id.clone(),
                        serde_json::to_value(prompt_result).unwrap(),
                    )
                }
                Err(e) => {
                    error!("Failed to get prompt: {}", e);
                    JsonRpcResponse::error(request.id.clone(), -32603, e.to_string())
                }
            }
        }
        Err(e) => JsonRpcResponse::error(
            request.id.clone(),
            -32602,
            format!("Invalid params: {}", e),
        ),
    }
}

// Dashboard endpoints

async fn serve_dashboard() -> impl IntoResponse {
    let html = include_str!("../../dashboard/index.html");
    (StatusCode::OK, [("Content-Type", "text/html")], html)
}

async fn serve_login() -> impl IntoResponse {
    let html = include_str!("../../dashboard/login.html");
    (StatusCode::OK, [("Content-Type", "text/html")], html)
}

async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = state.metrics.get_endpoint_metrics().await;
    (StatusCode::OK, Json(json!({
        "metrics": metrics,
        "timestamp": Utc::now()
    })))
}

async fn get_logs(State(state): State<AppState>) -> impl IntoResponse {
    let logs = state.metrics.get_recent_logs(100).await;
    (StatusCode::OK, Json(json!({
        "logs": logs,
        "count": logs.len(),
        "timestamp": Utc::now()
    })))
}

async fn get_errors(State(state): State<AppState>) -> impl IntoResponse {
    // Get error logs - will use MySQL if available, otherwise in-memory
    let errors = state.metrics.get_error_logs(50).await;
    
    (StatusCode::OK, Json(json!({
        "errors": errors,
        "count": errors.len(),
        "timestamp": Utc::now()
    })))
}

async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let logs = state.metrics.get_recent_logs(1000).await;
    let metrics = state.metrics.get_endpoint_metrics().await;
    
    let total_calls: u64 = metrics.iter().map(|m| m.total_calls).sum();
    let total_errors: u64 = metrics.iter().map(|m| m.failed_calls).sum();
    let avg_duration: f64 = if !metrics.is_empty() {
        metrics.iter().map(|m| m.avg_duration_ms).sum::<f64>() / metrics.len() as f64
    } else {
        0.0
    };
    
    (StatusCode::OK, Json(json!({
        "total_calls": total_calls,
        "total_errors": total_errors,
        "success_rate": if total_calls > 0 {
            ((total_calls - total_errors) as f64 / total_calls as f64) * 100.0
        } else {
            0.0
        },
        "avg_duration_ms": avg_duration,
        "endpoints_count": metrics.len(),
        "recent_logs_count": logs.len(),
        "timestamp": Utc::now()
    })))
}

// Auth endpoints

async fn get_auth_challenge(
    State(state): State<AppState>,
    Json(req): Json<ChallengeRequest>,
) -> impl IntoResponse {
    if let Some(ref auth) = state.auth {
        match auth.generate_challenge(&req.address) {
            Ok(challenge) => (StatusCode::OK, Json(challenge)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response(),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Authentication not configured"})),
        ).into_response()
    }
}

async fn verify_auth_signature(
    State(state): State<AppState>,
    Json(req): Json<VerifyRequest>,
) -> impl IntoResponse {
    if let Some(ref auth) = state.auth {
        match auth.verify_signature(&req.address, &req.signature, &req.message) {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ).into_response(),
        }
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"error": "Authentication not configured"})),
        ).into_response()
    }
}

// Auth middleware

async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract Authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth_str) = auth_header {
        // Expected format: "Bearer <token>"
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            if let Some(ref auth_service) = state.auth {
                // Validate token
                match auth_service.validate_token(token) {
                    Ok(_claims) => {
                        // Token valid, proceed
                        return Ok(next.run(request).await);
                    }
                    Err(e) => {
                        error!("Token validation failed: {}", e);
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                }
            }
        }
    }

    // No valid authentication
    Err(StatusCode::UNAUTHORIZED)
}
