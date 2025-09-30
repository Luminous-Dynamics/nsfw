#!/usr/bin/env bash
#
# Check if host system is ready for VM testing
# Verifies all prerequisites before starting Day 16
#

set -euo pipefail

echo "🔍 NSFW VM Testing - Host Readiness Check"
echo "=========================================="
echo ""

ERRORS=0
WARNINGS=0

# Function to check and report
check_pass() {
    echo "✅ $1"
}

check_warn() {
    echo "⚠️  $1"
    ((WARNINGS++))
}

check_fail() {
    echo "❌ $1"
    ((ERRORS++))
}

# Check 1: Virtualization support
echo "🖥️  Checking Virtualization Support..."
if lscpu | grep -qi "virtualization.*VT-x\|AMD-V"; then
    VIRT_TYPE=$(lscpu | grep -i virtualization | awk '{print $2}')
    check_pass "Virtualization enabled: $VIRT_TYPE"
else
    check_fail "Virtualization not enabled in BIOS"
    echo "   Enable VT-x (Intel) or AMD-V (AMD) in BIOS settings"
fi
echo ""

# Check 2: libvirt daemon
echo "🐧 Checking libvirt..."
if systemctl is-active --quiet libvirtd; then
    check_pass "libvirtd is running"
else
    check_warn "libvirtd is not running"
    echo "   Start with: sudo systemctl start libvirtd"
fi
echo ""

# Check 3: Disk space
echo "💾 Checking Disk Space..."
DISK_DIR="$HOME/.local/share/libvirt/images"
mkdir -p "$DISK_DIR" 2>/dev/null || true

if [ -d "$DISK_DIR" ]; then
    AVAILABLE=$(df -BG "$DISK_DIR" | tail -1 | awk '{print $4}' | sed 's/G//')
    if [ "$AVAILABLE" -gt 70 ]; then
        check_pass "Disk space: ${AVAILABLE}GB available (need 70GB)"
    elif [ "$AVAILABLE" -gt 60 ]; then
        check_warn "Disk space: ${AVAILABLE}GB available (70GB recommended)"
    else
        check_fail "Disk space: ${AVAILABLE}GB available (need at least 60GB)"
    fi
else
    check_warn "Cannot check disk space for $DISK_DIR"
fi
echo ""

# Check 4: Required commands
echo "🛠️  Checking Required Tools..."

if command -v virsh &> /dev/null; then
    check_pass "virsh: $(virsh --version)"
else
    check_fail "virsh not found (part of libvirt)"
fi

if command -v virt-install &> /dev/null; then
    check_pass "virt-install available"
else
    check_fail "virt-install not found (needed for VM creation)"
fi

if command -v qemu-img &> /dev/null; then
    check_pass "qemu-img: $(qemu-img --version | head -1)"
else
    check_fail "qemu-img not found (part of qemu)"
fi

if command -v virt-viewer &> /dev/null; then
    check_pass "virt-viewer available"
else
    check_warn "virt-viewer not found (recommended for GUI access)"
    echo "   Install with: nix-shell -p virt-viewer"
fi

if command -v vncviewer &> /dev/null; then
    check_pass "vncviewer available (alternative to virt-viewer)"
else
    check_warn "vncviewer not found (alternative for VM access)"
fi
echo ""

# Check 5: Network connectivity
echo "🌐 Checking Network..."
if ping -c 1 google.com &> /dev/null; then
    check_pass "Internet connectivity available"
else
    check_fail "No internet connection (needed for ISO download and packages)"
fi
echo ""

# Check 6: libvirt default network
echo "🔌 Checking libvirt Network..."
if virsh net-list --all 2>/dev/null | grep -q "default"; then
    if virsh net-list 2>/dev/null | grep -q "default.*active"; then
        check_pass "Default network is active"
    else
        check_warn "Default network exists but not active"
        echo "   Start with: sudo virsh net-start default"
    fi
else
    check_warn "Default network not found"
    echo "   Create with: sudo virsh net-define /etc/libvirt/qemu/networks/default.xml"
fi
echo ""

# Check 7: Permissions
echo "👤 Checking Permissions..."
if groups | grep -q "libvirt\|kvm"; then
    check_pass "User is in libvirt/kvm group"
else
    check_warn "User not in libvirt or kvm group"
    echo "   Add with: sudo usermod -aG libvirt,kvm $USER"
    echo "   Then logout and login again"
fi
echo ""

# Check 8: Rust/Cargo (for building NSFW)
echo "🦀 Checking Rust (for building NSFW)..."
if command -v cargo &> /dev/null; then
    CARGO_VER=$(cargo --version)
    check_pass "cargo: $CARGO_VER"
else
    check_warn "cargo not found (needed to build NSFW)"
    echo "   Can build in WSL2 instead"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo "🎉 HOST IS READY FOR VM TESTING!"
    echo ""
    echo "Next steps:"
    echo "1. cd vm-testing"
    echo "2. ./download-windows-iso.sh"
    echo "3. ./create-vm.sh"
    echo ""
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo "⚠️  HOST IS MOSTLY READY (${WARNINGS} warnings)"
    echo ""
    echo "You can proceed, but consider fixing warnings for best experience."
    echo ""
    echo "Next steps:"
    echo "1. cd vm-testing"
    echo "2. ./download-windows-iso.sh"
    echo "3. ./create-vm.sh"
    echo ""
    exit 0
else
    echo "❌ HOST IS NOT READY (${ERRORS} errors, ${WARNINGS} warnings)"
    echo ""
    echo "Fix the errors above before proceeding."
    echo ""
    exit 1
fi
