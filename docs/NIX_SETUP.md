# Nix Setup for NSFW

This guide helps you set up Nix correctly in WSL2 for use with NSFW.

## Quick Fix

If you're getting permission errors like:
```
error: getting status of /nix/var/nix/daemon-socket/socket: Permission denied
```

**Solution**: Add your WSL2 user to the `nix-users` group:

```bash
# In WSL2 (as root or with sudo)
sudo usermod -a -G nix-users $USER

# Log out and log back in, or run:
newgrp nix-users
```

## Fresh Nix Installation

If you haven't installed Nix yet, here's the recommended setup:

### Option 1: Multi-User Installation (Recommended)

```bash
# Install Nix with daemon mode
curl -L https://nixos.org/nix/install | sh -s -- --daemon

# Add your user to nix-users group (usually done automatically)
sudo usermod -a -G nix-users $USER

# Verify the daemon is running
ps aux | grep nix-daemon

# Source the Nix profile
source /etc/profile.d/nix.sh
```

### Option 2: Single-User Installation (Simpler)

```bash
# Install Nix in single-user mode (no daemon)
curl -L https://nixos.org/nix/install | sh -s -- --no-daemon

# Source the Nix profile
source ~/.nix-profile/etc/profile.d/nix.sh
```

## Verifying Your Setup

Test that Nix works correctly:

```bash
# Check Nix version
nix --version

# Test a simple search (may take a minute first time)
nix --extra-experimental-features "nix-command flakes" search nixpkgs hello --json
```

## Troubleshooting

### Daemon Not Running

If the daemon isn't running:

```bash
# Start the daemon
sudo systemctl start nix-daemon

# Enable it to start on boot
sudo systemctl enable nix-daemon
```

On systems without systemd (older WSL2):

```bash
# Start daemon manually
sudo nix-daemon --daemon &
```

### Permission Denied Errors

1. **Check you're in the nix-users group**:
   ```bash
   groups $USER
   ```

   Should show `nix-users` in the list.

2. **If not, add yourself**:
   ```bash
   sudo usermod -a -G nix-users $USER
   # Then log out and back in
   ```

3. **Check socket permissions**:
   ```bash
   ls -la /nix/var/nix/daemon-socket/
   ```

   The socket should be `srw-rw-rw-`

### Channel Not Found Errors

Update your Nix channels:

```bash
nix-channel --add https://nixos.org/channels/nixpkgs-unstable nixpkgs
nix-channel --update
```

### Slow First Search

The first `nix search` command downloads the package database and can take 2-5 minutes. This is normal! Subsequent searches will be much faster, especially with NSFW's caching.

## WSL2 Specific Notes

### Windows User → WSL2 User Mapping

When you run NSFW from Windows, it executes commands in WSL2 as your default WSL2 user. Make sure that user is in the `nix-users` group.

Find your default WSL2 user:

```bash
# In WSL2
whoami
```

### Multiple WSL2 Distributions

If you have multiple WSL2 distributions, install Nix in the one you want to use with NSFW. You can set the default with:

```powershell
# In PowerShell
wsl --set-default <distro-name>
```

## Configuration Files

### Enable Experimental Features Globally (Optional)

Instead of passing flags every time, you can enable experimental features in your Nix config:

```bash
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

**Note**: NSFW automatically adds the experimental features flags, so this is optional.

### Trusted Users (Optional)

For faster builds, you can add yourself as a trusted user:

```bash
# Edit /etc/nix/nix.conf (requires root)
echo "trusted-users = root $USER" | sudo tee -a /etc/nix/nix.conf

# Restart the daemon
sudo systemctl restart nix-daemon
```

## Uninstalling Nix

If you need to start over:

```bash
# Stop daemon
sudo systemctl stop nix-daemon

# Remove Nix
sudo rm -rf /nix
sudo rm -f /etc/profile.d/nix.sh

# Remove group
sudo groupdel nix-users
```

## Getting Help

- **Nix Manual**: https://nixos.org/manual/nix/stable/
- **Nix Search**: https://search.nixos.org/
- **NSFW Issues**: https://github.com/Luminous-Dynamics/nsfw/issues

## Summary

For most users, the quick fix is:

```bash
# Add yourself to nix-users
sudo usermod -a -G nix-users $USER

# Refresh group membership
newgrp nix-users

# Test NSFW
nsfw.exe search hello --limit 3
```

That's it! Your Nix setup should now work perfectly with NSFW.
