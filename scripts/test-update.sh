#!/bin/bash
#
# Test script for remote blocklist download feature
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Remote Blocklist Download Test${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Build binary
echo -e "${YELLOW}[1/8]${NC} Building release binary..."
cargo build --release --quiet
echo -e "${GREEN}✓${NC} Build successful"
echo ""

BINARY="./target/release/skypier-blackhole"
CONFIG="test-update-config.toml"
CACHE_FILE="/tmp/remote-blocklist-cache.txt"
CUSTOM_FILE="/tmp/skypier-update-test-custom.txt"

# Clean up old files
rm -f "$CACHE_FILE" "$CUSTOM_FILE"

# Test 1: Update with no config
echo -e "${YELLOW}[2/8]${NC} Testing update with no remote sources..."
cat > /tmp/test-update-empty.toml <<EOF
[server]
listen_addr = "127.0.0.1"
listen_port = 15356
upstream_dns = ["1.1.1.1:53"]
blocked_response = "refused"

[blocklist]
custom_list = "/tmp/test-empty-custom.txt"
local_lists = []
remote_lists = []

[logging]
log_blocked = true
log_path = "/tmp/test-empty.log"
log_level = "info"

[updater]
enabled = false
schedule = "0 0 * * *"
timezone = "EST"
EOF

OUTPUT=$($BINARY update --config /tmp/test-update-empty.toml 2>&1)
if echo "$OUTPUT" | grep -q "No remote sources configured"; then
    echo -e "${GREEN}✓${NC} Correctly handled empty remote list"
else
    echo -e "${RED}✗${NC} Failed to handle empty remote list"
    exit 1
fi
echo ""

# Test 2: Download real blocklist
echo -e "${YELLOW}[3/8]${NC} Downloading real blocklist (StevenBlack hosts)..."
echo "custom-domain.com" > "$CUSTOM_FILE"

OUTPUT=$($BINARY update --config "$CONFIG" 2>&1)
if echo "$OUTPUT" | grep -q "Downloaded.*domains"; then
    DOMAIN_COUNT=$(echo "$OUTPUT" | grep -oP "Downloaded \K\d+")
    echo -e "${GREEN}✓${NC} Successfully downloaded ${CYAN}$DOMAIN_COUNT${NC} domains"
else
    echo -e "${RED}✗${NC} Failed to download blocklist"
    exit 1
fi
echo ""

# Test 3: Verify cache file created
echo -e "${YELLOW}[4/8]${NC} Verifying cache file..."
if [ -f "$CACHE_FILE" ]; then
    LINES=$(wc -l < "$CACHE_FILE")
    echo -e "${GREEN}✓${NC} Cache file created with ${CYAN}$LINES${NC} lines"
else
    echo -e "${RED}✗${NC} Cache file not created"
    exit 1
fi
echo ""

# Test 4: Verify domains are valid
echo -e "${YELLOW}[5/8]${NC} Verifying domain format..."
FIRST_DOMAIN=$(head -1 "$CACHE_FILE")
if [[ "$FIRST_DOMAIN" =~ ^[a-z0-9.-]+$ ]]; then
    echo -e "${GREEN}✓${NC} Domains have valid format (example: ${CYAN}$FIRST_DOMAIN${NC})"
else
    echo -e "${RED}✗${NC} Invalid domain format: $FIRST_DOMAIN"
    exit 1
fi
echo ""

# Test 5: Verify no IPs in blocklist
echo -e "${YELLOW}[6/8]${NC} Checking for IP addresses (should be filtered)..."
IP_COUNT=$(grep -cE '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' "$CACHE_FILE" || true)
if [ "$IP_COUNT" -eq 0 ]; then
    echo -e "${GREEN}✓${NC} No IP addresses found (correctly filtered)"
else
    echo -e "${RED}✗${NC} Found $IP_COUNT IP addresses (should be 0)"
    exit 1
fi
echo ""

# Test 6: Test that downloaded domains are blocked
echo -e "${YELLOW}[7/8]${NC} Testing downloaded domains are blocked..."
# a-ads.com should be in the StevenBlack list
OUTPUT=$($BINARY test a-ads.com --config "$CONFIG" 2>&1)
if echo "$OUTPUT" | grep -q "BLOCKED"; then
    echo -e "${GREEN}✓${NC} Downloaded domain (a-ads.com) is correctly blocked"
else
    echo -e "${RED}✗${NC} Downloaded domain not blocked"
    exit 1
fi
echo ""

# Test 7: Test that good domains are allowed
echo -e "${YELLOW}[8/8]${NC} Testing legitimate domains are allowed..."
OUTPUT=$($BINARY test google.com --config "$CONFIG" 2>&1)
if echo "$OUTPUT" | grep -q "ALLOWED"; then
    echo -e "${GREEN}✓${NC} Legitimate domain (google.com) is correctly allowed"
else
    echo -e "${RED}✗${NC} Legitimate domain incorrectly blocked"
    exit 1
fi
echo ""

# Summary
echo -e "${CYAN}========================================${NC}"
echo -e "${GREEN}✓ All download tests passed! (8/8)${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Show statistics
echo -e "${CYAN}Download Statistics:${NC}"
echo "  • Total domains downloaded: ${CYAN}$DOMAIN_COUNT${NC}"
echo "  • Cache file: ${CYAN}$CACHE_FILE${NC}"
echo "  • File size: ${CYAN}$(du -h $CACHE_FILE | cut -f1)${NC}"
echo ""

# Show sample blocked domains
echo -e "${CYAN}Sample blocked domains:${NC}"
head -5 "$CACHE_FILE" | while read domain; do
    echo "  • $domain"
done
echo ""

echo -e "${GREEN}Remote blocklist download feature is working perfectly!${NC}"
echo ""
