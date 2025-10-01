# What to Copy to the Windows VM for Testing

**Status**: Claude Code installed in WSL2 ✅
**Next**: Get NSFW binary and start testing

---

## 🎯 Two Approaches

### Approach A: Build in VM (Recommended - 10 minutes)
**Why**: Most flexible, can modify code and rebuild
**Requires**: Copying source code to VM

### Approach B: Copy Binary Only (Faster - 2 minutes)
**Why**: Quick testing without build setup
**Requires**: Pre-built binary (need to cross-compile for Windows)

**Recommendation**: Use **Approach A** since you have Claude Code ready and can iterate faster

---

## 📦 Approach A: Build NSFW in VM's WSL2

### Step 1: Copy Source Code to VM

**On Host** (your NixOS machine):
```bash
# Create a tarball of the NSFW source
cd /srv/luminous-dynamics/11-meta-consciousness
tar -czf nsfw-source.tar.gz nsfw/

# Now transfer to VM - you have options:
```

**Option 1 - If VM has network access**:
```bash
# On host - start a simple HTTP server
cd /srv/luminous-dynamics/11-meta-consciousness
python3 -m http.server 8000

# On VM in WSL2:
cd ~
wget http://HOST_IP:8000/nsfw-source.tar.gz
tar -xzf nsfw-source.tar.gz
cd nsfw
```

**Option 2 - Shared folder** (if VM has shared folders):
```bash
# Copy to shared folder on host
cp nsfw-source.tar.gz /path/to/shared/folder/

# On VM in WSL2:
cd ~
cp /mnt/shared/nsfw-source.tar.gz .
tar -xzf nsfw-source.tar.gz
cd nsfw
```

**Option 3 - Manual file copy** (via VM console):
```bash
# You'll need to copy the files manually through the VM interface
# or use scp if you have SSH set up
```

### Step 2: Install Rust in WSL2

```bash
# In WSL2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow prompts (use default options)
# Then:
source ~/.cargo/env

# Verify
rustc --version
cargo --version
```

### Step 3: Build NSFW in WSL2

```bash
# In WSL2, in the nsfw directory
cd ~/nsfw

# Build release binary
cargo build --release

# This creates: target/release/nsfw (Linux binary)
```

### Step 4: Test from WSL2 First (Quick Validation)

```bash
# Still in WSL2
./target/release/nsfw --version
./target/release/nsfw search vim

# If this works, NSFW core functionality is good!
```

### Step 5: Build Windows Binary (For Windows PowerShell Testing)

```bash
# In WSL2, add Windows target
rustup target add x86_64-pc-windows-gnu

# Install mingw cross-compiler
sudo apt update
sudo apt install -y mingw-w64

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# This creates: target/x86_64-pc-windows-gnu/release/nsfw.exe
```

### Step 6: Copy to Windows

```bash
# Still in WSL2
cp target/x86_64-pc-windows-gnu/release/nsfw.exe /mnt/c/Users/YourUsername/Downloads/

# Or wherever you want it on Windows
```

### Step 7: Test from Windows PowerShell

```powershell
# In Windows PowerShell
cd ~\Downloads
.\nsfw.exe --version
.\nsfw.exe search vim
```

---

## 📦 Approach B: Copy Pre-Built Binary (If Available)

**Problem**: We built on NixOS (Linux), but need Windows binary

**Solutions**:

### Option 1: Cross-compile on Host

```bash
# On your NixOS host
cd /srv/luminous-dynamics/11-meta-consciousness/nsfw

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Install cross-compilation tools (might require adding to shell.nix)
# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Binary will be at:
# target/x86_64-pc-windows-gnu/release/nsfw.exe

# Transfer this to VM
```

### Option 2: Use GitHub Release (If Available)

```powershell
# In Windows PowerShell in VM
# Download from GitHub releases if available
# Or copy from host if you built it there
```

---

## 📋 What Files to Copy (Summary)

### Minimal (Just Binary - Approach B):
```
nsfw.exe  (Windows executable)
```

### Recommended (Source Code - Approach A):
```
nsfw/                    (entire source directory)
├── src/                 (source code)
├── tests/               (test suites)
├── benches/             (benchmarks)
├── docs/                (documentation)
├── Cargo.toml           (dependencies)
└── Cargo.lock           (locked versions)
```

### Optional (Documentation for Reference):
```
vm-testing/              (testing guides)
├── START_HERE.md
├── DAY_16_PREPARATION.md
├── TESTING_CHECKLIST.md
└── QUICK_REFERENCE.md
```

---

## 🚀 Quick Start Script

Here's a complete script for the VM setup:

```bash
#!/bin/bash
# Run this in WSL2 on the VM after copying source code

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Extract source (if you copied tarball)
tar -xzf nsfw-source.tar.gz
cd nsfw

# Build Linux binary (for WSL2 testing)
cargo build --release
echo "✓ Linux binary built"

# Test it
./target/release/nsfw --version
echo "✓ Basic functionality verified"

# Install Windows cross-compilation tools
rustup target add x86_64-pc-windows-gnu
sudo apt update
sudo apt install -y mingw-w64
echo "✓ Cross-compilation tools installed"

# Build Windows binary
cargo build --release --target x86_64-pc-windows-gnu
echo "✓ Windows binary built"

# Copy to Windows filesystem
cp target/x86_64-pc-windows-gnu/release/nsfw.exe /mnt/c/Users/$USER/Downloads/
echo "✓ Copied nsfw.exe to Downloads"

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Open Windows PowerShell"
echo "2. cd ~\\Downloads"
echo "3. .\\nsfw.exe search vim"
```

Save this as `setup-nsfw.sh` and run it!

---

## 🧪 Testing Checklist

Once you have nsfw.exe in Windows:

### Basic Functionality
```powershell
# 1. Version check
.\nsfw.exe --version

# 2. Search test
.\nsfw.exe search vim

# 3. Search with limit
.\nsfw.exe search firefox --limit 5

# 4. JSON output
.\nsfw.exe search python --format json

# 5. Install test (if you have Nix in WSL2)
.\nsfw.exe install vim --yes

# 6. List test
.\nsfw.exe list

# 7. List detailed
.\nsfw.exe list --detailed

# 8. Remove test
.\nsfw.exe remove vim --yes
```

### Performance Testing
```powershell
# Cache test
Measure-Command { .\nsfw.exe search firefox }  # First run
Measure-Command { .\nsfw.exe search firefox }  # Cached (should be <1ms)
```

### Visual Testing
- ✅ Check colored output
- ✅ Check progress spinners
- ✅ Check error messages
- ✅ Check formatting

---

## 🐛 Troubleshooting

### "nsfw.exe is not recognized"
```powershell
# Make sure you're in the right directory
cd ~\Downloads
ls  # Should see nsfw.exe

# Or use full path
C:\Users\YourUsername\Downloads\nsfw.exe search vim
```

### "cargo: command not found" in WSL2
```bash
# Rust not installed or not in PATH
source ~/.cargo/env

# Or install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build fails with "linker not found"
```bash
# In WSL2 - install build tools
sudo apt update
sudo apt install -y build-essential
```

### Windows binary doesn't work
```powershell
# Check if it's a valid executable
Get-Item .\nsfw.exe | Select-Object *

# Try running from WSL2 instead
wsl ~/nsfw/target/release/nsfw search vim
```

---

## 📊 Expected Results

### What Should Work ✅
- Version command (instant)
- Search command (2-5 seconds first time, <1ms cached)
- Colored output in Windows Terminal
- Progress spinners during operations
- JSON output format
- List command (if packages installed)

### What Might Not Work Yet ⚠️
- Install/remove (requires Nix properly set up in WSL2)
- Wrapper generation (needs testing)
- Some edge cases (that's what we're testing for!)

---

## 🎯 Recommended Testing Flow

1. **Build in WSL2** (Approach A - 10 minutes)
2. **Test Linux binary** in WSL2 (2 minutes)
3. **Build Windows binary** with cross-compilation (5 minutes)
4. **Test Windows binary** from PowerShell (5 minutes)
5. **Run through test checklist** (30 minutes)
6. **Report issues** to Claude Code in WSL2 for fixes!

---

## 💬 Using Claude Code for Help

As you test, ask Claude Code in WSL2:

```bash
# Example debugging session
claude "I ran nsfw.exe search vim from PowerShell and got error: \
       'WSL2 not available'. But wsl --version works fine. What's wrong?"

claude "The colored output isn't showing in Windows Terminal. \
       It's all plain text. How do I debug this?"

claude "Search is taking 4 seconds every time, cache doesn't seem to work. \
       Where should I look in the code?"
```

---

## ✅ Success Criteria

You'll know the setup is complete when:
- ✅ nsfw.exe runs from Windows PowerShell
- ✅ Search returns colored results
- ✅ Cached searches are instant
- ✅ Progress spinners appear during operations
- ✅ Error messages are helpful and colored

---

## 📝 Next Steps After Setup

1. **Run comprehensive tests** (TESTING_CHECKLIST.md)
2. **Document any issues** you find
3. **Measure performance** (cache, search, install times)
4. **Test edge cases** (unicode, special chars, etc.)
5. **Provide feedback** for improvements!

---

**Ready?** Let's get NSFW running on Windows! 🚀

**Start with**: Copy source code → Build in WSL2 → Test from Windows PowerShell
