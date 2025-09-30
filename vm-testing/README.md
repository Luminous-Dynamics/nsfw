# Windows VM Testing Setup for NSFW

This directory contains scripts and configuration for testing NSFW on real Windows 11 with WSL2.

## Prerequisites

- NixOS host system with virtualization enabled (VT-x/AMD-V)
- At least 12GB free RAM (8GB for VM + 4GB for host)
- At least 60GB free disk space
- Internet connection for downloading Windows ISO

## Quick Start

```bash
# 1. Start libvirt daemon
sudo systemctl start libvirtd

# 2. Download Windows 11 ISO (evaluation version, free for 90 days)
./download-windows-iso.sh

# 3. Create and start the VM
./create-vm.sh

# 4. Access the VM (graphical)
virt-viewer nsfw-test-win11

# Or use VNC
vncviewer localhost:5900
```

## VM Specifications

- **Name**: nsfw-test-win11
- **OS**: Windows 11 Pro (Evaluation)
- **CPU**: 4 cores
- **RAM**: 8192 MB
- **Disk**: 60 GB (qcow2, dynamic)
- **Network**: NAT (internet access)
- **Graphics**: VGA with VNC on port 5900

## Setup Steps

### 1. Install Windows 11

When the VM starts, you'll boot into Windows 11 setup:

1. Select language and region
2. Click "Install Now"
3. Accept license terms
4. Choose "Custom: Install Windows only"
5. Select the disk and proceed
6. Create local account when prompted
7. Skip all Microsoft account/OneDrive prompts

**Tips:**
- Use a simple password for testing (you'll type it often)
- Disable all telemetry/suggestions during setup
- Skip Windows Update initially (we'll do it later)

### 2. Install WSL2

Once Windows is installed and you're at the desktop:

```powershell
# Open PowerShell as Administrator

# Enable WSL
wsl --install

# Reboot when prompted
shutdown /r /t 0

# After reboot, open PowerShell again
# Set default version to WSL2
wsl --set-default-version 2

# Install Ubuntu (default distribution)
wsl --install -d Ubuntu

# Set up Ubuntu username/password when prompted
```

### 3. Install Nix in WSL2

```bash
# Inside WSL2 Ubuntu
curl -L https://nixos.org/nix/install | sh

# Restart shell or run
source ~/.nix-profile/etc/profile.d/nix.sh

# Verify Nix works
nix --version
```

### 4. Build and Test NSFW

```bash
# Inside WSL2, clone the repository
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Build NSFW
cargo build --release

# Run tests
cargo test

# Test CLI commands
./target/release/nsfw.exe search firefox
./target/release/nsfw.exe install firefox
./target/release/nsfw.exe list
```

## File Transfer to VM

### Method 1: Shared Folder (Recommended)

```bash
# On NixOS host, create shared directory
mkdir -p vm-testing/shared

# Copy NSFW binary
cp target/release/nsfw.exe vm-testing/shared/

# In VM, access via \\<host-ip>\shared
# Or set up VirtFS mount
```

### Method 2: SCP to WSL2

```bash
# Get WSL2 IP address
wsl hostname -I

# Copy from host to WSL2
scp target/release/nsfw.exe user@<wsl2-ip>:~/
```

### Method 3: Git Clone in WSL2

```bash
# Simplest: just clone in WSL2
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw
cargo build --release
```

## VM Management

### Start VM
```bash
virsh start nsfw-test-win11
```

### Stop VM
```bash
virsh shutdown nsfw-test-win11
```

### Force Stop
```bash
virsh destroy nsfw-test-win11
```

### Delete VM
```bash
virsh undefine nsfw-test-win11
rm -rf ~/.local/share/libvirt/images/nsfw-test-win11.qcow2
```

### Snapshot Management

```bash
# Create snapshot before testing
virsh snapshot-create-as nsfw-test-win11 \
  clean-install "Fresh Windows 11 + WSL2 + Nix"

# List snapshots
virsh snapshot-list nsfw-test-win11

# Restore snapshot
virsh snapshot-revert nsfw-test-win11 clean-install

# Delete snapshot
virsh snapshot-delete nsfw-test-win11 clean-install
```

## Testing Checklist

### Phase 2 Testing Goals

- [ ] **Installation Verification**
  - [ ] Binary runs without errors
  - [ ] WSL2 detected correctly
  - [ ] Nix detected in WSL2

- [ ] **Path Translation**
  - [ ] Windows paths (C:\Users) convert correctly
  - [ ] WSL2 paths (/mnt/c) work
  - [ ] Nix store paths preserved

- [ ] **Package Operations**
  - [ ] Search works and displays results
  - [ ] Install command succeeds
  - [ ] List shows installed packages
  - [ ] Remove command works

- [ ] **Error Handling**
  - [ ] Clear error when WSL2 not installed
  - [ ] Clear error when Nix not installed
  - [ ] Network errors handled gracefully
  - [ ] Invalid package names handled

- [ ] **Performance**
  - [ ] Search response time (<5s)
  - [ ] Install progress indicators work
  - [ ] No hangs or freezes

- [ ] **User Experience**
  - [ ] Output is readable and helpful
  - [ ] Confirmation prompts work
  - [ ] Help text is clear
  - [ ] Error messages are actionable

## Troubleshooting

### VM won't start
```bash
# Check virtualization is enabled
lscpu | grep -i virtualization

# Check libvirt is running
sudo systemctl status libvirtd

# Check for errors
virsh dominfo nsfw-test-win11
```

### Can't connect to VM display
```bash
# Check VNC is running
virsh vncdisplay nsfw-test-win11

# Try virt-viewer
virt-viewer nsfw-test-win11

# Or direct VNC
vncviewer localhost:5900
```

### WSL2 won't install
- Ensure virtualization is enabled in VM settings
- Check Windows version is 11 (build 22000+)
- Run Windows Update to get latest version
- Try manual WSL2 installation: https://aka.ms/wsl2

### Slow performance
- Increase CPU cores in create-vm.sh (default 4)
- Increase RAM (default 8GB)
- Use virtio drivers for better I/O
- Enable KVM acceleration (should be automatic)

## Advanced Configuration

### Enable Nested Virtualization
If you need WSL2 inside the VM, nested virtualization must be enabled:

```bash
# For Intel CPUs
sudo modprobe -r kvm_intel
sudo modprobe kvm_intel nested=1

# For AMD CPUs
sudo modprobe -r kvm_amd
sudo modprobe kvm_amd nested=1

# Make permanent
echo "options kvm_intel nested=1" | sudo tee /etc/modprobe.d/kvm.conf
```

### Increase Performance
```bash
# Edit VM to use more resources
virsh edit nsfw-test-win11

# Change:
# <vcpu>4</vcpu> to <vcpu>8</vcpu>
# <memory>8388608</memory> to <memory>16777216</memory>
```

## Cleanup

```bash
# Delete VM and all data
virsh destroy nsfw-test-win11
virsh undefine nsfw-test-win11
rm -rf ~/.local/share/libvirt/images/nsfw-test-win11.qcow2
```

## Resources

- Windows 11 Evaluation: https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise
- WSL2 Installation: https://learn.microsoft.com/en-us/windows/wsl/install
- Nix Installation: https://nixos.org/download.html
- libvirt Documentation: https://libvirt.org/docs.html

---

**Status**: Ready for Phase 2 testing
**VM Template**: Windows 11 Pro + WSL2 + Nix
**Test Duration**: ~2 hours for full test suite
