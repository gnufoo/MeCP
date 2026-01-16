-- Migration script to add response_data column to existing history_logs table
-- Run this if you have an existing database

-- Check if column already exists before adding
SET @col_exists = (
    SELECT COUNT(*)
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
    AND TABLE_NAME = 'history_logs'
    AND COLUMN_NAME = 'response_data'
);

-- Add column if it doesn't exist
SET @query = IF(@col_exists = 0,
    'ALTER TABLE history_logs ADD COLUMN response_data TEXT AFTER request_params',
    'SELECT "Column response_data already exists" AS message'
);

PREPARE stmt FROM @query;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

SELECT 'Migration complete: response_data column added' AS status;
