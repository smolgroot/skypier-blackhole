#!/bin/bash

# Test signal handling for Skypier Blackhole

# Change to project root directory
cd "$(dirname "$0")/.." || exit 1

set -e

CONFIG_FILE="test-config.toml"
BINARY="./target/release/skypier-blackhole"
PID_FILE="/tmp/skypier-blackhole-test.pid"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Skypier Blackhole Signal Handling Tests ===${NC}"
echo ""

# Cleanup function
cleanup() {
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE")
        if kill -0 "$PID" 2>/dev/null; then
            echo -e "${YELLOW}Cleaning up server (PID: $PID)...${NC}"
            kill -TERM "$PID" 2>/dev/null || true
            sleep 2
            kill -9 "$PID" 2>/dev/null || true
        fi
        rm -f "$PID_FILE"
    fi
}

trap cleanup EXIT

# Test 1: Start server and verify it's running
echo -e "${GREEN}Test 1: Starting server...${NC}"
$BINARY --config "$CONFIG_FILE" start &
SERVER_PID=$!
echo $SERVER_PID > "$PID_FILE"
echo "Server PID: $SERVER_PID"

# Wait for server to start
sleep 2

# Check if server is running
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo -e "${RED}FAIL: Server failed to start${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server started successfully${NC}"
echo ""

# Test 2: Verify DNS is responding
echo -e "${GREEN}Test 2: Testing DNS resolution...${NC}"
RESPONSE=$(dig @127.0.0.1 -p 15353 google.com +short +time=2 +tries=1 || true)
if [ -z "$RESPONSE" ]; then
    echo -e "${RED}FAIL: DNS not responding${NC}"
    exit 1
fi
echo -e "${GREEN}✓ DNS is responding: $RESPONSE${NC}"
echo ""

# Test 3: Verify blocklist is working
echo -e "${GREEN}Test 3: Testing blocklist (should block ads.example.com)...${NC}"
BLOCKED=$(dig @127.0.0.1 -p 15353 ads.example.com +short +time=2 +tries=1 || true)
if [ -n "$BLOCKED" ]; then
    echo -e "${RED}FAIL: Domain should be blocked but got: $BLOCKED${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Blocklist is working${NC}"
echo ""

# Test 4: SIGHUP reload
echo -e "${GREEN}Test 4: Testing SIGHUP reload...${NC}"
echo "# Adding new domain" >> /tmp/skypier-test-blocklist.txt
echo "tracker.example.com" >> /tmp/skypier-test-blocklist.txt
echo "Sending SIGHUP to PID $SERVER_PID..."
kill -HUP "$SERVER_PID"
sleep 2

# Verify server is still running
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo -e "${RED}FAIL: Server crashed after SIGHUP${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server survived SIGHUP${NC}"

# Verify new domain is blocked
BLOCKED=$(dig @127.0.0.1 -p 15353 tracker.example.com +short +time=2 +tries=1 || true)
if [ -n "$BLOCKED" ]; then
    echo -e "${RED}FAIL: New domain should be blocked after reload but got: $BLOCKED${NC}"
    exit 1
fi
echo -e "${GREEN}✓ New domain is blocked after SIGHUP reload${NC}"
echo ""

# Test 5: Multiple SIGHUPs
echo -e "${GREEN}Test 5: Testing multiple SIGHUP signals...${NC}"
for i in {1..5}; do
    echo "SIGHUP #$i..."
    kill -HUP "$SERVER_PID"
    sleep 1
    if ! kill -0 "$SERVER_PID" 2>/dev/null; then
        echo -e "${RED}FAIL: Server crashed on SIGHUP #$i${NC}"
        exit 1
    fi
done
echo -e "${GREEN}✓ Server handled multiple SIGHUPs${NC}"
echo ""

# Test 6: DNS still working after reloads
echo -e "${GREEN}Test 6: Verifying DNS still works after multiple reloads...${NC}"
RESPONSE=$(dig @127.0.0.1 -p 15353 google.com +short +time=2 +tries=1 || true)
if [ -z "$RESPONSE" ]; then
    echo -e "${RED}FAIL: DNS not responding after reloads${NC}"
    exit 1
fi
echo -e "${GREEN}✓ DNS still working: $RESPONSE${NC}"
echo ""

# Test 7: SIGTERM graceful shutdown
echo -e "${GREEN}Test 7: Testing SIGTERM graceful shutdown...${NC}"
kill -TERM "$SERVER_PID"
sleep 2

# Verify server stopped
if kill -0 "$SERVER_PID" 2>/dev/null; then
    echo -e "${YELLOW}Server still running, forcing shutdown...${NC}"
    kill -9 "$SERVER_PID"
    echo -e "${RED}FAIL: Server didn't stop gracefully${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server stopped gracefully with SIGTERM${NC}"
rm -f "$PID_FILE"
echo ""

# Test 8: Start server again and test SIGINT
echo -e "${GREEN}Test 8: Testing SIGINT (Ctrl+C)...${NC}"
$BINARY --config "$CONFIG_FILE" start &
SERVER_PID=$!
echo $SERVER_PID > "$PID_FILE"
sleep 2

kill -INT "$SERVER_PID"
sleep 2

if kill -0 "$SERVER_PID" 2>/dev/null; then
    echo -e "${YELLOW}Server still running, forcing shutdown...${NC}"
    kill -9 "$SERVER_PID"
    echo -e "${RED}FAIL: Server didn't stop gracefully${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Server stopped gracefully with SIGINT${NC}"
rm -f "$PID_FILE"
echo ""

echo -e "${GREEN}=== All Signal Tests Passed! ===${NC}"
echo ""
echo "Summary:"
echo "  ✓ Server starts and stops gracefully"
echo "  ✓ DNS resolution works correctly"
echo "  ✓ Blocklist filtering works"
echo "  ✓ SIGHUP reloads blocklist without downtime"
echo "  ✓ Multiple reloads work correctly"
echo "  ✓ SIGTERM graceful shutdown works"
echo "  ✓ SIGINT graceful shutdown works"
