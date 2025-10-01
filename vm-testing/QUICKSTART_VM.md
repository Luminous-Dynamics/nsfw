# 🚀 VM Testing Quick Start

**Status**: ✅ Claude Code installed in WSL2
**Goal**: Get NSFW running in 15 minutes

---

## Step 1: Copy Files to VM (2 methods)

### Method A: Copy Entire Directory (Recommended)

**What to copy**:
```
/srv/luminous-dynamics/11-meta-consciousness/nsfw/
```

**Where to copy it**:
- VM's shared folder, OR
- Transfer via network, OR
- USB drive if you can mount it

**In the VM (Windows)**:
- Put it anywhere accessible (Desktop, Downloads, etc.)

### Method B: Minimal Files Only

If you can't copy the whole directory, copy just these:

**Essential**:
```
nsfw/
├── src/           (entire folder)
├── Cargo.toml
├── Cargo.lock
└── vm-testing/
    └── vm-setup-script.sh
```

---

## Step 2: Open WSL2 in Windows VM

```powershell
# In Windows PowerShell
wsl
```

You should now be in Ubuntu/WSL2.

---

## Step 3: Navigate to NSFW Directory

```bash
# If you copied to Windows Desktop:
cd /mnt/c/Users/YourUsername/Desktop/nsfw

# If you copied to Downloads:
cd /mnt/c/Users/YourUsername/Downloads/nsfw

# Verify you're in the right place:
ls Cargo.toml  # Should exist
```

---

## Step 4: Run Setup Script

```bash
# Make script executable
chmod +x vm-testing/vm-setup-script.sh

# Run it
./vm-testing/vm-setup-script.sh
```

**This will**:
1. Install Rust (if needed)
2. Install build tools
3. Build Linux binary
4. Build Windows binary
5. Copy nsfw.exe to Downloads
6. Take ~10 minutes on first run

**Just sit back and watch!** ☕

---

## Step 5: Test from Windows PowerShell

```powershell
# Open Windows PowerShell (new window)
cd ~\Downloads

# Test version
.\nsfw.exe --version

# Test search (should see beautiful colors!)
.\nsfw.exe search vim

# Test cache (run again - should be instant!)
.\nsfw.exe search vim
```

**If you see colored output and the second search is instant** → ✅ **Success!**

---

## 🐛 Troubleshooting

### "No such file or directory"
```bash
# Wrong directory. Find it:
find /mnt/c/Users -name "Cargo.toml" 2>/dev/null | grep nsfw
cd <path-from-above>
```

### "Permission denied"
```bash
chmod +x vm-testing/vm-setup-script.sh
```

### Build fails
```bash
# Make sure you have internet in WSL2
ping google.com

# Update packages
sudo apt update
sudo apt upgrade -y

# Try again
./vm-testing/vm-setup-script.sh
```

### Can't find nsfw.exe after build
```bash
# It's here:
ls target/x86_64-pc-windows-gnu/release/nsfw.exe

# Copy manually:
WINDOWS_USER=$(cmd.exe /c "echo %USERNAME%" 2>/dev/null | tr -d '\r')
cp target/x86_64-pc-windows-gnu/release/nsfw.exe /mnt/c/Users/$WINDOWS_USER/Downloads/
```

---

## 🎯 What You Should See

### In WSL2 (during build):
```
📦 Step 1: Installing Rust toolchain...
✓ Rust installed

🔧 Step 2: Installing build tools...
✓ Build tools installed

...

✅ SETUP COMPLETE!
```

### In Windows PowerShell (during test):
```
PS> .\nsfw.exe search vim

Searching for 'vim'
─────────────────────────

✓ Found 5 result(s)

1. vim
   Version: 9.0.2116
   Description: The most popular clone of the VI editor

...
```

**With colors!** Green package names, yellow versions, etc.

---

## 💡 Next Steps

Once basic testing works:

1. **Follow TESTING_CHECKLIST.md** for comprehensive tests
2. **Measure performance** with `Measure-Command`
3. **Test edge cases** (unicode, errors, etc.)
4. **Report issues** to Claude Code:
   ```bash
   # In WSL2
   claude "nsfw.exe search is slow, cache not working, help debug"
   ```

---

## ⏱️ Timeline

- **Copy files**: 2-5 minutes
- **Run setup script**: 10-15 minutes (first time)
- **Basic testing**: 5 minutes
- **Total**: ~20 minutes to first successful test

---

## 🎉 Success Criteria

You're ready for full testing when:
- ✅ `nsfw.exe --version` works from PowerShell
- ✅ Search shows colored output
- ✅ Second search is instant (cached)
- ✅ Progress spinners appear
- ✅ No errors

---

## 📞 Get Help

If anything goes wrong, ask Claude Code in WSL2:

```bash
claude "Setup script failed at step 4 with error: <paste error here>"
```

I'm here to help debug! 🤖

---

**Ready? Start with Step 1!** 🚀
