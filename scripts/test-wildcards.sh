#!/bin/bash
# Test wildcard domain blocking

# Change to project root directory
cd "$(dirname "$0")/.." || exit 1

set -e

echo "🧪 Skypier Blackhole Wildcard Domain Test"
echo "=========================================="
echo ""

# Create test config with wildcard blocklist
cat > /tmp/test-wildcard-config.toml << 'EOF'
[server]
listen_addr = "127.0.0.1"
listen_port = 15354
upstream_dns = ["1.1.1.1:53"]
blocked_response = "refused"

[blocklist]
custom_list = "/tmp/skypier-wildcard-test.txt"
local_lists = []
remote_lists = []
enable_wildcards = true

[logging]
log_level = "info"
log_blocked = true

[updater]
enabled = false
EOF

echo "✅ Created test config with wildcard blocklist"
echo ""

# Build if needed
if [ ! -f ./target/release/skypier-blackhole ]; then
    echo "Building release binary..."
    cargo build --release
fi

# Start server in background
echo "🚀 Starting DNS server on port 15354..."
RUST_LOG=info ./target/release/skypier-blackhole --config /tmp/test-wildcard-config.toml start &
SERVER_PID=$!
echo "Server PID: $SERVER_PID"

# Give server time to start
sleep 2

# Check if server is running
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "❌ FAIL: Server failed to start"
    exit 1
fi

echo ""
echo "Running wildcard tests..."
echo "=========================="
echo ""

# Test 1: Exact domain blocking
echo "Test 1: Exact domain (ads.example.com) should be BLOCKED"
RESULT=$(dig @127.0.0.1 -p 15354 ads.example.com +short +time=2 +tries=1 || true)
if [ -z "$RESULT" ]; then
    echo "✅ PASS: ads.example.com is blocked"
else
    echo "❌ FAIL: ads.example.com not blocked, got: $RESULT"
fi
echo ""

# Test 2: Wildcard subdomain blocking
echo "Test 2: Wildcard subdomain (sub.doubleclick.net) should be BLOCKED"
RESULT=$(dig @127.0.0.1 -p 15354 sub.doubleclick.net +short +time=2 +tries=1 || true)
if [ -z "$RESULT" ]; then
    echo "✅ PASS: sub.doubleclick.net is blocked by *.doubleclick.net"
else
    echo "❌ FAIL: sub.doubleclick.net not blocked, got: $RESULT"
fi
echo ""

# Test 3: Deep wildcard subdomain
echo "Test 3: Deep subdomain (a.b.c.googlesyndication.com) should be BLOCKED"
RESULT=$(dig @127.0.0.1 -p 15354 a.b.c.googlesyndication.com +short +time=2 +tries=1 || true)
if [ -z "$RESULT" ]; then
    echo "✅ PASS: a.b.c.googlesyndication.com is blocked by *.googlesyndication.com"
else
    echo "❌ FAIL: a.b.c.googlesyndication.com not blocked, got: $RESULT"
fi
echo ""

# Test 4: Base domain should NOT be blocked
echo "Test 4: Base domain (doubleclick.net) should be ALLOWED"
RESULT=$(dig @127.0.0.1 -p 15354 doubleclick.net +short +time=2 +tries=1 || true)
if [ -n "$RESULT" ]; then
    echo "✅ PASS: doubleclick.net is allowed (wildcard doesn't match base): $RESULT"
else
    echo "❌ FAIL: doubleclick.net should not be blocked by *.doubleclick.net"
fi
echo ""

# Test 5: Multi-level wildcard
echo "Test 5: Multi-level wildcard (tracker.ads.facebook.com) should be BLOCKED"
RESULT=$(dig @127.0.0.1 -p 15354 tracker.ads.facebook.com +short +time=2 +tries=1 || true)
if [ -z "$RESULT" ]; then
    echo "✅ PASS: tracker.ads.facebook.com is blocked by *.ads.facebook.com"
else
    echo "❌ FAIL: tracker.ads.facebook.com not blocked, got: $RESULT"
fi
echo ""

# Test 6: Parent of multi-level wildcard should be allowed
echo "Test 6: Parent domain (ads.facebook.com) should be ALLOWED"
RESULT=$(dig @127.0.0.1 -p 15354 ads.facebook.com +short +time=2 +tries=1 || true)
if [ -n "$RESULT" ]; then
    echo "✅ PASS: ads.facebook.com is allowed (wildcard doesn't match base): $RESULT"
else
    echo "❌ FAIL: ads.facebook.com should not be blocked by *.ads.facebook.com"
fi
echo ""

# Test 7: Unrelated domain should be allowed
echo "Test 7: Unrelated domain (google.com) should be ALLOWED"
RESULT=$(dig @127.0.0.1 -p 15354 google.com +short +time=2 +tries=1 || true)
if [ -n "$RESULT" ]; then
    echo "✅ PASS: google.com resolves: $RESULT"
else
    echo "❌ FAIL: google.com should not be blocked"
fi
echo ""

# Cleanup
echo "🧹 Cleaning up..."
kill -TERM "$SERVER_PID" 2>/dev/null || true
sleep 1
kill -9 "$SERVER_PID" 2>/dev/null || true

echo ""
echo "✅ All wildcard tests completed!"
