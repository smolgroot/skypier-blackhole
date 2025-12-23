#!/bin/bash
#
# Test script for CLI commands with colorful output
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for test output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}================================${NC}"
echo -e "${CYAN}  CLI Commands Test Suite${NC}"
echo -e "${CYAN}================================${NC}"
echo ""

# Build binary
echo -e "${YELLOW}[1/10]${NC} Building release binary..."
cargo build --release --quiet
echo -e "${GREEN}✓${NC} Build successful"
echo ""

BINARY="./target/release/skypier-blackhole"
CONFIG="test-config.toml"

# Test 1: Help command
echo -e "${YELLOW}[2/10]${NC} Testing --help..."
$BINARY --help > /dev/null
echo -e "${GREEN}✓${NC} Help command works"
echo ""

# Test 2: No command (default help)
echo -e "${YELLOW}[3/10]${NC} Testing default command (no subcommand)..."
OUTPUT=$($BINARY 2>&1)
if echo "$OUTPUT" | grep -q "No command specified"; then
    echo -e "${GREEN}✓${NC} Default help message displayed"
else
    echo -e "${RED}✗${NC} Default command failed"
    exit 1
fi
echo ""

# Test 3: Status command
echo -e "${YELLOW}[4/10]${NC} Testing status command..."
OUTPUT=$($BINARY status --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "Skypier Blackhole Status"; then
    echo -e "${GREEN}✓${NC} Status command works"
else
    echo -e "${RED}✗${NC} Status command failed"
    exit 1
fi
echo ""

# Test 4: List command
echo -e "${YELLOW}[5/10]${NC} Testing list command..."
OUTPUT=$($BINARY list --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "Blocklist Statistics"; then
    echo -e "${GREEN}✓${NC} List command works"
else
    echo -e "${RED}✗${NC} List command failed"
    exit 1
fi
echo ""

# Test 5: Test command (blocked domain)
echo -e "${YELLOW}[6/10]${NC} Testing blocked domain..."
OUTPUT=$($BINARY test ads.example.com --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "BLOCKED"; then
    echo -e "${GREEN}✓${NC} Blocked domain detected correctly"
else
    echo -e "${RED}✗${NC} Blocked domain test failed"
    exit 1
fi
echo ""

# Test 6: Test command (allowed domain)
echo -e "${YELLOW}[7/10]${NC} Testing allowed domain..."
OUTPUT=$($BINARY test google.com --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "ALLOWED"; then
    echo -e "${GREEN}✓${NC} Allowed domain detected correctly"
else
    echo -e "${RED}✗${NC} Allowed domain test failed"
    exit 1
fi
echo ""

# Test 7: Add command
echo -e "${YELLOW}[8/10]${NC} Testing add command..."
$BINARY add cli-test.badsite.com --config $CONFIG 2>&1 | grep -q "Domain added"
echo -e "${GREEN}✓${NC} Add command works"
echo ""

# Test 8: Verify added domain is blocked
echo -e "${YELLOW}[9/10]${NC} Verifying added domain is blocked..."
OUTPUT=$($BINARY test cli-test.badsite.com --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "BLOCKED"; then
    echo -e "${GREEN}✓${NC} Added domain is blocked"
else
    echo -e "${RED}✗${NC} Added domain is not blocked"
    exit 1
fi
echo ""

# Test 9: Remove command
echo -e "${YELLOW}[10/10]${NC} Testing remove command..."
$BINARY remove cli-test.badsite.com --config $CONFIG 2>&1 | grep -q "Domain removed"
echo -e "${GREEN}✓${NC} Remove command works"
echo ""

# Test 10: Update command (stub)
echo -e "${YELLOW}[BONUS]${NC} Testing update command..."
OUTPUT=$($BINARY update --config $CONFIG 2>&1)
if echo "$OUTPUT" | grep -q "Implementation coming soon"; then
    echo -e "${GREEN}✓${NC} Update command stub works"
else
    echo -e "${RED}✗${NC} Update command failed"
    exit 1
fi
echo ""

echo -e "${CYAN}================================${NC}"
echo -e "${GREEN}✓ All CLI tests passed! (10/10)${NC}"
echo -e "${CYAN}================================${NC}"
echo ""
echo -e "${CYAN}Colorful CLI commands:${NC}"
echo "  • status  - Show server status"
echo "  • test    - Test if domain is blocked"
echo "  • list    - Show blocklist statistics"
echo "  • add     - Add domain to blocklist"
echo "  • remove  - Remove domain from blocklist"
echo "  • stop    - Stop running server"
echo "  • reload  - Hot-reload blocklists"
echo "  • update  - Download remote blocklists (coming soon)"
echo ""
