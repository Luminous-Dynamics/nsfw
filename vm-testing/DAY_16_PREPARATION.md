# Day 16: VM Creation and Initial Setup - Preparation Guide

**Date**: 2025-09-30
**Goal**: Get Windows 11 VM running with NSFW ready for testing
**Estimated Time**: 2-3 hours (mostly Windows installation)

## Pre-Flight Checklist ✈️

Before starting, verify you have:

- [ ] **Virtualization enabled** - `lscpu | grep -i virtualization` shows VT-x or AMD-V
- [ ] **libvirt running** - `sudo systemctl status libvirtd` is active
- [ ] **Disk space** - At least 70GB free in `~/.local/share/libvirt/images/`
- [ ] **Internet connection** - For downloading ISO and packages
- [ ] **Time available** - 2-3 hours for complete setup
- [ ] **virt-viewer installed** - `which virt-viewer` returns path
- [ ] **VNC viewer** (alternative) - `which vncviewer` returns path

### Quick Host Verification

```bash
# Run all checks at once
cd /srv/luminous-dynamics/11-meta-consciousness/nsfw/vm-testing

# Check virtualization
echo "Virtualization: $(lscpu | grep -i virtualization | awk '{print $2}')"

# Check libvirt
systemctl is-active libvirtd && echo "libvirtd: ✅ Running" || echo "libvirtd: ❌ Not running"

# Check disk space
df -h ~/.local/share/libvirt/images/ 2>/dev/null | tail -1 | awk '{print "Disk space: " $4 " available"}'

# Check virt-viewer
which virt-viewer &>/dev/null && echo "virt-viewer: ✅ Available" || echo "virt-viewer: ⚠️ Not found"

# Check VNC viewer (alternative)
which vncviewer &>/dev/null && echo "vncviewer: ✅ Available" || echo "vncviewer: ⚠️ Not found"
```

## Step-by-Step Setup Process

### Step 1: Download Windows 11 ISO (30-60 minutes)

```bash
# Run the download helper
./download-windows-iso.sh
```

**Manual Download**:
1. Visit: https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise
2. Click "Download the ISO - Enterprise"
3. Fill out form (required)
4. Select language: English (United States)
5. Download 64-bit ISO (~5GB)
6. Move to: `vm-testing/iso/Win11_EnglishInternational_x64.iso`

**Verify Download**:
```bash
# Check file exists and size is reasonable
ls -lh iso/Win11_EnglishInternational_x64.iso
# Should be ~4.5-5.5 GB
```

### Step 2: Create the VM (2 minutes)

```bash
# Ensure libvirt is running
sudo systemctl start libvirtd

# Create VM (will start automatically)
./create-vm.sh
```

**Expected Output**:
```
🖥️  NSFW Windows Testing VM Setup
==================================

✅ Found Windows 11 ISO: Win11_EnglishInternational_x64.iso
   Size: 5.1G

✅ libvirtd is running

📀 Creating disk image...
✅ Disk created: /home/user/.local/share/libvirt/images/nsfw-test-win11.qcow2

🚀 Creating VM...
Domain nsfw-test-win11 created
```

**Connect to VM**:
```bash
# Option 1: virt-viewer (GUI)
virt-viewer nsfw-test-win11

# Option 2: VNC
vncviewer localhost:5900

# Option 3: VM Management helper
./vm-manage.sh console
```

### Step 3: Install Windows 11 (30-45 minutes)

When the VM boots, you'll see Windows Setup:

#### Initial Setup
1. **Language**: English (United States)
2. **Time**: US format
3. **Keyboard**: US
4. Click **"Install Now"**

#### License & Installation Type
1. **Enter product key**: Click "I don't have a product key"
2. **Select edition**: Windows 11 Pro
3. **License terms**: Check "I accept"
4. **Installation type**: Custom: Install Windows only (advanced)

#### Disk Setup
1. Select "Drive 0 Unallocated Space"
2. Click "Next" (Windows will create partitions automatically)
3. Installation begins (15-30 minutes)
4. VM will restart automatically

#### First Boot Setup
1. **Region**: United States
2. **Keyboard layout**: US
3. **Second keyboard**: Skip
4. **Name**: `Test` or `NSFW-Test`
5. **Password**: Use simple password (e.g., `test123`)
   - You'll type this often during testing!
6. **Security questions**: Answer with simple answers
7. **Privacy settings**: Turn everything OFF (testing VM)
8. **Cortana**: Skip
9. Skip all Microsoft account prompts

**Create Snapshot After Windows Install**:
```bash
# Once at Windows desktop
./vm-manage.sh snapshot
# Name: windows-installed
# Description: Clean Windows 11 install before WSL2
```

### Step 4: Install WSL2 (10 minutes + reboot)

In Windows, open PowerShell as Administrator:
1. Click Start
2. Type "PowerShell"
3. Right-click "Windows PowerShell"
4. Select "Run as Administrator"

```powershell
# Install WSL2
wsl --install

# This will:
# - Enable WSL feature
# - Enable Virtual Machine Platform
# - Download WSL2 kernel
# - Set WSL2 as default
# - Install Ubuntu distribution

# Output should show:
# Installing: Windows Subsystem for Linux
# Installing: Ubuntu
# The requested operation is successful...
```

**Reboot**:
```powershell
shutdown /r /t 0
```

**After Reboot**:
Ubuntu terminal will open automatically:
1. **Username**: `test` (or your choice)
2. **Password**: `test` (or your choice)
3. Wait for setup to complete

**Verify WSL2**:
```powershell
wsl --version
wsl --list --verbose
# Should show: Ubuntu, Running, version 2
```

**Create Snapshot After WSL2**:
```bash
./vm-manage.sh snapshot
# Name: wsl2-installed
# Description: Windows 11 + WSL2 + Ubuntu
```

### Step 5: Install Nix in WSL2 (5 minutes)

In Ubuntu terminal (WSL2):

```bash
# Download and install Nix
curl -L https://nixos.org/nix/install | sh

# This takes 2-3 minutes
# Answer prompts with default (Enter)

# Source Nix profile
source ~/.nix-profile/etc/profile.d/nix.sh

# Add to .bashrc for future sessions
echo 'source ~/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc

# Verify Nix is working
nix --version
# Should show: nix (Nix) 2.xx.x

# Test a simple command
nix-shell -p hello --run "hello"
# Should print: Hello, world!
```

**Create Snapshot After Nix**:
```bash
./vm-manage.sh snapshot
# Name: ready-for-testing
# Description: Windows 11 + WSL2 + Ubuntu + Nix - Ready for NSFW testing
```

### Step 6: Get NSFW Binary into VM (Multiple Methods)

#### Method A: Build in WSL2 (Recommended)

In WSL2 Ubuntu:
```bash
# Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone repository
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Build
cargo build --release

# Binary is now at: target/release/nsfw.exe
```

#### Method B: Copy from Host to WSL2

On NixOS host:
```bash
# Build on host
cd /srv/luminous-dynamics/11-meta-consciousness/nsfw
cargo build --release

# Get WSL2 IP address (in Windows PowerShell)
wsl hostname -I
# Example output: 172.24.123.45

# Copy to WSL2 (from host)
scp target/release/nsfw.exe test@172.24.123.45:~/
```

#### Method C: Shared Folder (Advanced)

Configure VirtFS sharing (requires VM configuration edit).

### Step 7: First Test Run 🎯

In Windows PowerShell:
```powershell
# Navigate to NSFW binary location
# If built in WSL2:
wsl
cd ~/nsfw/target/release

# If copied:
cd ~/

# Run first test
./nsfw.exe --version
# Expected: nsfw 0.1.0 (or current version)

# Run help
./nsfw.exe --help
# Should show all commands

# First real test - search
./nsfw.exe search vim
# Should show search results!

# If successful: 🎉 READY FOR FULL TESTING!
```

### Step 8: Prepare for Testing

Create a testing workspace in Windows:
```powershell
# In Windows PowerShell
mkdir C:\nsfw-testing
cd C:\nsfw-testing

# Copy binary (if in WSL2)
wsl cp ~/nsfw/target/release/nsfw.exe /mnt/c/nsfw-testing/

# Copy testing checklist (from host or download)
# You'll fill this out as you test

# Ready to begin comprehensive testing!
```

## Troubleshooting Common Issues

### Issue: VM won't start
```bash
# Check libvirt is running
sudo systemctl start libvirtd

# Check for errors
virsh dominfo nsfw-test-win11

# Try starting manually
./vm-manage.sh start
```

### Issue: Can't connect to VM display
```bash
# Check VNC is running
./vm-manage.sh vnc

# Try direct VNC
vncviewer localhost:5900

# Or try virt-manager GUI
virt-manager
```

### Issue: WSL2 won't install
In Windows:
```powershell
# Check Windows version
winver
# Must be build 19041 or higher

# Update Windows
# Settings > Update & Security > Windows Update

# Try manual install
wsl --install --force
```

### Issue: Nix install fails in WSL2
```bash
# Ensure curl is available
sudo apt update
sudo apt install curl -y

# Try install again with verbose output
sh <(curl -L https://nixos.org/nix/install) --verbose

# Check disk space
df -h
# Need at least 2GB free
```

### Issue: Can't access internet in VM
```bash
# Check VM network
virsh domiflist nsfw-test-win11

# Restart default network
sudo virsh net-destroy default
sudo virsh net-start default

# Restart VM
./vm-manage.sh restart
```

## Expected Timeline

| Task | Estimated Time | Running Total |
|------|---------------|---------------|
| Download ISO | 30-60 min | 1 hour |
| Create VM | 2 min | 1 hour |
| Install Windows | 30-45 min | 1.5-2 hours |
| Install WSL2 | 10 min + reboot | 2 hours |
| Install Nix | 5 min | 2 hours |
| Build/Copy NSFW | 5-10 min | 2-2.5 hours |
| First Tests | 5 min | 2.5 hours |

**Total**: ~2.5-3 hours for complete setup

## Success Criteria for Day 16

- [ ] VM created and Windows 11 installed
- [ ] WSL2 installed and Ubuntu running
- [ ] Nix installed in WSL2 and working
- [ ] NSFW binary available in Windows
- [ ] `nsfw.exe --version` works
- [ ] `nsfw.exe search vim` returns results
- [ ] Snapshots created at each milestone
- [ ] Ready to begin comprehensive testing (Day 18)

## Next Steps After Day 16

Once Day 16 is complete:
1. **Day 17**: Continue initial testing, explore edge cases
2. **Day 18**: Execute all 30 tests from TESTING_CHECKLIST.md
3. Document any issues found
4. Prepare bug list for Day 19 fixes

## Quick Commands Reference

```bash
# VM Management
./vm-manage.sh start        # Start VM
./vm-manage.sh stop         # Stop VM
./vm-manage.sh status       # Check status
./vm-manage.sh console      # Open GUI
./vm-manage.sh snapshot     # Create snapshot
./vm-manage.sh restore      # Restore snapshot

# In Windows PowerShell
wsl                         # Enter WSL2
wsl hostname -I             # Get WSL2 IP
wsl --version              # Check WSL version
wsl --list --verbose       # List distributions

# In WSL2
nix --version              # Check Nix
source ~/.nix-profile/etc/profile.d/nix.sh  # Load Nix
cd ~/nsfw                  # Navigate to project
```

---

**Preparation Status**: Ready
**Next Action**: Download Windows 11 ISO and create VM
**Estimated Completion**: 2.5-3 hours

Good luck with Day 16 testing! 🚀
