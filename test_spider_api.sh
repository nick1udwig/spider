#!/bin/bash

# Spider API Test Script
# Tests all Spider API endpoints to verify they work correctly

# Configuration
BASE_URL="http://localhost:8080"
PROVIDER="anthropic"
TEST_API_KEY="test-key-123"
SPIDER_KEY_NAME="Test Key"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print test results
print_result() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
    fi
}

# Function to test an endpoint
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo -e "\n${YELLOW}Testing:${NC} $description"
    echo "Endpoint: $method $endpoint"
    
    if [ "$method" = "POST" ]; then
        if [ -n "$data" ]; then
            response=$(curl -s -X POST \
                -H "Content-Type: application/json" \
                -d "$data" \
                "$BASE_URL/api" \
                -w "\nHTTP_STATUS:%{http_code}")
        else
            response=$(curl -s -X POST \
                -H "Content-Type: application/json" \
                "$BASE_URL/api" \
                -w "\nHTTP_STATUS:%{http_code}")
        fi
    else
        response=$(curl -s -X GET \
            "$BASE_URL/api" \
            -w "\nHTTP_STATUS:%{http_code}")
    fi
    
    http_status=$(echo "$response" | grep "HTTP_STATUS" | cut -d: -f2)
    body=$(echo "$response" | sed '$d')
    
    echo "Response: $body"
    
    if [ "$http_status" = "200" ]; then
        print_result 0 "HTTP Status: $http_status"
        
        # Check if response has Ok field (successful Result type)
        if echo "$body" | grep -q '"Ok"'; then
            print_result 0 "Response has Ok field"
        # Or check if it's an array (for list endpoints)
        elif echo "$body" | grep -q '^\[' || echo "$body" | grep -q '"Ok":\['; then
            print_result 0 "Response is properly formatted"
        else
            print_result 1 "Response format unexpected"
        fi
    else
        print_result 1 "HTTP Status: $http_status"
    fi
    
    return $?
}

echo "========================================="
echo "      Spider API Test Suite"
echo "========================================="
echo "Testing against: $BASE_URL"
echo ""

# Test 1: List API Keys (should be empty initially)
test_endpoint "POST" "listApiKeys" '{"ListApiKeys":null}' "List API Keys (empty)"

# Test 2: Set an API Key
test_endpoint "POST" "setApiKey" "{\"SetApiKey\":{\"provider\":\"$PROVIDER\",\"key\":\"$TEST_API_KEY\"}}" "Set API Key for $PROVIDER"

# Test 3: List API Keys (should have one now)
test_endpoint "POST" "listApiKeys" '{"ListApiKeys":null}' "List API Keys (with one key)"

# Test 4: List Spider Keys (should be empty initially)
test_endpoint "POST" "listSpiderKeys" '{"ListSpiderKeys":null}' "List Spider Keys (empty)"

# Test 5: Create a Spider Key
test_endpoint "POST" "createSpiderKey" "{\"CreateSpiderKey\":{\"name\":\"$SPIDER_KEY_NAME\",\"permissions\":[\"chat\",\"read\",\"write\"]}}" "Create Spider Key"

# Store the spider key for later use
SPIDER_KEY=$(echo "$body" | grep -o '"key":"[^"]*"' | cut -d'"' -f4)
echo "Created Spider Key: $SPIDER_KEY"

# Test 6: List Spider Keys (should have one now)
test_endpoint "POST" "listSpiderKeys" '{"ListSpiderKeys":null}' "List Spider Keys (with one key)"

# Test 7: Get Config
test_endpoint "POST" "getConfig" '{"GetConfig":null}' "Get Configuration"

# Test 8: Update Config
test_endpoint "POST" "updateConfig" '{"UpdateConfig":{"defaultLlmProvider":"anthropic","maxTokens":2048,"temperature":0.5}}' "Update Configuration"

# Test 9: List MCP Servers (should be empty)
test_endpoint "POST" "listMcpServers" '{"ListMcpServers":null}' "List MCP Servers"

# Test 10: Add MCP Server
test_endpoint "POST" "addMcpServer" '{"AddMcpServer":{"name":"Test Server","transport":{"transportType":"http","command":null,"args":null,"url":"http://localhost:3000"}}}' "Add MCP Server"

# Test 11: List Conversations (should be empty)
test_endpoint "POST" "listConversations" '{"ListConversations":{"limit":10,"offset":0,"client":null}}' "List Conversations"

# Test 12: Test Chat endpoint (requires valid Spider key)
if [ -n "$SPIDER_KEY" ]; then
    test_endpoint "POST" "chat" "{\"Chat\":{\"apiKey\":\"$SPIDER_KEY\",\"messages\":[{\"role\":\"user\",\"content\":\"Hello, this is a test message\",\"toolCallsJson\":null,\"toolResultsJson\":null,\"timestamp\":$(date +%s)}],\"llmProvider\":\"anthropic\",\"mcpServers\":null,\"metadata\":{\"startTime\":\"$(date -Iseconds)\",\"client\":\"test-script\",\"fromStt\":false}}}" "Send Chat Message"
else
    echo -e "${YELLOW}Skipping chat test - no Spider key available${NC}"
fi

# Test 13: Remove API Key
test_endpoint "POST" "removeApiKey" "{\"RemoveApiKey\":\"$PROVIDER\"}" "Remove API Key"

# Test 14: Revoke Spider Key
if [ -n "$SPIDER_KEY" ]; then
    test_endpoint "POST" "revokeSpiderKey" "{\"RevokeSpiderKey\":\"$SPIDER_KEY\"}" "Revoke Spider Key"
fi

echo ""
echo "========================================="
echo "      Test Suite Complete"
echo "========================================="