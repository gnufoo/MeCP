use mecp::core::metrics::{MetricsCollector, ApiCallLog};
use chrono::Utc;

#[tokio::test]
async fn test_metrics_collector_record_call() {
    let collector = MetricsCollector::new();
    
    let log = ApiCallLog {
        id: None,
        method: "initialize".to_string(),
        endpoint: "/mcp".to_string(),
        request_params: Some(r#"{"protocolVersion":"2024-11-05"}"#.to_string()),
        response_data: Some(r#"{"result":{"protocolVersion":"2024-11-05"}}"#.to_string()),
        response_status: "success".to_string(),
        error_message: None,
        duration_ms: 50,
        timestamp: Utc::now(),
        client_info: None,
    };
    
    // Record the call
    let result = collector.record_call(log.clone()).await;
    assert!(result.is_ok(), "Failed to record call");
    
    // Get recent logs
    let logs = collector.get_recent_logs(10).await;
    assert_eq!(logs.len(), 1, "Should have exactly one log");
    assert_eq!(logs[0].method, "initialize");
    assert_eq!(logs[0].response_status, "success");
}

#[tokio::test]
async fn test_metrics_collector_multiple_calls() {
    let collector = MetricsCollector::new();
    
    // Record multiple calls
    for i in 0..5 {
        let log = ApiCallLog {
            id: None,
            method: format!("method_{}", i),
            endpoint: "/mcp".to_string(),
            request_params: None,
            response_data: Some(r#"{"result":"ok"}"#.to_string()),
            response_status: "success".to_string(),
            error_message: None,
            duration_ms: 50 + i as u64,
            timestamp: Utc::now(),
            client_info: None,
        };
        
        collector.record_call(log).await.unwrap();
    }
    
    // Get recent logs
    let logs = collector.get_recent_logs(10).await;
    assert_eq!(logs.len(), 5, "Should have 5 logs");
}

#[tokio::test]
async fn test_endpoint_metrics_aggregation() {
    let collector = MetricsCollector::new();
    
    // Record calls to the same endpoint
    for i in 0..3 {
            let log = ApiCallLog {
                id: None,
                method: "initialize".to_string(),
                endpoint: "/mcp".to_string(),
                request_params: None,
                response_data: Some(r#"{"result":"data"}"#.to_string()),
                response_status: if i == 2 { "error" } else { "success" }.to_string(),
                error_message: if i == 2 { Some("Test error".to_string()) } else { None },
                duration_ms: 50,
                timestamp: Utc::now(),
                client_info: None,
            };
        
        collector.record_call(log).await.unwrap();
    }
    
    // Get metrics
    let metrics = collector.get_endpoint_metrics().await;
    assert_eq!(metrics.len(), 1, "Should have metrics for one endpoint");
    
    let metric = &metrics[0];
    assert_eq!(metric.method, "initialize");
    assert_eq!(metric.endpoint, "/mcp");
    assert_eq!(metric.total_calls, 3);
    assert_eq!(metric.successful_calls, 2);
    assert_eq!(metric.failed_calls, 1);
}

#[tokio::test]
async fn test_metrics_collector_memory_limit() {
    let collector = MetricsCollector::new();
    
    // Record more than 1000 calls to test memory limit
    for i in 0..1100 {
        let log = ApiCallLog {
            id: None,
            method: format!("method_{}", i),
            endpoint: "/mcp".to_string(),
            request_params: None,
            response_data: Some(r#"{"result":"ok"}"#.to_string()),
            response_status: "success".to_string(),
            error_message: None,
            duration_ms: 50,
            timestamp: Utc::now(),
            client_info: None,
        };
        
        collector.record_call(log).await.unwrap();
    }
    
    // Get recent logs - should be limited to 1000
    let logs = collector.get_recent_logs(1100).await;
    assert!(logs.len() <= 1000, "Should limit logs to 1000 in memory");
}

#[tokio::test]
async fn test_endpoint_metrics_multiple_methods() {
    let collector = MetricsCollector::new();
    
    // Record calls to different methods
    let methods = vec!["initialize", "resources/list", "tools/call", "prompts/get"];
    
    for method in methods.iter() {
        let log = ApiCallLog {
            id: None,
            method: method.to_string(),
            endpoint: "/mcp".to_string(),
            request_params: None,
            response_data: Some(r#"{"result":"ok"}"#.to_string()),
            response_status: "success".to_string(),
            error_message: None,
            duration_ms: 50,
            timestamp: Utc::now(),
            client_info: None,
        };
        
        collector.record_call(log).await.unwrap();
    }
    
    // Get metrics
    let metrics = collector.get_endpoint_metrics().await;
    assert_eq!(metrics.len(), 4, "Should have metrics for 4 different methods");
}

#[tokio::test]
async fn test_api_call_log_serialization() {
    let log = ApiCallLog {
        id: Some(1),
        method: "initialize".to_string(),
        endpoint: "/mcp".to_string(),
        request_params: Some(r#"{"test":"value"}"#.to_string()),
        response_data: Some(r#"{"result":"success"}"#.to_string()),
        response_status: "success".to_string(),
        error_message: None,
        duration_ms: 100,
        timestamp: Utc::now(),
        client_info: Some("test-client".to_string()),
    };
    
    // Test serialization
    let json = serde_json::to_string(&log);
    assert!(json.is_ok(), "Should serialize to JSON");
    
    // Test deserialization
    let deserialized: Result<ApiCallLog, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok(), "Should deserialize from JSON");
    
    let deserialized_log = deserialized.unwrap();
    assert_eq!(deserialized_log.method, "initialize");
    assert_eq!(deserialized_log.duration_ms, 100);
}

#[tokio::test]
async fn test_metrics_with_errors() {
    let collector = MetricsCollector::new();
    
    // Record some successful and some failed calls
    for i in 0..10 {
        let is_error = i % 3 == 0;
        let log = ApiCallLog {
            id: None,
            method: "test_method".to_string(),
            endpoint: "/mcp".to_string(),
            request_params: None,
            response_data: if is_error { None } else { Some(r#"{"result":"ok"}"#.to_string()) },
            response_status: if is_error { "error" } else { "success" }.to_string(),
            error_message: if is_error { Some(format!("Error {}", i)) } else { None },
            duration_ms: 50,
            timestamp: Utc::now(),
            client_info: None,
        };
        
        collector.record_call(log).await.unwrap();
    }
    
    // Get metrics
    let metrics = collector.get_endpoint_metrics().await;
    assert_eq!(metrics.len(), 1);
    
    let metric = &metrics[0];
    assert_eq!(metric.total_calls, 10);
    assert_eq!(metric.failed_calls, 4); // 0, 3, 6, 9 are errors
    assert_eq!(metric.successful_calls, 6);
}
