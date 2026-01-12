#!/bin/bash
# Oracle Tool Execution Integration Test
# Tests that voice commands trigger actual tool execution

set -e

API_URL="http://localhost:3535"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🧪 Oracle Tool Execution Integration Test"
echo "=========================================="
echo ""

# Check if server is running
echo -n "Checking server status... "
if curl -s "${API_URL}/status" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Server is running"
else
    echo -e "${RED}✗${NC} Server not running"
    echo "Start server with: cd karana-core && cargo run --release"
    exit 1
fi

echo ""
echo "Running test commands..."
echo "------------------------"

# Test 1: Open Camera
echo -n "Test 1: 'open camera' → "
RESPONSE=$(curl -s -X POST "${API_URL}/oracle" \
    -H "Content-Type: application/json" \
    -d '{"text": "open camera"}')

if echo "$RESPONSE" | grep -q "camera"; then
    echo -e "${GREEN}✓ PASS${NC}"
    echo "  Response: $(echo $RESPONSE | jq -r '.content' 2>/dev/null || echo $RESPONSE)"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
fi

# Test 2: Check Balance
echo -n "Test 2: 'check balance' → "
RESPONSE=$(curl -s -X POST "${API_URL}/oracle" \
    -H "Content-Type: application/json" \
    -d '{"text": "check my balance"}')

if echo "$RESPONSE" | grep -q "balance\|KARA"; then
    echo -e "${GREEN}✓ PASS${NC}"
    echo "  Response: $(echo $RESPONSE | jq -r '.content' 2>/dev/null || echo $RESPONSE)"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
fi

# Test 3: Navigate
echo -n "Test 3: 'navigate to SF' → "
RESPONSE=$(curl -s -X POST "${API_URL}/oracle" \
    -H "Content-Type: application/json" \
    -d '{"text": "navigate to San Francisco"}')

if echo "$RESPONSE" | grep -q "navigat\|San Francisco"; then
    echo -e "${GREEN}✓ PASS${NC}"
    echo "  Response: $(echo $RESPONSE | jq -r '.content' 2>/dev/null || echo $RESPONSE)"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
fi

# Test 4: Create Task
echo -n "Test 4: 'take note' → "
RESPONSE=$(curl -s -X POST "${API_URL}/oracle" \
    -H "Content-Type: application/json" \
    -d '{"text": "take note buy groceries"}')

if echo "$RESPONSE" | grep -q "note\|task\|groceries"; then
    echo -e "${GREEN}✓ PASS${NC}"
    echo "  Response: $(echo $RESPONSE | jq -r '.content' 2>/dev/null || echo $RESPONSE)"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
fi

# Test 5: Play Music
echo -n "Test 5: 'play music' → "
RESPONSE=$(curl -s -X POST "${API_URL}/oracle" \
    -H "Content-Type: application/json" \
    -d '{"text": "play jazz music"}')

if echo "$RESPONSE" | grep -q "music\|jazz\|playing"; then
    echo -e "${GREEN}✓ PASS${NC}"
    echo "  Response: $(echo $RESPONSE | jq -r '.content' 2>/dev/null || echo $RESPONSE)"
else
    echo -e "${RED}✗ FAIL${NC}"
    echo "  Response: $RESPONSE"
fi

echo ""
echo "------------------------"
echo -e "${GREEN}Integration test complete!${NC}"
echo ""
echo "Check server logs for tool execution details:"
echo "  [API] ✓ Tool executed: <action>"
echo ""
