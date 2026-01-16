#!/bin/bash

# MeCP Dashboard Flow Test Script
# This script sends mock requests to test the entire flow:
# Client -> MeCP Server -> Database -> Dashboard

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
MCP_HOST="${MCP_HOST:-127.0.0.1}"
MCP_PORT="${MCP_PORT:-3000}"
BASE_URL="http://${MCP_HOST}:${MCP_PORT}"
NUM_REQUESTS="${1:-20}"

echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   MeCP Dashboard Flow Test Script                 ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Target:${NC} ${BASE_URL}"
echo -e "${GREEN}Requests:${NC} ${NUM_REQUESTS}"
echo ""

# Check if server is running
echo -n "Checking if MeCP server is running... "
if curl -s -f "${BASE_URL}/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Server is running${NC}"
else
    echo -e "${RED}✗ Server is not responding${NC}"
    echo "Please start the server first: cargo run --release"
    exit 1
fi

# Check if MySQL is running
echo -n "Checking if MySQL is running... "
if sudo systemctl is-active --quiet mysql; then
    echo -e "${GREEN}✓ MySQL is running${NC}"
else
    echo -e "${RED}✗ MySQL is not running${NC}"
    echo "Please start MySQL: sudo systemctl start mysql"
    exit 1
fi

# Check if history_logs table exists
echo -n "Checking if history_logs table exists... "
if mysql -u mecp_user -pmecp_password mecp_db -e "SELECT 1 FROM history_logs LIMIT 1;" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Table exists${NC}"
else
    echo -e "${RED}✗ Table not found${NC}"
    echo "Please initialize the database: ./scripts/init-mysql-db.sh"
    exit 1
fi

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Starting test requests...${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""

# Counter for tracking
SUCCESS_COUNT=0
ERROR_COUNT=0

# Function to send request
send_request() {
    local method=$1
    local params=$2
    local description=$3
    local expect_error=$4
    
    echo -n "  → ${description}... "
    
    response=$(curl -s -X POST "${BASE_URL}/mcp" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"id\": ${RANDOM},
            \"method\": \"${method}\",
            \"params\": ${params}
        }")
    
    if echo "$response" | grep -q '"error"'; then
        if [ "$expect_error" = "true" ]; then
            echo -e "${YELLOW}Expected Error${NC}"
            ERROR_COUNT=$((ERROR_COUNT + 1))
        else
            echo -e "${RED}Unexpected Error${NC}"
            ERROR_COUNT=$((ERROR_COUNT + 1))
        fi
    else
        echo -e "${GREEN}Success${NC}"
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    fi
    
    # Small delay to avoid overwhelming the server
    sleep 0.1
}

# Test 1: Initialize requests
echo -e "${BLUE}[1/7] Testing initialize endpoint...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    send_request "initialize" \
        "{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test-client-$i\",\"version\":\"1.0.0\"}}" \
        "Initialize request #$i" \
        "false"
done

# Test 2: List resources
echo -e "${BLUE}[2/7] Testing resources/list endpoint...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    send_request "resources/list" \
        "{}" \
        "List resources #$i" \
        "false"
done

# Test 3: Read resource
echo -e "${BLUE}[3/7] Testing resources/read endpoint...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    send_request "resources/read" \
        "{\"uri\":\"mock://example/resource\"}" \
        "Read resource #$i" \
        "false"
done

# Test 4: List tools
echo -e "${BLUE}[4/7] Testing tools/list endpoint...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    send_request "tools/list" \
        "{}" \
        "List tools #$i" \
        "false"
done

# Test 5: Call tool (success)
echo -e "${BLUE}[5/7] Testing tools/call endpoint (success)...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    names=("Alice" "Bob" "Charlie" "Diana" "Eve")
    name=${names[$((RANDOM % ${#names[@]}))]}
    send_request "tools/call" \
        "{\"name\":\"hello_world\",\"arguments\":{\"name\":\"$name\"}}" \
        "Call hello_world tool with name=$name" \
        "false"
done

# Test 6: Call tool (errors)
echo -e "${BLUE}[6/7] Testing tools/call endpoint (errors)...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    fake_tools=("nonexistent_tool" "invalid_tool" "unknown_tool" "missing_tool" "bad_tool")
    tool=${fake_tools[$((RANDOM % ${#fake_tools[@]}))]}
    send_request "tools/call" \
        "{\"name\":\"$tool\",\"arguments\":{}}" \
        "Call nonexistent tool: $tool" \
        "true"
done

# Test 7: Prompts
echo -e "${BLUE}[7/7] Testing prompts endpoints...${NC}"
for i in $(seq 1 $((NUM_REQUESTS / 7))); do
    if [ $((RANDOM % 2)) -eq 0 ]; then
        send_request "prompts/list" \
            "{}" \
            "List prompts #$i" \
            "false"
    else
        topics=("Rust programming" "AI development" "Database design" "Web development" "Testing")
        topic=${topics[$((RANDOM % ${#topics[@]}))]}
        send_request "prompts/get" \
            "{\"name\":\"mock_prompt\",\"arguments\":{\"topic\":\"$topic\"}}" \
            "Get prompt with topic: $topic" \
            "false"
    fi
done

echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Test Results${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}Successful requests:${NC} $SUCCESS_COUNT"
echo -e "${RED}Error requests:${NC} $ERROR_COUNT"
echo -e "${BLUE}Total requests:${NC} $((SUCCESS_COUNT + ERROR_COUNT))"
echo ""

# Wait for metrics to be written
echo "Waiting for metrics to be written to database..."
sleep 2

# Verify database has the logs
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Database Verification${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""

db_count=$(mysql -u mecp_user -pmecp_password mecp_db -s -N -e "SELECT COUNT(*) FROM history_logs;")
echo -e "${GREEN}Total logs in database:${NC} $db_count"

echo ""
echo "Recent logs:"
mysql -u mecp_user -pmecp_password mecp_db -e "
    SELECT 
        DATE_FORMAT(timestamp, '%H:%i:%S') as time,
        method,
        response_status,
        duration_ms,
        COALESCE(error_message, '-') as error
    FROM history_logs 
    ORDER BY timestamp DESC 
    LIMIT 10;
"

echo ""
echo "Metrics by endpoint:"
mysql -u mecp_user -pmecp_password mecp_db -e "
    SELECT 
        method,
        COUNT(*) as total,
        SUM(CASE WHEN response_status = 'success' THEN 1 ELSE 0 END) as success,
        SUM(CASE WHEN response_status = 'error' THEN 1 ELSE 0 END) as errors,
        ROUND(AVG(duration_ms), 2) as avg_ms
    FROM history_logs 
    GROUP BY method
    ORDER BY total DESC;
"

# Test API endpoints
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}API Endpoints Verification${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""

echo -n "Testing /api/stats... "
if curl -s "${BASE_URL}/api/stats" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Working${NC}"
    curl -s "${BASE_URL}/api/stats" | jq '{total_calls, total_errors, success_rate, avg_duration_ms}'
else
    echo -e "${RED}✗ Failed${NC}"
fi

echo ""
echo -n "Testing /api/metrics... "
if curl -s "${BASE_URL}/api/metrics" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Working${NC}"
    echo "  Endpoints tracked: $(curl -s "${BASE_URL}/api/metrics" | jq '.metrics | length')"
else
    echo -e "${RED}✗ Failed${NC}"
fi

echo ""
echo -n "Testing /api/logs... "
if curl -s "${BASE_URL}/api/logs" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Working${NC}"
    echo "  Recent logs count: $(curl -s "${BASE_URL}/api/logs" | jq '.count')"
else
    echo -e "${RED}✗ Failed${NC}"
fi

echo ""
echo -n "Testing /api/errors... "
if curl -s "${BASE_URL}/api/errors" | jq . > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Working${NC}"
    echo "  Error logs count: $(curl -s "${BASE_URL}/api/errors" | jq '.count')"
else
    echo -e "${RED}✗ Failed${NC}"
fi

# Summary
echo ""
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Summary${NC}"
echo -e "${BLUE}════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${GREEN}✓ Flow Test Complete!${NC}"
echo ""
echo "Next steps:"
echo "  1. Open dashboard: ${BASE_URL}/dashboard"
echo "  2. Verify metrics are displayed correctly"
echo "  3. Check that errors are shown in the errors section"
echo "  4. Confirm auto-refresh is working"
echo ""
echo -e "${YELLOW}Dashboard URL: ${BASE_URL}/dashboard${NC}"
echo ""
