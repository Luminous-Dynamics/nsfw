# Installing Claude Code in WSL2 for VM Testing

**Context**: You've just installed Windows 11 in the VM and are ready to test NSFW
**Goal**: Set up Claude Code in WSL2 for efficient development and testing

---

## 🎯 Quick Setup (5-10 minutes)

### Step 1: Open WSL2 in Your Windows VM

```powershell
# From Windows PowerShell in the VM
wsl
```

You should now be in Ubuntu (or your WSL2 distro).

### Step 2: Install Node.js (Required for Claude Code)

```bash
# Update package list
sudo apt update

# Install Node.js via NodeSource
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify installation
node --version  # Should be v20.x
npm --version   # Should be 10.x
```

### Step 3: Install Claude Code CLI

```bash
# Install Claude Code globally
npm install -g @anthropic-ai/claude-code

# Verify installation
claude --version
```

### Step 4: Authenticate Claude Code

```bash
# Start authentication
claude auth login

# This will:
# 1. Open a browser (or give you a URL)
# 2. Ask you to log in to Claude
# 3. Authorize the CLI
# 4. Return to terminal when complete
```

**Note**: If the browser doesn't open in WSL2:
1. Copy the URL shown in terminal
2. Open it in Windows browser
3. Complete authentication
4. Return to WSL2 terminal

### Step 5: Test Claude Code

```bash
# Quick test
claude "Hello, can you help me test NSFW?"

# If this works, you're ready! 🎉
```

---

## 🔄 Workflow for Testing NSFW

### From Windows Side (Testing NSFW)

```powershell
# In Windows PowerShell
cd C:\Users\YourUser\Downloads  # Where you put nsfw.exe

# Test NSFW commands
.\nsfw.exe search vim
.\nsfw.exe install vim --yes
.\nsfw.exe list
```

### From WSL2 Side (Development)

```bash
# In WSL2 terminal
cd /mnt/c/Users/YourUser/Downloads

# Ask Claude Code for help
claude "The nsfw search is slow, can you help optimize?"

# Or work on the codebase
cd ~/nsfw  # If you clone the repo
claude "Help me add Windows Registry integration"
```

---

## 🎨 Recommended Setup

### Option 1: Windows Terminal with Split Panes

1. Open Windows Terminal
2. Split pane (Alt+Shift+D)
3. Left: PowerShell (for testing NSFW)
4. Right: WSL2 (for Claude Code assistance)

### Option 2: VS Code with WSL Extension

1. Install VS Code in Windows
2. Install "Remote - WSL" extension
3. Connect to WSL2
4. Use integrated terminal for Claude Code
5. Use Windows terminal for NSFW testing

---

## 📋 Quick Reference

### Test NSFW from Windows
```powershell
# Windows PowerShell
.\nsfw.exe search firefox
.\nsfw.exe install vim --yes
.\nsfw.exe list --detailed
```

### Get Help from Claude Code in WSL2
```bash
# WSL2 terminal
claude "NSFW search takes 5 seconds, should be faster"
claude "Help me debug Windows path translation"
claude "What tests should I run for NSFW?"
```

### Share Files Between Windows and WSL2
```bash
# In WSL2, Windows C: drive is at:
/mnt/c/

# Example: Access Downloads folder
cd /mnt/c/Users/YourUser/Downloads
ls -la nsfw.exe

# Copy to WSL2 home
cp /mnt/c/Users/YourUser/Downloads/nsfw.exe ~/
```

---

## 🧪 Suggested Testing Workflow

### Phase 1: Basic Functionality (Day 16-17)

**Windows Side**:
```powershell
# Test search
.\nsfw.exe search vim
.\nsfw.exe search firefox --limit 5
.\nsfw.exe search python --format json

# Test install
.\nsfw.exe install vim --yes

# Test list
.\nsfw.exe list
.\nsfw.exe list --detailed

# Test remove
.\nsfw.exe remove vim --yes
```

**WSL2 Side** (Ask Claude):
```bash
claude "I ran 'nsfw search vim' and it took 4.2 seconds. \
       Is that expected? The code says it should be cached."

claude "Search results show garbled characters in descriptions. \
       Could this be a UTF-8 encoding issue?"
```

### Phase 2: Performance Testing (Day 18)

**Windows Side**:
```powershell
# Measure performance
Measure-Command { .\nsfw.exe search firefox }

# Test cache
.\nsfw.exe search firefox  # First run
.\nsfw.exe search firefox  # Should be instant (<1ms)
```

**WSL2 Side**:
```bash
claude "I tested cached search and it still takes 2 seconds. \
       Cache should make it <1ms. What's wrong?"

claude "Can you help me add performance logging to debug this?"
```

### Phase 3: Edge Cases (Day 18)

**Windows Side**:
```powershell
# Test edge cases from TESTING_CHECKLIST.md
.\nsfw.exe search ""                    # Empty query
.\nsfw.exe search "pkg-with-unicode-🎉"  # Unicode
.\nsfw.exe install nonexistent --yes    # Package not found
```

**WSL2 Side**:
```bash
claude "Empty search returned error 'query cannot be empty'. \
       Should we show a helpful message instead?"
```

---

## 🚀 Advanced: Clone NSFW Repo in WSL2

If you want to modify NSFW code:

```bash
# In WSL2
cd ~
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Build NSFW
cargo build --release

# Binary is now at:
# target/release/nsfw

# Copy to Windows for testing
cp target/release/nsfw /mnt/c/Users/YourUser/Downloads/nsfw.exe
```

Then test from Windows PowerShell!

---

## 🐛 Troubleshooting

### Claude Code Not Found
```bash
# Check if npm global bin is in PATH
echo $PATH

# Add to ~/.bashrc if needed
echo 'export PATH="$PATH:~/.npm-global/bin"' >> ~/.bashrc
source ~/.bashrc
```

### Authentication Issues
```bash
# Clear auth and try again
rm -rf ~/.config/claude-code
claude auth login
```

### WSL2 Can't See Windows Files
```bash
# Windows drives should be mounted at /mnt/
ls /mnt/c/Users/

# If not, check WSL2 configuration
cat /etc/wsl.conf
```

---

## 🎯 Why This Setup is Optimal

1. **Fast Iteration**
   - Modify code in WSL2
   - Build in seconds
   - Test in Windows immediately

2. **Real-Time Help**
   - Encounter issue in Windows
   - Ask Claude in WSL2
   - Get solution instantly

3. **Best of Both Worlds**
   - Windows for realistic testing
   - Linux for development tools
   - Claude Code for AI assistance

4. **No Context Switching**
   - Split screen in Windows Terminal
   - See both environments at once
   - Seamless workflow

---

## 📊 Expected Timeline

- **Setup** (this guide): 10 minutes
- **First tests**: 15 minutes
- **Full test suite**: 2-3 hours
- **Debugging with Claude**: Saves hours!

---

## ✅ Ready to Start?

Once you have Claude Code in WSL2:

1. Open Windows Terminal
2. Split pane
3. Left: `powershell` (for NSFW testing)
4. Right: `wsl` then `claude` (for development help)
5. Start testing! 🚀

---

**Next Step**: Run the first test from Windows PowerShell:
```powershell
.\nsfw.exe --version
```

If that works, you're ready for full testing! 🎉
