#!/bin/bash
# Quick setup script for Nix in WSL2 for NSFW
# Run this if you're getting permission errors or "no channels" errors

set -e

echo "🔧 NSFW - Nix Setup for WSL2"
echo "=============================="
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "⚠️  Please run as your regular user, not root"
    echo "   Usage: ./setup-nix-wsl2.sh"
    exit 1
fi

USER_NAME=$(whoami)

echo "📋 Checking Nix installation..."

# Check if Nix is installed
if ! command -v nix &> /dev/null; then
    echo "❌ Nix is not installed!"
    echo ""
    echo "Please install Nix first:"
    echo "  curl -L https://nixos.org/nix/install | sh -s -- --daemon"
    exit 1
fi

echo "✅ Nix is installed: $(nix --version)"
echo ""

# Check if user is in nix-users group
echo "📋 Checking group membership..."
if groups | grep -q nix-users; then
    echo "✅ You are in the nix-users group"
else
    echo "⚠️  You are NOT in the nix-users group"
    echo "   Adding you now (requires sudo)..."
    sudo usermod -a -G nix-users "$USER_NAME"
    echo "✅ Added to nix-users group"
    echo ""
    echo "⚠️  IMPORTANT: You need to log out and back in for group changes to take effect"
    echo "   Or run: newgrp nix-users"
fi
echo ""

# Check if daemon is running
echo "📋 Checking Nix daemon..."
if pgrep -x nix-daemon > /dev/null; then
    echo "✅ Nix daemon is running"
else
    echo "⚠️  Nix daemon is NOT running"
    echo "   Starting daemon (requires sudo)..."
    if command -v systemctl &> /dev/null; then
        sudo systemctl start nix-daemon
        sudo systemctl enable nix-daemon
        echo "✅ Daemon started via systemd"
    else
        echo "   No systemd found - you may need to start manually:"
        echo "   sudo nix-daemon --daemon &"
    fi
fi
echo ""

# Check channels
echo "📋 Checking Nix channels..."
if nix-channel --list | grep -q nixpkgs; then
    echo "✅ Channels are configured"
    nix-channel --list
else
    echo "⚠️  No channels configured"
    echo "   Adding nixpkgs channel..."
    nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixpkgs
    echo "✅ Channel added"
fi
echo ""

# Update channels
echo "📋 Updating Nix channels (this may take 2-5 minutes)..."
nix-channel --update
echo "✅ Channels updated"
echo ""

# Test Nix
echo "📋 Testing Nix search..."
if timeout 30 nix --extra-experimental-features "nix-command flakes" search nixpkgs hello --json > /dev/null 2>&1; then
    echo "✅ Nix search is working!"
else
    echo "⚠️  Nix search test failed or timed out"
    echo "   This might be normal on first run - try running NSFW to test"
fi
echo ""

echo "🎉 Setup complete!"
echo ""
echo "Next steps:"
echo "1. If you saw group membership warnings, log out and back in"
echo "2. Test NSFW from Windows PowerShell:"
echo "   cd C:\\Users\\YOUR_USERNAME\\nsfw-test"
echo "   .\\nsfw.exe search hello --limit 5"
echo ""
