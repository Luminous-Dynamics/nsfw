# 🚀 Start Here - Windows VM Testing

**Phase 2 Status**: Day 16 Ready
**Goal**: Test NSFW on real Windows 11 + WSL2 + Nix

---

## ⚡ Quick Start (3 Commands)

```bash
# 1. Check if your host is ready
./check-host-ready.sh

# 2. Download Windows 11 ISO (manual step)
./download-windows-iso.sh

# 3. Create and start VM
./create-vm.sh
```

Then follow the Windows installation wizard in the VM!

---

## 📚 Documentation

| Document | Purpose | Read Time |
|----------|---------|-----------|
| **[START_HERE.md](START_HERE.md)** | This file - quick overview | 2 min |
| **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** | Daily commands & shortcuts | 5 min |
| **[DAY_16_PREPARATION.md](DAY_16_PREPARATION.md)** | Complete step-by-step guide | 15 min |
| **[DAY_16_PROGRESS.md](DAY_16_PROGRESS.md)** | Track your progress | Fill as you go |
| **[TESTING_CHECKLIST.md](TESTING_CHECKLIST.md)** | 30 comprehensive tests | For Day 18 |
| **[README.md](README.md)** | Full documentation | 30 min |

---

## 🎯 Your Path to Testing

### Right Now (10 minutes)
1. Read this file
2. Run `./check-host-ready.sh`
3. Read [DAY_16_PREPARATION.md](DAY_16_PREPARATION.md)

### Today/Tomorrow (2-3 hours)
1. Download Windows 11 ISO
2. Create VM with `./create-vm.sh`
3. Install Windows 11 (30-45 min)
4. Install WSL2 (10 min)
5. Install Nix (5 min)
6. Run first NSFW tests! 🎉

### This Week
- Day 16-17: Complete VM setup
- Day 18: Run all 30 tests
- Day 19: Fix bugs
- Day 20: Optimize performance
- Day 21: Release v0.2.0-rc

---

## 🛠️ Tools Provided

### Scripts
- `check-host-ready.sh` - Verify prerequisites
- `download-windows-iso.sh` - ISO download guide
- `create-vm.sh` - Automated VM creation
- `vm-manage.sh` - VM lifecycle management

### Documents
- Step-by-step guides
- Progress trackers
- Testing checklists
- Quick references

---

## 💡 Key Points

### What You Need
- **Time**: 2-3 hours for complete setup
- **Disk Space**: 70GB available
- **Internet**: For ISO and packages
- **Patience**: Windows installation takes 30-45 minutes

### What's Provided
- ✅ Complete setup automation
- ✅ Detailed documentation
- ✅ Progress tracking templates
- ✅ 30-test comprehensive checklist
- ✅ VM management tools
- ✅ Troubleshooting guides

### What You'll Test
- ✅ Package search and installation
- ✅ Path translation (Windows ↔ WSL2)
- ✅ Error handling and messages
- ✅ Performance benchmarks
- ✅ User experience
- ✅ Edge cases and stress tests

---

## 🎬 Getting Started

### Step 1: Verify Host (2 minutes)

```bash
./check-host-ready.sh
```

**Expected Output**:
```
🎉 HOST IS READY FOR VM TESTING!

Next steps:
1. cd vm-testing
2. ./download-windows-iso.sh
3. ./create-vm.sh
```

**If Not Ready**: Follow the instructions to fix issues.

### Step 2: Download ISO (30-60 minutes)

```bash
./download-windows-iso.sh
```

This will guide you to download Windows 11 Evaluation (free for 90 days) from Microsoft.

**Manual Steps**:
1. Visit the URL shown
2. Fill out form
3. Download ISO (~5GB)
4. Place in `iso/` directory

### Step 3: Create VM (2 minutes)

```bash
./create-vm.sh
```

This creates a Windows 11 VM optimized for NSFW testing:
- 8GB RAM
- 4 CPU cores
- 60GB disk
- VNC access on port 5900

### Step 4: Follow the Guide

Open [DAY_16_PREPARATION.md](DAY_16_PREPARATION.md) and follow the complete walkthrough:

1. Install Windows 11 (30-45 min)
2. Install WSL2 (10 min + reboot)
3. Install Nix (5 min)
4. Get NSFW binary (5-10 min)
5. Run first tests (5 min)

**Track Progress**: Fill in [DAY_16_PROGRESS.md](DAY_16_PROGRESS.md) as you go.

---

## 🚨 Common Issues

### "Virtualization not enabled"
- Enable VT-x (Intel) or AMD-V (AMD) in BIOS
- Check with: `lscpu | grep -i virtualization`

### "libvirtd not running"
```bash
sudo systemctl start libvirtd
sudo systemctl enable libvirtd
```

### "Not enough disk space"
- Need at least 70GB free
- Check with: `df -h ~/.local/share/libvirt/images/`
- Clean up old VMs if needed

### "Can't connect to VM"
```bash
# Try VNC directly
vncviewer localhost:5900

# Or check VM status
./vm-manage.sh status
```

### "WSL2 won't install"
- Windows must be build 19041 or higher
- Run Windows Update
- Try: `wsl --install --force`

---

## 📊 Success Metrics

After Day 16, you should have:
- ✅ Windows 11 VM running
- ✅ WSL2 installed and working
- ✅ Nix installed in WSL2
- ✅ NSFW binary available
- ✅ Basic tests passing
- ✅ 3+ snapshots for testing

---

## 🎯 Next Steps

### After Day 16 Setup
1. **Day 17**: Explore NSFW features, run additional tests
2. **Day 18**: Execute all 30 comprehensive tests
3. **Day 19**: Fix critical bugs found
4. **Day 20**: Optimize performance
5. **Day 21**: Polish and release v0.2.0-rc

### Need Help?
- Check [README.md](README.md) for detailed info
- Read [QUICK_REFERENCE.md](QUICK_REFERENCE.md) for commands
- See [DAY_16_PREPARATION.md](DAY_16_PREPARATION.md) troubleshooting section

---

## 🎉 You're Ready!

Everything is prepared for successful Windows testing:

✅ **Documentation**: Complete guides and references
✅ **Automation**: Scripts for all major tasks
✅ **Tracking**: Progress and test templates
✅ **Support**: Troubleshooting and help

**Time Investment**: 2-3 hours today → Production-ready NSFW!

**Let's make NSFW work flawlessly on Windows!** 🚀

---

**Quick Commands Reminder**:
```bash
./check-host-ready.sh       # Verify prerequisites
./download-windows-iso.sh   # ISO download guide
./create-vm.sh              # Create VM
./vm-manage.sh start        # Start VM
./vm-manage.sh console      # Open VM window
./vm-manage.sh snapshot     # Create snapshot
```

**Start with**: `./check-host-ready.sh`
