#!/usr/bin/env bash
#
# VM Management Helper Script
# Quick commands for managing nsfw-test-win11 VM
#

set -euo pipefail

VM_NAME="nsfw-test-win11"

show_help() {
    cat << EOF
🖥️  VM Management Helper

Usage: $0 <command>

Commands:
    start       Start the VM
    stop        Shutdown the VM gracefully
    kill        Force stop the VM
    restart     Restart the VM
    status      Show VM status and info
    console     Open graphical console (virt-viewer)
    vnc         Show VNC connection info
    snapshot    Create a snapshot
    restore     Restore from snapshot
    list        List all snapshots
    delete      Delete the VM (with confirmation)
    info        Show detailed VM information
    monitor     Watch VM resource usage

Examples:
    $0 start            # Start the VM
    $0 console          # Open graphical window
    $0 snapshot         # Create snapshot for testing
    $0 restore          # Restore to last snapshot

EOF
}

check_vm_exists() {
    if ! virsh dominfo "$VM_NAME" &> /dev/null; then
        echo "❌ VM '$VM_NAME' does not exist!"
        echo "Run ./create-vm.sh to create it."
        exit 1
    fi
}

cmd_start() {
    check_vm_exists
    echo "🚀 Starting VM..."
    virsh start "$VM_NAME"
    echo "✅ VM started"
    echo ""
    echo "Connect with:"
    echo "  virt-viewer $VM_NAME"
    echo "  or"
    echo "  vncviewer localhost:5900"
}

cmd_stop() {
    check_vm_exists
    echo "🛑 Shutting down VM gracefully..."
    virsh shutdown "$VM_NAME"
    echo "⏳ Waiting for shutdown..."
    echo "(This may take a minute)"
}

cmd_kill() {
    check_vm_exists
    echo "⚡ Force stopping VM..."
    virsh destroy "$VM_NAME"
    echo "✅ VM stopped"
}

cmd_restart() {
    check_vm_exists
    echo "🔄 Restarting VM..."
    virsh reboot "$VM_NAME"
    echo "✅ Reboot signal sent"
}

cmd_status() {
    check_vm_exists
    echo "📊 VM Status"
    echo "============"
    echo ""

    STATE=$(virsh domstate "$VM_NAME")
    echo "State: $STATE"

    if [ "$STATE" = "running" ]; then
        echo ""
        virsh dominfo "$VM_NAME" | grep -E "(CPU|Max memory|Used memory)"
        echo ""
        echo "Console access:"
        echo "  VNC: localhost:5900"
        echo "  virt-viewer: $VM_NAME"
    fi
}

cmd_console() {
    check_vm_exists
    echo "🖥️  Opening console..."

    if command -v virt-viewer &> /dev/null; then
        virt-viewer "$VM_NAME" &
        echo "✅ Console window opened"
    else
        echo "❌ virt-viewer not found"
        echo "Install with: nix-shell -p virt-viewer"
        echo ""
        echo "Alternative: Use VNC at localhost:5900"
    fi
}

cmd_vnc() {
    check_vm_exists
    echo "📺 VNC Connection Info"
    echo "====================="
    echo ""

    VNC=$(virsh vncdisplay "$VM_NAME" 2>/dev/null || echo "Not available")
    echo "VNC Display: $VNC"

    if [ "$VNC" != "Not available" ]; then
        echo ""
        echo "Connect with:"
        echo "  vncviewer localhost:5900"
        echo "  or"
        echo "  virt-viewer $VM_NAME"
    else
        echo ""
        echo "VM is not running or VNC is not configured"
    fi
}

cmd_snapshot() {
    check_vm_exists
    echo "📸 Create Snapshot"
    echo "=================="
    echo ""

    read -p "Snapshot name: " SNAP_NAME
    read -p "Description: " SNAP_DESC

    echo "Creating snapshot '$SNAP_NAME'..."
    virsh snapshot-create-as "$VM_NAME" "$SNAP_NAME" "$SNAP_DESC"
    echo "✅ Snapshot created"
}

cmd_restore() {
    check_vm_exists
    echo "⏮️  Restore from Snapshot"
    echo "========================"
    echo ""

    echo "Available snapshots:"
    virsh snapshot-list "$VM_NAME"
    echo ""

    read -p "Snapshot name to restore: " SNAP_NAME

    echo "Restoring to '$SNAP_NAME'..."
    virsh snapshot-revert "$VM_NAME" "$SNAP_NAME"
    echo "✅ Restored to snapshot"
}

cmd_list() {
    check_vm_exists
    echo "📋 Snapshots"
    echo "============"
    echo ""
    virsh snapshot-list "$VM_NAME"
}

cmd_delete() {
    check_vm_exists
    echo "🗑️  Delete VM"
    echo "============"
    echo ""
    echo "⚠️  WARNING: This will permanently delete:"
    echo "  - VM configuration"
    echo "  - VM disk image"
    echo "  - All snapshots"
    echo ""

    read -p "Type 'delete' to confirm: " CONFIRM

    if [ "$CONFIRM" = "delete" ]; then
        echo "Stopping VM..."
        virsh destroy "$VM_NAME" 2>/dev/null || true

        echo "Deleting VM..."
        virsh undefine "$VM_NAME"

        DISK="$HOME/.local/share/libvirt/images/$VM_NAME.qcow2"
        if [ -f "$DISK" ]; then
            echo "Deleting disk: $DISK"
            rm -f "$DISK"
        fi

        echo "✅ VM deleted"
    else
        echo "❌ Deletion cancelled"
    fi
}

cmd_info() {
    check_vm_exists
    echo "ℹ️  VM Information"
    echo "================="
    echo ""
    virsh dominfo "$VM_NAME"
    echo ""
    echo "Disk:"
    virsh domblklist "$VM_NAME"
    echo ""
    echo "Network:"
    virsh domiflist "$VM_NAME"
}

cmd_monitor() {
    check_vm_exists
    echo "📊 VM Resource Monitor"
    echo "====================="
    echo ""
    echo "Press Ctrl+C to stop"
    echo ""

    while true; do
        clear
        echo "VM: $VM_NAME - $(date)"
        echo "================================"
        echo ""

        STATE=$(virsh domstate "$VM_NAME")
        echo "State: $STATE"

        if [ "$STATE" = "running" ]; then
            echo ""
            virsh domstats "$VM_NAME" | grep -E "(cpu|balloon.current)"
        fi

        sleep 2
    done
}

# Main command dispatcher
case "${1:-}" in
    start)      cmd_start ;;
    stop)       cmd_stop ;;
    kill)       cmd_kill ;;
    restart)    cmd_restart ;;
    status)     cmd_status ;;
    console)    cmd_console ;;
    vnc)        cmd_vnc ;;
    snapshot)   cmd_snapshot ;;
    restore)    cmd_restore ;;
    list)       cmd_list ;;
    delete)     cmd_delete ;;
    info)       cmd_info ;;
    monitor)    cmd_monitor ;;
    help|--help|-h|"")
        show_help ;;
    *)
        echo "❌ Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
