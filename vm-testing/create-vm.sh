#!/usr/bin/env bash
#
# Create Windows 11 VM for NSFW Testing
# Optimized for WSL2 + Nix development
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ISO_DIR="$SCRIPT_DIR/iso"
ISO_FILE="$ISO_DIR/Win11_25H2_English_x64.iso"

# VM Configuration
VM_NAME="nsfw-test-win11"
VM_RAM=8192        # 8GB RAM
VM_CPUS=4          # 4 CPU cores
VM_DISK_SIZE=60    # 60GB disk
VM_DISK_DIR="$HOME/.local/share/libvirt/images"
VM_DISK="$VM_DISK_DIR/$VM_NAME.qcow2"

echo "🖥️  NSFW Windows Testing VM Setup"
echo "=================================="
echo ""

# Check for ISO
if [ ! -f "$ISO_FILE" ]; then
    echo "❌ Windows 11 ISO not found!"
    echo ""
    echo "Expected location: $ISO_FILE"
    echo ""
    echo "Run ./download-windows-iso.sh first, then manually download"
    echo "the ISO and place it in the iso/ directory."
    exit 1
fi

echo "✅ Found Windows 11 ISO: $(basename "$ISO_FILE")"
echo "   Size: $(du -h "$ISO_FILE" | cut -f1)"
echo ""

# Check libvirt
if ! systemctl is-active --quiet libvirtd; then
    echo "⚠️  libvirtd is not running. Starting it..."
    sudo systemctl start libvirtd
    sleep 2
fi

echo "✅ libvirtd is running"
echo ""

# Check if VM already exists
if sudo virsh -c qemu:///system dominfo "$VM_NAME" &> /dev/null; then
    echo "⚠️  VM '$VM_NAME' already exists!"
    echo ""
    read -p "Delete existing VM and recreate? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Deleting existing VM..."
        sudo virsh -c qemu:///system destroy "$VM_NAME" 2>/dev/null || true
        sudo virsh -c qemu:///system undefine "$VM_NAME" 2>/dev/null || true
        rm -f "$VM_DISK"
        echo "✅ Deleted existing VM"
    else
        echo "Aborting. Use 'sudo virsh -c qemu:///system start $VM_NAME' to start existing VM."
        exit 0
    fi
fi

echo "Creating VM with specifications:"
echo "  Name:  $VM_NAME"
echo "  RAM:   ${VM_RAM}MB (8GB)"
echo "  CPUs:  $VM_CPUS cores"
echo "  Disk:  ${VM_DISK_SIZE}GB"
echo "  ISO:   $(basename "$ISO_FILE")"
echo ""

# Create disk directory if needed
mkdir -p "$VM_DISK_DIR"

# Create disk image
echo "📀 Creating disk image..."
qemu-img create -f qcow2 "$VM_DISK" "${VM_DISK_SIZE}G"
echo "✅ Disk created: $VM_DISK"
echo ""

# Create VM
echo "🚀 Creating VM..."
sudo virt-install \
    --connect qemu:///system \
    --name "$VM_NAME" \
    --ram "$VM_RAM" \
    --vcpus "$VM_CPUS" \
    --disk path="$VM_DISK",format=qcow2,bus=sata \
    --cdrom "$ISO_FILE" \
    --os-variant win11 \
    --network network=default,model=e1000e \
    --graphics vnc,listen=0.0.0.0,port=5900 \
    --video vga \
    --boot uefi \
    --features kvm_hidden=on \
    --cpu host-passthrough \
    --noautoconsole

echo ""
echo "✅ VM created successfully!"
echo ""
echo "🎮 Access the VM:"
echo "   VNC:         vncviewer localhost:5900"
echo "   virt-viewer: virt-viewer $VM_NAME"
echo "   virt-manager: virt-manager (GUI)"
echo ""
echo "📋 Next steps:"
echo "   1. Connect to VM display"
echo "   2. Install Windows 11"
echo "   3. Install WSL2: wsl --install"
echo "   4. Install Nix in WSL2"
echo "   5. Build and test NSFW"
echo ""
echo "💾 VM Management:"
echo "   Start:   virsh start $VM_NAME"
echo "   Stop:    virsh shutdown $VM_NAME"
echo "   Destroy: virsh destroy $VM_NAME"
echo "   Delete:  virsh undefine $VM_NAME && rm $VM_DISK"
echo ""
echo "📸 Snapshot before testing:"
echo "   virsh snapshot-create-as $VM_NAME clean-install 'Fresh install'"
echo ""
echo "🔧 For detailed instructions, see: README.md"
