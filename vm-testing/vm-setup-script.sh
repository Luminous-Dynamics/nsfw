#!/bin/bash
# NSFW VM Setup Script
# Run this in WSL2 on your Windows VM after copying the source code

set -e  # Exit on error

echo "╔════════════════════════════════════════════════╗"
echo "║         NSFW VM Setup Script                   ║"
echo "║  Setting up NSFW for Windows testing           ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

# Check if we're in WSL2
if ! grep -qi microsoft /proc/version; then
    echo "⚠️  Warning: Doesn't look like WSL2. Are you running this in WSL?"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Step 1: Install Rust
echo "📦 Step 1: Installing Rust toolchain..."
if command -v cargo &> /dev/null; then
    echo "✓ Rust already installed: $(rustc --version)"
else
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed: $(rustc --version)"
fi

# Step 2: Install build tools
echo ""
echo "🔧 Step 2: Installing build tools..."
sudo apt update -qq
sudo apt install -y build-essential mingw-w64 pkg-config libssl-dev
echo "✓ Build tools installed"

# Step 3: Check if source code is present
echo ""
echo "📂 Step 3: Checking for NSFW source code..."
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Can't find Cargo.toml"
    echo "   Please run this script from the nsfw directory"
    echo "   (The directory with Cargo.toml in it)"
    exit 1
fi
echo "✓ Found NSFW source code"

# Step 4: Build Linux binary
echo ""
echo "🔨 Step 4: Building Linux binary..."
cargo build --release
echo "✓ Linux binary built: target/release/nsfw"

# Step 5: Test Linux binary
echo ""
echo "🧪 Step 5: Testing Linux binary..."
if ./target/release/nsfw --version &> /dev/null; then
    echo "✓ Basic functionality verified"
    ./target/release/nsfw --version
else
    echo "❌ Warning: Binary test failed"
fi

# Step 6: Add Windows target
echo ""
echo "🎯 Step 6: Adding Windows cross-compilation target..."
rustup target add x86_64-pc-windows-gnu
echo "✓ Windows target added"

# Step 7: Build Windows binary
echo ""
echo "🔨 Step 7: Building Windows binary (this may take a few minutes)..."
cargo build --release --target x86_64-pc-windows-gnu
echo "✓ Windows binary built: target/x86_64-pc-windows-gnu/release/nsfw.exe"

# Step 8: Copy to Windows Downloads
echo ""
echo "📋 Step 8: Copying nsfw.exe to Windows..."
WINDOWS_USER=$(cmd.exe /c "echo %USERNAME%" 2>/dev/null | tr -d '\r')
if [ -z "$WINDOWS_USER" ]; then
    WINDOWS_USER="$USER"
fi

DEST_DIR="/mnt/c/Users/$WINDOWS_USER/Downloads"
if [ -d "$DEST_DIR" ]; then
    cp target/x86_64-pc-windows-gnu/release/nsfw.exe "$DEST_DIR/"
    echo "✓ Copied to: $DEST_DIR/nsfw.exe"
else
    echo "⚠️  Couldn't find Downloads folder at $DEST_DIR"
    echo "   You can manually copy from:"
    echo "   $(pwd)/target/x86_64-pc-windows-gnu/release/nsfw.exe"
fi

# Step 9: Summary
echo ""
echo "╔════════════════════════════════════════════════╗"
echo "║              ✅ SETUP COMPLETE!                 ║"
echo "╚════════════════════════════════════════════════╝"
echo ""
echo "📁 Files created:"
echo "   Linux binary:   $(pwd)/target/release/nsfw"
echo "   Windows binary: $(pwd)/target/x86_64-pc-windows-gnu/release/nsfw.exe"
if [ -d "$DEST_DIR" ]; then
    echo "   Copied to:      $DEST_DIR/nsfw.exe"
fi
echo ""
echo "🧪 Test from WSL2:"
echo "   ./target/release/nsfw search vim"
echo ""
echo "🪟 Test from Windows PowerShell:"
echo "   cd ~\\Downloads"
echo "   .\\nsfw.exe search vim"
echo ""
echo "💡 Next steps:"
echo "   1. Open Windows PowerShell"
echo "   2. Run: cd ~\\Downloads"
echo "   3. Run: .\\nsfw.exe --version"
echo "   4. Start testing with TESTING_CHECKLIST.md"
echo ""
echo "🎉 Happy testing!"
