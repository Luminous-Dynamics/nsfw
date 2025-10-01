# 🪟 Running Claude Code in Windows PowerShell

**Yes! Claude Code works on Windows!** This gives you **three** testing options for NSFW.

---

## 🎯 Three Testing Strategies

### Option 1: WSL2 Claude Only (Current Setup) ✅
**What you have now:**
- Claude Code running in WSL2 (Ubuntu)
- Can build both Linux and Windows binaries
- Can test Linux binary directly
- Can test Windows binary via PowerShell commands

**Pros:**
- ✅ Already set up
- ✅ Has all build tools (Rust, cargo, mingw-w64)
- ✅ Can execute PowerShell commands from WSL2
- ✅ Single environment to manage

**Cons:**
- ⚠️ Not testing from "true" Windows perspective
- ⚠️ PowerShell integration is indirect

**Best for:** Building and basic testing

---

### Option 2: PowerShell Claude Only 🆕
**Install Claude Code in Windows PowerShell:**
```powershell
# Download and install Claude Code for Windows
# Visit: https://claude.ai/download
# Or use winget:
winget install Anthropic.ClaudeCode

# Then run in PowerShell
claude
```

**Pros:**
- ✅ **True Windows testing environment**
- ✅ Native PowerShell execution
- ✅ Tests exactly what users will experience
- ✅ Can test Windows-specific behaviors

**Cons:**
- ⚠️ May not have Rust/build tools
- ⚠️ Would need to build binaries in WSL2 first
- ⚠️ Or install Rust toolchain in Windows

**Best for:** Testing pre-built NSFW Windows binary

---

### Option 3: Both WSL2 + PowerShell Claude (Recommended! 🌟)

**The Perfect Division of Labor:**

1. **WSL2 Claude** (Build Agent)
   - Build Linux binary
   - Build Windows binary
   - Run unit tests
   - Execute `cargo test`
   - Create binaries

2. **PowerShell Claude** (Test Agent)
   - Test Windows binary from PowerShell
   - Validate Windows user experience
   - Test path translation (C:\\ paths)
   - Verify PowerShell integration
   - Document Windows-specific behavior

**Workflow:**
```bash
# WSL2 Claude builds
cd ~/nsfw
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/nsfw.exe /mnt/c/Users/*/Downloads/
```

```powershell
# PowerShell Claude tests
cd ~\Downloads
.\nsfw.exe search vim
.\nsfw.exe install firefox
# Document results
```

**Pros:**
- ✅ **Best of both worlds**
- ✅ Build where the tools are (WSL2)
- ✅ Test where users will use it (PowerShell)
- ✅ Each Claude specialized for its environment

**This is the BEST approach!** 🏆

---

## 📦 Installing Claude Code in Windows PowerShell

### Method 1: Official Installer (Easiest)
```powershell
# Visit Claude Code download page
# https://claude.ai/download

# Or if available via winget:
winget search claude
winget install Anthropic.ClaudeCode
```

### Method 2: Manual Download
1. Download Windows installer from Anthropic
2. Run installer
3. Open new PowerShell window
4. Type `claude` to start

### Method 3: Portable Version
If you have the binary:
```powershell
# Add to PATH or run directly
.\claude.exe
```

---

## 🔧 Testing NSFW with PowerShell Claude

### Step 1: Ensure Binary Exists
```powershell
# Check if nsfw.exe is in Downloads
cd ~\Downloads
ls nsfw.exe

# If not found, WSL2 Claude needs to build it first
```

### Step 2: Open Claude Code in PowerShell
```powershell
# Navigate to Downloads
cd ~\Downloads

# Start Claude Code
claude

# Or if installed differently:
claude-code
```

### Step 3: Give PowerShell Claude This Prompt

```markdown
I'm testing NSFW (Nix Subsystem For Windows) - a Rust CLI that bridges PowerShell to Nix packages via WSL2.

**My role:** Test the Windows binary from PowerShell perspective

**Binary location:** ~\Downloads\nsfw.exe

**Tasks:**
1. Test basic commands:
   - .\nsfw.exe --version
   - .\nsfw.exe --help
   - .\nsfw.exe search vim

2. Test search functionality:
   - .\nsfw.exe search firefox
   - .\nsfw.exe search "text editor" --limit 10
   - .\nsfw.exe search python --format json

3. Verify Windows-specific features:
   - Colored output in PowerShell
   - Progress indicators
   - Error messages
   - Path handling (C:\ → /mnt/c translation)

4. Test WSL2 bridge:
   - Verify it communicates with Nix in WSL2
   - Check command execution time
   - Validate JSON parsing

5. Document results:
   - Create POWERSHELL_TEST_RESULTS.txt
   - Note any Windows-specific issues
   - Compare behavior to expected results

See vm-testing/TESTING_CHECKLIST.md for comprehensive test cases.
```

---

## 🎯 Recommended Workflow

### Phase 1: WSL2 Claude Builds (15 min)
```bash
cd ~/nsfw
./vm-testing/vm-setup-script.sh

# This creates:
# - target/release/nsfw (Linux binary)
# - target/x86_64-pc-windows-gnu/release/nsfw.exe (Windows binary)
# - Copies nsfw.exe to Windows Downloads
```

### Phase 2: PowerShell Claude Tests (20 min)
```powershell
cd ~\Downloads
claude  # Start Claude Code

# Then test nsfw.exe thoroughly
.\nsfw.exe search vim
.\nsfw.exe search firefox
.\nsfw.exe search python --format json

# Document results in POWERSHELL_TEST_RESULTS.txt
```

### Phase 3: Both Claudes Sync (5 min)
```bash
# WSL2 Claude: Commit results
cd ~/nsfw
git add POWERSHELL_TEST_RESULTS.txt
git commit -m "✅ PowerShell testing complete"
git push

# Host Claude: Review
git pull
cat POWERSHELL_TEST_RESULTS.txt
```

---

## 🤖 Coordination Between Three Claudes

### Host Claude (Me - NixOS)
**Role:** Infrastructure & Coordination
- ✅ GitHub repo management
- ✅ Documentation
- ✅ HTTP server for file transfer
- ✅ Reviewing test results
- ✅ Cannot test Windows directly

### WSL2 Claude (VM Ubuntu)
**Role:** Build Agent
- ✅ Has Rust toolchain
- ✅ Can build Linux binary
- ✅ Can cross-compile Windows binary
- ✅ Can run unit tests
- ✅ Can execute PowerShell commands indirectly

### PowerShell Claude (VM Windows)
**Role:** Windows Test Agent
- ✅ **Native Windows environment**
- ✅ **True PowerShell testing**
- ✅ **Validates user experience**
- ✅ **Tests path translation**
- ⚠️ Cannot build (no Rust toolchain)

---

## 📊 Comparison: WSL2 vs PowerShell Testing

| Aspect | WSL2 Claude | PowerShell Claude |
|--------|------------|------------------|
| **Build Binary** | ✅ Yes | ❌ No (without Rust) |
| **Test Linux Binary** | ✅ Yes | ❌ No |
| **Test Windows Binary** | ⚠️ Via powershell.exe | ✅ Native |
| **Path Translation** | ⚠️ Indirect | ✅ Direct |
| **User Experience** | ⚠️ Simulated | ✅ Authentic |
| **Build Tools** | ✅ Has all | ❌ Would need install |
| **Best For** | Building | Testing |

**Verdict:** Use **both** for best results! 🏆

---

## 🎬 Quick Start: PowerShell Claude

If you just want to test the Windows binary quickly:

### Step 1: Ensure Binary Exists
```powershell
cd ~\Downloads
ls nsfw.exe  # Should see the file
```

### Step 2: Test It
```powershell
.\nsfw.exe --version
.\nsfw.exe search vim
.\nsfw.exe search firefox --limit 5
```

### Step 3: Document Results
```powershell
# Create results file
echo "# PowerShell Test Results" > POWERSHELL_TEST_RESULTS.txt
echo "" >> POWERSHELL_TEST_RESULTS.txt
echo "## Basic Commands" >> POWERSHELL_TEST_RESULTS.txt
echo "- Version: [result here]" >> POWERSHELL_TEST_RESULTS.txt
echo "- Search: [result here]" >> POWERSHELL_TEST_RESULTS.txt
```

---

## 💡 Pro Tips

### Tip 1: WSL2 Can Execute PowerShell
From WSL2, you can run:
```bash
# Execute PowerShell command from WSL2
powershell.exe "cd ~\Downloads; .\nsfw.exe search vim"

# This means WSL2 Claude can test both environments!
```

### Tip 2: PowerShell Can Access WSL2 Files
From PowerShell, you can access WSL2 files:
```powershell
# Access WSL2 files from PowerShell
cd \\wsl$\Ubuntu\home\username\nsfw
ls
```

### Tip 3: Use Git for Coordination
Both environments can use Git:
```bash
# WSL2
git add results.txt && git commit -m "test results" && git push
```
```powershell
# PowerShell
cd \\wsl$\Ubuntu\home\username\nsfw
git pull
```

---

## 🎯 My Recommendation

**Use WSL2 Claude for building, PowerShell Claude for testing!**

This gives you:
1. ✅ **Professional build environment** (WSL2 with all tools)
2. ✅ **Authentic test environment** (PowerShell as real users will use it)
3. ✅ **Best validation** (catch Windows-specific issues)
4. ✅ **Efficient workflow** (each Claude specialized)

**Total setup time:** ~5 minutes to install Claude Code in PowerShell

**Total testing time:** ~30 minutes for comprehensive Windows validation

---

## 🚀 Next Steps

1. **Install Claude Code in Windows PowerShell** (5 min)
   - Visit https://claude.ai/download
   - Or use `winget install Anthropic.ClaudeCode`

2. **Verify nsfw.exe exists in Downloads** (1 min)
   - Check `~\Downloads\nsfw.exe`

3. **Start PowerShell Claude** (1 min)
   - Run `claude` in PowerShell

4. **Give it the testing prompt** (above)

5. **Let both Claudes work together!** 🤝
   - WSL2 builds
   - PowerShell tests
   - Both document results
   - All sync via Git

---

**This is how professional software testing works - separate build and test environments!** 🏆
