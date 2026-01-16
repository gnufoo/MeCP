use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use mysql_async::prelude::*;

/// Represents a single API call log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallLog {
    pub id: Option<i64>,
    pub method: String,
    pub endpoint: String,
    pub request_params: Option<String>,
    pub response_data: Option<String>,
    pub response_status: String,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub client_info: Option<String>,
}

/// Represents aggregated metrics for an endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub method: String,
    pub endpoint: String,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub avg_duration_ms: f64,
    pub last_called: Option<DateTime<Utc>>,
}

/// Metrics collector for tracking API calls
pub struct MetricsCollector {
    logs: Arc<RwLock<Vec<ApiCallLog>>>,
    mysql_writer: Option<Arc<MySqlMetricsWriter>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            mysql_writer: None,
        }
    }

    pub fn with_mysql_writer(mysql_writer: Arc<MySqlMetricsWriter>) -> Self {
        Self {
            logs: Arc::new(RwLock::new(Vec::new())),
            mysql_writer: Some(mysql_writer),
        }
    }

    /// Record an API call
    pub async fn record_call(&self, log: ApiCallLog) -> Result<()> {
        // Store in memory
        {
            let mut logs = self.logs.write().await;
            logs.push(log.clone());
            
            // Keep only last 1000 entries in memory
            if logs.len() > 1000 {
                logs.drain(0..100);
            }
        }

        // Write to MySQL if available
        if let Some(writer) = &self.mysql_writer {
            writer.write_log(log).await?;
        }

        Ok(())
    }

    /// Get all logs (from MySQL if available, otherwise from memory)
    pub async fn get_recent_logs(&self, limit: usize) -> Vec<ApiCallLog> {
        // Try to get from MySQL first if available
        if let Some(writer) = &self.mysql_writer {
            if let Ok(logs) = writer.get_logs(limit, 0).await {
                return logs;
            }
        }
        
        // Fallback to in-memory
        let logs = self.logs.read().await;
        logs.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get aggregated metrics per endpoint (from MySQL if available, otherwise from memory)
    pub async fn get_endpoint_metrics(&self) -> Vec<EndpointMetrics> {
        // Try to get from MySQL first if available
        if let Some(writer) = &self.mysql_writer {
            if let Ok(metrics) = writer.get_metrics().await {
                return metrics;
            }
        }
        
        // Fallback to in-memory calculation
        let logs = self.logs.read().await;
        let mut metrics_map: std::collections::HashMap<String, Vec<&ApiCallLog>> = std::collections::HashMap::new();

        // Group logs by endpoint
        for log in logs.iter() {
            let key = format!("{}:{}", log.method, log.endpoint);
            metrics_map.entry(key).or_insert_with(Vec::new).push(log);
        }

        // Calculate metrics for each endpoint
        metrics_map
            .into_iter()
            .map(|(key, endpoint_logs)| {
                let parts: Vec<&str> = key.split(':').collect();
                let method = parts.get(0).unwrap_or(&"").to_string();
                let endpoint = parts.get(1).unwrap_or(&"").to_string();

                let total_calls = endpoint_logs.len() as u64;
                let successful_calls = endpoint_logs
                    .iter()
                    .filter(|log| log.response_status == "success")
                    .count() as u64;
                let failed_calls = total_calls - successful_calls;

                let total_duration: u64 = endpoint_logs.iter().map(|log| log.duration_ms).sum();
                let avg_duration_ms = if total_calls > 0 {
                    total_duration as f64 / total_calls as f64
                } else {
                    0.0
                };

                let last_called = endpoint_logs.iter().map(|log| log.timestamp).max();

                EndpointMetrics {
                    method,
                    endpoint,
                    total_calls,
                    successful_calls,
                    failed_calls,
                    avg_duration_ms,
                    last_called,
                }
            })
            .collect()
    }

    /// Get error logs (from MySQL if available, otherwise from memory)
    pub async fn get_error_logs(&self, limit: usize) -> Vec<ApiCallLog> {
        // Try to get from MySQL first if available
        if let Some(writer) = &self.mysql_writer {
            if let Ok(errors) = writer.get_error_logs(limit).await {
                return errors;
            }
        }
        
        // Fallback to in-memory filtering
        let logs = self.logs.read().await;
        logs.iter()
            .rev()
            .filter(|log| log.response_status == "error")
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// MySQL writer for metrics
pub struct MySqlMetricsWriter {
    connection_string: String,
}

impl MySqlMetricsWriter {
    pub fn new(host: &str, port: u16, database: &str, username: &str, password: &str) -> Self {
        let connection_string = format!(
            "mysql://{}:{}@{}:{}/{}",
            username, password, host, port, database
        );
        Self { connection_string }
    }

    /// Write a log entry to MySQL
    pub async fn write_log(&self, log: ApiCallLog) -> Result<()> {
        // Create MySQL connection
        let mut conn = mysql_async::Conn::new(mysql_async::Opts::from_url(&self.connection_string)?).await?;

        // Insert log entry
        let query = r"
            INSERT INTO history_logs 
            (method, endpoint, request_params, response_data, response_status, error_message, duration_ms, timestamp, client_info)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW(), ?)
        ";

        conn.exec_drop(
            query,
            (
                &log.method,
                &log.endpoint,
                log.request_params.as_ref(),
                log.response_data.as_ref(),
                &log.response_status,
                log.error_message.as_ref(),
                log.duration_ms,
                log.client_info.as_ref(),
            ),
        )
        .await?;

        conn.disconnect().await?;
        Ok(())
    }

    /// Get logs from MySQL
    pub async fn get_logs(&self, limit: usize, offset: usize) -> Result<Vec<ApiCallLog>> {
        let mut conn = mysql_async::Conn::new(mysql_async::Opts::from_url(&self.connection_string)?).await?;

        let query = format!(
            "SELECT id, method, endpoint, request_params, response_data, response_status, error_message, duration_ms, UNIX_TIMESTAMP(timestamp) as ts, client_info 
             FROM history_logs 
             ORDER BY timestamp DESC 
             LIMIT {} OFFSET {}",
            limit, offset
        );

        let logs: Vec<ApiCallLog> = conn
            .query_map(
                query,
                |(id, method, endpoint, request_params, response_data, response_status, error_message, duration_ms, ts, client_info): (i64, String, String, Option<String>, Option<String>, String, Option<String>, u64, i64, Option<String>)| {
                    ApiCallLog {
                        id: Some(id),
                        method,
                        endpoint,
                        request_params,
                        response_data,
                        response_status,
                        error_message,
                        duration_ms,
                        timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()),
                        client_info,
                    }
                },
            )
            .await?;

        conn.disconnect().await?;
        Ok(logs)
    }

    /// Get aggregated metrics from MySQL
    pub async fn get_metrics(&self) -> Result<Vec<EndpointMetrics>> {
        let mut conn = mysql_async::Conn::new(mysql_async::Opts::from_url(&self.connection_string)?).await?;

        let query = r"
            SELECT 
                method,
                endpoint,
                COUNT(*) as total_calls,
                SUM(CASE WHEN response_status = 'success' THEN 1 ELSE 0 END) as successful_calls,
                SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as failed_calls,
                AVG(duration_ms) as avg_duration_ms,
                UNIX_TIMESTAMP(MAX(timestamp)) as last_called
            FROM history_logs
            GROUP BY method, endpoint
            ORDER BY total_calls DESC
        ";

        let metrics: Vec<EndpointMetrics> = conn
            .query_map(
                query,
                |(method, endpoint, total_calls, successful_calls, failed_calls, avg_duration_ms, last_called): (String, String, u64, u64, u64, f64, Option<i64>)| {
                    EndpointMetrics {
                        method,
                        endpoint,
                        total_calls,
                        successful_calls,
                        failed_calls,
                        avg_duration_ms,
                        last_called: last_called.and_then(|ts| DateTime::from_timestamp(ts, 0)),
                    }
                },
            )
            .await?;

        conn.disconnect().await?;
        Ok(metrics)
    }

    /// Get error logs from MySQL
    pub async fn get_error_logs(&self, limit: usize) -> Result<Vec<ApiCallLog>> {
        let mut conn = mysql_async::Conn::new(mysql_async::Opts::from_url(&self.connection_string)?).await?;

        let query = format!(
            "SELECT id, method, endpoint, request_params, response_data, response_status, error_message, duration_ms, UNIX_TIMESTAMP(timestamp) as ts, client_info 
             FROM history_logs 
             WHERE response_status = 'error'
             ORDER BY timestamp DESC 
             LIMIT {}",
            limit
        );

        let logs: Vec<ApiCallLog> = conn
            .query_map(
                query,
                |(id, method, endpoint, request_params, response_data, response_status, error_message, duration_ms, ts, client_info): (i64, String, String, Option<String>, Option<String>, String, Option<String>, u64, i64, Option<String>)| {
                    ApiCallLog {
                        id: Some(id),
                        method,
                        endpoint,
                        request_params,
                        response_data,
                        response_status,
                        error_message,
                        duration_ms,
                        timestamp: DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now()),
                        client_info,
                    }
                },
            )
            .await?;

        conn.disconnect().await?;
        Ok(logs)
    }

    /// Get total count of logs
    pub async fn get_total_count(&self) -> Result<u64> {
        let mut conn = mysql_async::Conn::new(mysql_async::Opts::from_url(&self.connection_string)?).await?;

        let count: Option<u64> = conn
            .query_first("SELECT COUNT(*) FROM history_logs")
            .await?;

        conn.disconnect().await?;
        Ok(count.unwrap_or(0))
    }
}
