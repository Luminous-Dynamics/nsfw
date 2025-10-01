# Windows Testing Guide for NSFW

This guide helps you test NSFW from Windows PowerShell after setting up WSL2 and Nix.

## Prerequisites Check

Before testing NSFW, ensure WSL2 and Nix are properly configured:

### 1. WSL2 Setup

From PowerShell:
```powershell
# Check WSL2 is installed
wsl --version

# Check your default distribution
wsl --list --verbose
```

### 2. Nix Setup in WSL2

Open a WSL2 terminal and run our setup script:

```bash
cd ~/nsfw
./setup-nix-wsl2.sh
```

This will automatically:
- Add you to the `nix-users` group
- Verify the Nix daemon is running
- Configure Nix channels
- Test that everything works

**Important**: If the script adds you to the group, you may need to restart WSL2:

```powershell
# From PowerShell
wsl --shutdown
wsl
```

## Testing NSFW from Windows

### Test 1: Basic Commands

```powershell
cd C:\Users\YOUR_USERNAME\nsfw-test

# Check version
.\nsfw.exe --version

# Show help
.\nsfw.exe --help
```

**Expected Output:**
- Version: `nsfw 0.1.0`
- Help text with all available commands

### Test 2: Search Command

```powershell
# Basic search
.\nsfw.exe search hello --limit 5

# With verbose logging
.\nsfw.exe --verbose search firefox --limit 3
```

**Expected Output:**
```
Searching for 'hello'
─────────────────────
✓ Found 5 result(s)

1. hello
   Version: 2.12.1
   Description: A program that produces a familiar, friendly greeting

...
```

**First Run Note**: The first search may take 30-60 seconds as Nix builds its package database cache. Subsequent searches will be much faster.

### Test 3: Cache Performance

Test that caching works (second search should be instant):

```powershell
# First search (may take 30-60 seconds)
Measure-Command { .\nsfw.exe search python --limit 10 }

# Second search (should be < 1 second)
Measure-Command { .\nsfw.exe search python --limit 10 }
```

**Expected:**
- First run: 30-60 seconds
- Second run: < 1 second (cache hit)

### Test 4: List Installed Packages

```powershell
# List installed packages
.\nsfw.exe list

# List with details
.\nsfw.exe list --detailed

# JSON output
.\nsfw.exe list --format json
```

**Expected:**
If you haven't installed anything yet, you'll see:
```
Installed Packages
──────────────────
ℹ No packages installed
```

### Test 5: Different Package Searches

```powershell
.\nsfw.exe search git --limit 5
.\nsfw.exe search vim --limit 5
.\nsfw.exe search rust --limit 5
```

### Test 6: Error Handling

Test that error messages are helpful:

```powershell
# Empty search
.\nsfw.exe search "" --limit 5

# Package that doesn't exist
.\nsfw.exe search thispackagedefinitelydoesnotexist123 --limit 5
```

## Expected Performance Metrics

Based on our benchmarks:

- **WSL2 Connection**: 2-3 seconds
- **First Nix Search**: 30-60 seconds (one-time database build)
- **Cached Search**: < 1 second
- **List Command**: 2-5 seconds

## Troubleshooting

### "WSL2 is not available"

If you see this error, WSL2 isn't installed or not running.

**Solution:**
```powershell
# Install WSL2
wsl --install

# Restart computer
# Then verify:
wsl --version
```

### "Permission denied" on /nix/var/nix/daemon-socket/socket

Your WSL2 user isn't in the `nix-users` group.

**Solution:**
```bash
# In WSL2
sudo usermod -a -G nix-users $USER
newgrp nix-users  # or log out and back in
```

### Nix search hangs or times out

Channels aren't configured or updated.

**Solution:**
```bash
# In WSL2
nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixpkgs
nix-channel --update
```

Or just run our setup script:
```bash
cd ~/nsfw
./setup-nix-wsl2.sh
```

### "Nix not found"

Nix isn't installed in your WSL2 distribution.

**Solution:**
```bash
# In WSL2
curl -L https://nixos.org/nix/install | sh -s -- --daemon
source /etc/profile.d/nix.sh
```

## What Success Looks Like

When everything is working correctly, you should see:

✅ **Clean UI**
- Colored output with icons (✓, ✗, ℹ, ⚠)
- Properly formatted sections with borders
- Clear, readable text

✅ **Fast Performance**
- Initial search: reasonable wait (30-60s first time)
- Cached searches: instant (< 1s)
- Commands respond quickly

✅ **Helpful Errors**
- Clear error messages
- Suggestions for fixing problems
- No cryptic stack traces

✅ **Functional Features**
- Search returns relevant results
- List shows installed packages
- Cache improves performance
- Verbose mode shows detailed logs

## Reporting Issues

If you encounter problems:

1. Run with verbose logging:
   ```powershell
   .\nsfw.exe --verbose search test --limit 3
   ```

2. Check the error message and suggestions

3. Verify WSL2 and Nix setup:
   ```bash
   # In WSL2
   ~/nsfw/setup-nix-wsl2.sh
   ```

4. If still having issues, report at:
   https://github.com/Luminous-Dynamics/nsfw/issues

Include:
- Error message
- Output from `wsl --version`
- Output from `nix --version` (in WSL2)
- Steps to reproduce

## Success Criteria

The testing is successful if:

- ✅ All basic commands work (--version, --help)
- ✅ Search returns results
- ✅ Cache improves performance on repeat searches
- ✅ UI is clean with colors and icons
- ✅ Error messages are helpful
- ✅ No crashes or hangs

## Next Steps After Testing

Once testing is successful:

1. **Try installing a package**:
   ```powershell
   .\nsfw.exe install hello
   .\nsfw.exe list
   ```

2. **Test different search queries**:
   ```powershell
   .\nsfw.exe search python
   .\nsfw.exe search nodejs
   .\nsfw.exe search docker
   ```

3. **Explore advanced features**:
   ```powershell
   .\nsfw.exe --help
   ```

Happy testing! 🚀
