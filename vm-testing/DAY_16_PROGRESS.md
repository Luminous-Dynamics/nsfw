# Day 16: VM Creation and Initial Setup - Progress Tracking

**Date Started**: _____________
**Tester**: _____________
**Goal**: Windows 11 VM with NSFW ready for testing

---

## Pre-Flight Checks

Run: `./check-host-ready.sh`

- [ ] Virtualization enabled
- [ ] libvirt running
- [ ] Disk space available (70GB+)
- [ ] virt-viewer or vncviewer available
- [ ] Internet connection working
- [ ] User in libvirt/kvm group (if needed)

**Check Results**: _________ (✅ Ready / ⚠️ Warnings / ❌ Not ready)

---

## Step 1: Download Windows 11 ISO

**Start Time**: _____________

```bash
./download-windows-iso.sh
```

**Download Method Used**: _________ (Manual / Other)

**Download Source**: https://www.microsoft.com/en-us/evalcenter/evaluate-windows-11-enterprise

**ISO Location**: `vm-testing/iso/Win11_EnglishInternational_x64.iso`

**ISO Size**: _________ GB (Expected: ~5GB)

**MD5/SHA256** (optional): _________

**Completion Time**: _____________
**Duration**: _________ minutes

- [ ] ISO downloaded successfully
- [ ] File size is reasonable (~5GB)
- [ ] File can be read (not corrupted)

---

## Step 2: Create VM

**Start Time**: _____________

```bash
sudo systemctl start libvirtd  # If needed
./create-vm.sh
```

**VM Name**: nsfw-test-win11
**Disk Location**: `~/.local/share/libvirt/images/nsfw-test-win11.qcow2`
**Disk Size**: _________ (Expected: 60GB)

**Creation Status**: _________

**Connection Method Used**: _________ (virt-viewer / vncviewer / virt-manager)

**Connection Command**: _________

**Completion Time**: _____________
**Duration**: _________ minutes

- [ ] VM created successfully
- [ ] Can connect to VM display
- [ ] Windows installer boots

---

## Step 3: Install Windows 11

**Start Time**: _____________

### Installation Progress

- [ ] Language selection (English US)
- [ ] "Install Now" clicked
- [ ] Skipped product key
- [ ] Selected Windows 11 Pro
- [ ] Accepted license terms
- [ ] Selected "Custom: Install Windows only"
- [ ] Selected Drive 0 and clicked Next
- [ ] Installation in progress... (15-30 min)
- [ ] First reboot completed
- [ ] Reached OOBE (Out of Box Experience)

### First Boot Configuration

- [ ] Region: United States
- [ ] Keyboard: US
- [ ] Computer name: _________
- [ ] Password: _________ (Write this down!)
- [ ] Security questions answered
- [ ] Privacy settings: All turned OFF
- [ ] Skipped Cortana
- [ ] Skipped Microsoft account
- [ ] Reached Windows desktop

**Completion Time**: _____________
**Duration**: _________ minutes

**Windows Version**: _________ (Run `winver` to check)

### Snapshot: windows-installed

```bash
./vm-manage.sh snapshot
```

- [ ] Snapshot created
- **Snapshot Name**: windows-installed
- **Description**: Clean Windows 11 before WSL2

---

## Step 4: Install WSL2

**Start Time**: _____________

### In Windows PowerShell (Admin)

```powershell
wsl --install
```

**Output**: _________

**Installation Status**: _________

### Reboot

```powershell
shutdown /r /t 0
```

**Reboot Time**: _____________

### After Reboot: Ubuntu Setup

- [ ] Ubuntu terminal opened automatically
- [ ] Created user: _________
- [ ] Created password: _________
- [ ] Setup completed

### Verification

```powershell
wsl --version
wsl --list --verbose
```

**WSL Version**: _________
**Distribution**: _________ (Expected: Ubuntu)
**Version**: _________ (Expected: 2)

**Completion Time**: _____________
**Duration**: _________ minutes (including reboot)

- [ ] WSL2 installed successfully
- [ ] Ubuntu distribution installed
- [ ] Can enter WSL2 with `wsl` command
- [ ] WSL2 version confirmed

### Snapshot: wsl2-installed

```bash
./vm-manage.sh snapshot
```

- [ ] Snapshot created
- **Snapshot Name**: wsl2-installed
- **Description**: Windows 11 + WSL2 + Ubuntu

---

## Step 5: Install Nix in WSL2

**Start Time**: _____________

### In WSL2 Ubuntu Terminal

```bash
curl -L https://nixos.org/nix/install | sh
```

**Installation Progress**:
- [ ] Download started
- [ ] Installation script running
- [ ] User prompts answered (defaults)
- [ ] Installation completed

**Post-Install**:
```bash
source ~/.nix-profile/etc/profile.d/nix.sh
echo 'source ~/.nix-profile/etc/profile.d/nix.sh' >> ~/.bashrc
```

### Verification

```bash
nix --version
nix-shell -p hello --run "hello"
```

**Nix Version**: _________
**Hello Test**: _________ (Should print "Hello, world!")

**Completion Time**: _____________
**Duration**: _________ minutes

- [ ] Nix installed successfully
- [ ] Nix version shown
- [ ] Test command works
- [ ] Nix profile added to .bashrc

### Snapshot: ready-for-testing

```bash
./vm-manage.sh snapshot
```

- [ ] Snapshot created
- **Snapshot Name**: ready-for-testing
- **Description**: Windows 11 + WSL2 + Ubuntu + Nix

---

## Step 6: Get NSFW Binary

**Start Time**: _____________

**Method Used**: _________ (Build in WSL2 / Copy from host / Other)

### Method A: Build in WSL2 (if chosen)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Clone
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Build
cargo build --release
```

**Build Status**: _________
**Build Time**: _________ minutes
**Binary Location**: `~/nsfw/target/release/nsfw.exe`

### Method B: Copy from Host (if chosen)

**Host Build**:
```bash
# On NixOS host
cargo build --release
```

**Copy Method**: _________

**WSL2 IP**: _________

**Copy Command**: _________

**Binary Location**: _________

### Method C: Other (if chosen)

**Description**: _________

**Binary Location**: _________

**Completion Time**: _____________
**Duration**: _________ minutes

- [ ] NSFW binary obtained
- [ ] Binary is executable
- [ ] Binary location known

---

## Step 7: First Test Run 🎯

**Start Time**: _____________

### Test 1: Version

```powershell
wsl
cd <binary-location>
./nsfw.exe --version
```

**Output**: _________
**Expected**: nsfw 0.1.0 (or current version)

- [ ] Version command works
- [ ] No errors

### Test 2: Help

```powershell
./nsfw.exe --help
```

**Output**: _________

- [ ] Help text displays
- [ ] All commands listed
- [ ] No errors

### Test 3: First Real Test - Search

```powershell
./nsfw.exe search vim
```

**Start Time**: _____________
**End Time**: _____________
**Duration**: _________ seconds

**Output**: _________

- [ ] Search executes
- [ ] Results displayed
- [ ] No errors
- [ ] Time is reasonable (<10s)

### Test 4: Quick Install Test

```powershell
./nsfw.exe install hello --yes
```

**Start Time**: _____________
**End Time**: _____________
**Duration**: _________ seconds

**Output**: _________

- [ ] Install works
- [ ] Package installed
- [ ] No errors

### Test 5: List Installed

```powershell
./nsfw.exe list
```

**Output**: _________

- [ ] Shows installed packages
- [ ] Includes 'hello' from previous test
- [ ] No errors

### Test 6: Remove Test

```powershell
./nsfw.exe remove hello --yes
```

**Output**: _________

- [ ] Remove works
- [ ] Package removed
- [ ] No errors

**Completion Time**: _____________

- [ ] All basic tests pass
- [ ] NSFW is functional
- [ ] Ready for comprehensive testing

---

## Day 16 Summary

**Total Time**: _________ hours

### Completed Steps

- [ ] ISO downloaded
- [ ] VM created
- [ ] Windows 11 installed
- [ ] WSL2 installed
- [ ] Nix installed
- [ ] NSFW binary ready
- [ ] Basic tests passing

### Issues Encountered

1. _________
2. _________
3. _________

### Solutions Applied

1. _________
2. _________
3. _________

### Snapshots Created

- [ ] `windows-installed` - Clean Windows 11
- [ ] `wsl2-installed` - Windows + WSL2
- [ ] `ready-for-testing` - Complete stack

### Performance Observations

- Windows install time: _________ min
- WSL2 install time: _________ min
- Nix install time: _________ min
- First search time: _________ sec
- First install time: _________ sec

### Next Steps for Day 17

1. _________
2. _________
3. _________

---

## Day 16 Status

**Overall**: _________ (✅ Complete / 🔄 In Progress / ❌ Blocked)

**Ready for Day 17**: _________ (Yes / No / Partial)

**Notes**: _________

---

**Completed By**: _____________
**Date Completed**: _____________
**Total Duration**: _________ hours
