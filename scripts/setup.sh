#!/bin/bash
#
# Skypier Blackhole - Production Setup Script
# This script sets up Skypier Blackhole with default configuration
#

set -e

echo "========================================="
echo " 🚀 Skypier Blackhole Setup"
echo "========================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  This script requires sudo privileges"
    echo "   Run with: sudo ./scripts/setup.sh"
    exit 1
fi

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "📦 Step 1: Creating directories..."
mkdir -p /etc/skypier
mkdir -p /var/log/skypier
echo "   ✓ Created /etc/skypier"
echo "   ✓ Created /var/log/skypier"
echo ""

echo "📝 Step 2: Installing configuration..."
if [ -f /etc/skypier/blackhole.toml ]; then
    echo "   ⚠️  Config already exists: /etc/skypier/blackhole.toml"
    echo "   Creating backup: /etc/skypier/blackhole.toml.backup"
    cp /etc/skypier/blackhole.toml /etc/skypier/blackhole.toml.backup
fi
cp config/blackhole.toml.example /etc/skypier/blackhole.toml
echo "   ✓ Installed /etc/skypier/blackhole.toml"
echo ""

echo "🚫 Step 3: Creating custom blocklist..."
if [ ! -f /etc/skypier/custom-blocklist.txt ]; then
    cat > /etc/skypier/custom-blocklist.txt << 'EOF'
# Skypier Blackhole Custom Blocklist
# Add one domain per line
# Wildcards supported: *.domain.com blocks all subdomains

# Example entries:
ads.example.com
tracker.example.com
*.doubleclick.net
EOF
    echo "   ✓ Created /etc/skypier/custom-blocklist.txt"
else
    echo "   ℹ️  Blocklist already exists: /etc/skypier/custom-blocklist.txt"
fi
echo ""

echo "🔧 Step 4: Installing binary..."
if [ -f target/release/skypier-blackhole ]; then
    cp target/release/skypier-blackhole /usr/bin/skypier-blackhole
    chmod +x /usr/bin/skypier-blackhole
    echo "   ✓ Installed /usr/bin/skypier-blackhole"
else
    echo "   ⚠️  Binary not found. Building..."
    cargo build --release
    cp target/release/skypier-blackhole /usr/bin/skypier-blackhole
    chmod +x /usr/bin/skypier-blackhole
    echo "   ✓ Built and installed /usr/bin/skypier-blackhole"
fi
echo ""

echo "📊 Step 5: Configuration summary"
echo "-------------------------------"
echo "Config file:    /etc/skypier/blackhole.toml"
echo "Blocklist:      /etc/skypier/custom-blocklist.txt"
echo "Binary:         /usr/bin/skypier-blackhole"
echo "Logs:           /var/log/skypier/blackhole.log"
echo ""

echo "========================================="
echo " ✅ Setup Complete!"
echo "========================================="
echo ""
echo "🎯 Quick Start Commands:"
echo ""
echo "  # Check status (no config needed)"
echo "  skypier-blackhole status"
echo ""
echo "  # Download remote blocklists"
echo "  skypier-blackhole update"
echo ""
echo "  # Test a domain"
echo "  skypier-blackhole test ads.example.com"
echo ""
echo "  # Add a domain to blocklist"
echo "  skypier-blackhole add malware.com"
echo ""
echo "  # Start the DNS server (requires port 53)"
echo "  sudo skypier-blackhole start"
echo ""
echo "📖 Config location: /etc/skypier/blackhole.toml"
echo "   Edit to customize settings"
echo ""
