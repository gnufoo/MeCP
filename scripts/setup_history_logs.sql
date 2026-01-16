-- MeCP History Logs Table
-- This table stores all API call history for monitoring and debugging

CREATE TABLE IF NOT EXISTS history_logs (
    id BIGINT AUTO_INCREMENT PRIMARY KEY,
    method VARCHAR(50) NOT NULL,
    endpoint VARCHAR(255) NOT NULL,
    request_params TEXT,
    response_data TEXT,
    response_status VARCHAR(20) NOT NULL,
    error_message TEXT,
    duration_ms BIGINT UNSIGNED NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    client_info VARCHAR(255),
    
    INDEX idx_method (method),
    INDEX idx_endpoint (endpoint),
    INDEX idx_response_status (response_status),
    INDEX idx_timestamp (timestamp),
    INDEX idx_method_endpoint (method, endpoint)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Create a view for quick statistics
CREATE OR REPLACE VIEW endpoint_statistics AS
SELECT 
    method,
    endpoint,
    COUNT(*) as total_calls,
    SUM(CASE WHEN response_status = 'success' THEN 1 ELSE 0 END) as successful_calls,
    SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as failed_calls,
    AVG(duration_ms) as avg_duration_ms,
    MIN(duration_ms) as min_duration_ms,
    MAX(duration_ms) as max_duration_ms,
    MAX(timestamp) as last_called
FROM history_logs
GROUP BY method, endpoint;
