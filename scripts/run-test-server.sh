#!/bin/bash
# Simple manual test - run the DNS server and test with dig

# Change to project root directory
cd "$(dirname "$0")/.." || exit 1

echo "🚀 Starting Skypier Blackhole on port 15353..."
echo "Press Ctrl+C to stop"
echo ""

# Create test blocklist
mkdir -p /tmp/skypier
cat > /tmp/skypier/test-blocklist.txt << 'EOF'
ads.example.com
tracker.example.com
doubleclick.net
EOF

echo "Test blocklist created with:"
cat /tmp/skypier/test-blocklist.txt
echo ""
echo "Starting server..."
echo ""
echo "Test with: dig @127.0.0.1 -p 15353 google.com"
echo ""

RUST_LOG=info cargo run -- --config test-config.toml start
