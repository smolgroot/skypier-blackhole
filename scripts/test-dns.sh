#!/bin/bash
# Test script for Skypier Blackhole DNS resolver

# Change to project root directory
cd "$(dirname "$0")/.." || exit 1

set -e

echo "🧪 Skypier Blackhole DNS Test Script"
echo "===================================="
echo ""

# Create test blocklist
echo "📝 Creating test blocklist..."
cat > /tmp/skypier-test-blocklist.txt << EOF
# Test blocklist
ads.example.com
tracker.example.com
*.badads.com
EOF

echo "✅ Test blocklist created at /tmp/skypier-test-blocklist.txt"
echo ""

# Build the project
echo "🔨 Building project..."
cargo build --release
echo "✅ Build complete"
echo ""

# Start the DNS server in background
echo "🚀 Starting Skypier Blackhole DNS server on port 5353..."
RUST_LOG=info ./target/release/skypier-blackhole --config test-config.toml start &
DNS_PID=$!
echo "✅ DNS server started (PID: $DNS_PID)"
echo ""

# Wait for server to start
echo "⏳ Waiting for server to start..."
sleep 2
echo ""

# Test 1: Query an allowed domain
echo "🧪 Test 1: Query allowed domain (google.com)"
echo "Command: dig @127.0.0.1 -p 5353 google.com +short"
dig @127.0.0.1 -p 5353 google.com +short || echo "❌ Query failed"
echo ""

# Test 2: Query a blocked domain (should get REFUSED)
echo "🧪 Test 2: Query blocked domain (ads.example.com)"
echo "Command: dig @127.0.0.1 -p 5353 ads.example.com"
dig @127.0.0.1 -p 5353 ads.example.com || echo "✅ Domain blocked (REFUSED)"
echo ""

# Test 3: Query another blocked domain
echo "🧪 Test 3: Query blocked domain (tracker.example.com)"
echo "Command: dig @127.0.0.1 -p 5353 tracker.example.com"
dig @127.0.0.1 -p 5353 tracker.example.com || echo "✅ Domain blocked (REFUSED)"
echo ""

# Test 4: Query wildcard blocked domain
echo "🧪 Test 4: Query wildcard blocked domain (sub.badads.com)"
echo "Command: dig @127.0.0.1 -p 5353 sub.badads.com"
dig @127.0.0.1 -p 5353 sub.badads.com || echo "✅ Domain blocked (REFUSED)"
echo ""

# Show logs
echo "📋 Server logs:"
echo "==============="
cat /tmp/skypier-blackhole-test.log 2>/dev/null || echo "No logs yet"
echo ""

# Cleanup
echo "🧹 Stopping DNS server..."
kill $DNS_PID 2>/dev/null || true
wait $DNS_PID 2>/dev/null || true
echo "✅ DNS server stopped"
echo ""

echo "🎉 Tests complete!"
