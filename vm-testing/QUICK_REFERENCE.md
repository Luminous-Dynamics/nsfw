# Windows VM Testing - Quick Reference

## 🚀 Setup (One-Time)

```bash
# 1. Download ISO (manual step)
./download-windows-iso.sh

# 2. Create VM
./create-vm.sh

# 3. Install Windows 11
./vm-manage.sh console
# Follow Windows setup wizard

# 4. Install WSL2 (in Windows PowerShell as Admin)
wsl --install
# Reboot when prompted

# 5. Install Nix (in WSL2 Ubuntu)
curl -L https://nixos.org/nix/install | sh
source ~/.nix-profile/etc/profile.d/nix.sh
```

## 💻 Daily Usage

```bash
# Start VM
./vm-manage.sh start

# Open console
./vm-manage.sh console

# Check status
./vm-manage.sh status

# Stop VM
./vm-manage.sh stop
```

## 🧪 Testing NSFW

### In Windows PowerShell:

```powershell
# Basic commands
nsfw.exe --version
nsfw.exe --help

# Search packages
nsfw.exe search firefox
nsfw.exe search vim --limit 10

# Install package
nsfw.exe install hello
nsfw.exe install firefox --yes

# List installed
nsfw.exe list

# Remove package
nsfw.exe remove hello
nsfw.exe remove firefox --yes
```

### Performance Testing:

```powershell
# Measure command time
Measure-Command { nsfw.exe search firefox }
Measure-Command { nsfw.exe install hello --yes }
Measure-Command { nsfw.exe list }
```

## 📸 Snapshots

```bash
# Create snapshot before testing
./vm-manage.sh snapshot
# Name: "pre-test"
# Description: "Clean state before NSFW testing"

# Restore after testing
./vm-manage.sh restore
# Select: "pre-test"

# List all snapshots
./vm-manage.sh list
```

## 🐛 Common Issues

### VM won't start
```bash
# Check libvirt is running
sudo systemctl status libvirtd

# Start if needed
sudo systemctl start libvirtd

# Try again
./vm-manage.sh start
```

### Can't connect to console
```bash
# Check VNC port
./vm-manage.sh vnc

# Try direct VNC
vncviewer localhost:5900
```

### WSL2 not working
```powershell
# In Windows PowerShell (Admin)
wsl --status
wsl --update
wsl --version

# If needed, reinstall
wsl --install --force
```

### Nix not found in WSL2
```bash
# In WSL2
echo $PATH  # Should include .nix-profile/bin

# Re-source Nix
source ~/.nix-profile/etc/profile.d/nix.sh

# Or add to .bashrc
echo 'source ~/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc
```

## 📋 Testing Workflow

1. **Start fresh**
   ```bash
   ./vm-manage.sh restore  # Restore to clean snapshot
   ./vm-manage.sh start
   ./vm-manage.sh console
   ```

2. **Copy binary to VM**
   - Build on host: `cargo build --release`
   - Copy to VM via shared folder or SCP to WSL2

3. **Run test suite**
   - Open PowerShell in VM
   - Follow TESTING_CHECKLIST.md

4. **Document results**
   - Fill in TESTING_CHECKLIST.md
   - Note any bugs or issues
   - Record performance metrics

5. **Snapshot after bugs**
   ```bash
   ./vm-manage.sh snapshot
   # Name: "bug-reproduction"
   # Description: "State showing bug XYZ"
   ```

## 🎯 Phase 2 Goals

- [ ] All 30 tests in TESTING_CHECKLIST.md pass
- [ ] Performance meets targets (search <5s, install <60s)
- [ ] No critical bugs
- [ ] User experience is smooth
- [ ] Error messages are helpful

## 📞 Getting Help

- **VM Issues**: See README.md section "Troubleshooting"
- **NSFW Issues**: Create issue on GitHub
- **WSL2 Issues**: https://aka.ms/wsl
- **Nix Issues**: https://nixos.org/manual/nix/stable/

## 🔧 Advanced

### Increase VM Performance
```bash
# Edit VM configuration
virsh edit nsfw-test-win11

# Change to:
# <vcpu>8</vcpu>           (more CPUs)
# <memory>16777216</memory> (16GB RAM)
```

### Enable Nested Virtualization
```bash
# For Intel
sudo modprobe -r kvm_intel
sudo modprobe kvm_intel nested=1

# For AMD
sudo modprobe -r kvm_amd
sudo modprobe kvm_amd nested=1
```

### Share Folder with VM
```bash
# Add to VM config
virsh edit nsfw-test-win11

# Add in <devices>:
# <filesystem type='mount' accessmode='passthrough'>
#   <source dir='/srv/luminous-dynamics/11-meta-consciousness/nsfw/vm-testing/shared'/>
#   <target dir='nsfw-shared'/>
# </filesystem>

# In Windows, access via:
# \\localhost\nsfw-shared
```

## 📊 Quick Status Check

```bash
# One command to see everything
./vm-manage.sh status
./vm-manage.sh info
./vm-manage.sh list
```

## 🗑️ Cleanup

```bash
# Delete VM and all data
./vm-manage.sh delete

# Type 'delete' to confirm
```

---

**Need more details?** See README.md
**Ready to test?** See TESTING_CHECKLIST.md
