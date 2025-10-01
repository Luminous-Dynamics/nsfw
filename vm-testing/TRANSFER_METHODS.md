# How to Transfer NSFW Folder to Windows VM

**From**: NixOS host at `/srv/luminous-dynamics/11-meta-consciousness/nsfw/`
**To**: Windows VM (any location)

---

## 🌐 Method 1: Network Transfer (HTTP Server - Easiest)

### On NixOS Host:

```bash
# Navigate to parent directory
cd /srv/luminous-dynamics/11-meta-consciousness

# Start a simple HTTP server
python3 -m http.server 8000

# You'll see:
# Serving HTTP on 0.0.0.0 port 8000 (http://0.0.0.0:8000/) ...
```

**Keep this running!** Don't close the terminal.

**Find your host IP**:
```bash
# In another terminal on host
ip addr show | grep "inet " | grep -v 127.0.0.1
# Look for something like: 192.168.x.x or 10.0.x.x
```

### In Windows VM:

**Option A - Download with PowerShell**:
```powershell
# Open PowerShell
cd ~\Downloads

# Download (replace HOST_IP with actual IP from above)
Invoke-WebRequest -Uri http://HOST_IP:8000/nsfw/ -OutFile nsfw.zip

# Or if the folder is browsable, navigate in browser to:
# http://HOST_IP:8000
# Then manually download the nsfw folder
```

**Option B - Use WSL2 to download**:
```bash
# In WSL2
cd ~
wget -r -np -nH --cut-dirs=1 http://HOST_IP:8000/nsfw/

# Or download as tarball if you created one:
wget http://HOST_IP:8000/nsfw-for-vm.tar.gz
tar -xzf nsfw-for-vm.tar.gz
```

---

## 📁 Method 2: Shared Folder (If using virt-manager/libvirt)

### On NixOS Host:

```bash
# Check if virtiofs is available
virsh --connect qemu:///system capabilities | grep virtiofs

# Add shared folder to your VM
# Edit VM configuration:
virsh --connect qemu:///system edit nsfw-test-vm

# Add this inside <devices>:
#   <filesystem type='mount' accessmode='passthrough'>
#     <driver type='virtiofs'/>
#     <source dir='/srv/luminous-dynamics/11-meta-consciousness/nsfw'/>
#     <target dir='nsfw-shared'/>
#   </filesystem>

# Restart VM
virsh --connect qemu:///system shutdown nsfw-test-vm
virsh --connect qemu:///system start nsfw-test-vm
```

### In Windows VM:

The shared folder should appear as a network drive or in File Explorer under "Network locations".

---

## 📂 Method 3: Create Tarball and Transfer

### On NixOS Host:

```bash
# Create a compressed archive
cd /srv/luminous-dynamics/11-meta-consciousness
tar -czf nsfw-vm.tar.gz \
  nsfw/src \
  nsfw/Cargo.toml \
  nsfw/Cargo.lock \
  nsfw/vm-testing \
  nsfw/docs \
  nsfw/README.md

# Check size
ls -lh nsfw-vm.tar.gz
# Should be a few MB

# Then transfer this single file using Method 1 (HTTP server)
python3 -m http.server 8000
```

### In Windows VM (WSL2):

```bash
# Download and extract
cd ~
wget http://HOST_IP:8000/nsfw-vm.tar.gz
tar -xzf nsfw-vm.tar.gz

# Now you have the nsfw folder!
cd nsfw
```

---

## 🔗 Method 4: Git Clone (If you have a repo)

### In Windows VM (WSL2):

```bash
# If NSFW is on GitHub or a git server:
cd ~
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Or if it's a local repo you can access:
git clone http://HOST_IP:3000/nsfw.git  # If you run gitea/similar
```

**Note**: This requires the code to be in a git repository and accessible.

---

## 💾 Method 5: USB Drive (If VM can mount it)

### On NixOS Host:

```bash
# Plug in USB drive, find it
lsblk
# Look for your USB (e.g., /dev/sdb1)

# Mount it
sudo mount /dev/sdb1 /mnt/usb

# Copy files
cp -r /srv/luminous-dynamics/11-meta-consciousness/nsfw /mnt/usb/

# Safely unmount
sudo umount /mnt/usb
```

### In Windows VM:

1. Attach USB to VM (VM settings)
2. Open File Explorer
3. Copy from USB drive to Downloads or Desktop
4. Access from WSL2: `/mnt/c/Users/YourUser/Downloads/nsfw`

---

## 🌍 Method 6: SCP/SFTP (If VM has network and SSH)

### On NixOS Host:

```bash
# If your VM has SSH enabled with IP (e.g., 192.168.122.50)
scp -r /srv/luminous-dynamics/11-meta-consciousness/nsfw \
  user@192.168.122.50:/home/user/
```

### In Windows VM (WSL2):

```bash
# Or pull from host (if host has SSH server)
scp -r user@HOST_IP:/srv/luminous-dynamics/11-meta-consciousness/nsfw ~/
```

---

## 🎯 Recommended Method (Step-by-Step)

**For most users, HTTP server is simplest:**

### Step 1: On Host (NixOS)

```bash
# Terminal 1 - Start HTTP server
cd /srv/luminous-dynamics/11-meta-consciousness
python3 -m http.server 8000

# Terminal 2 - Get your IP
hostname -I | awk '{print $1}'
# Remember this IP! (e.g., 192.168.1.100)
```

### Step 2: In VM (WSL2)

```bash
# Download the folder
cd ~
wget -r -np -nH --cut-dirs=1 -R "index.html*" http://YOUR_HOST_IP:8000/nsfw/

# Check it downloaded
ls -la nsfw/
cd nsfw
ls Cargo.toml  # Should exist
```

### Step 3: Run Setup

```bash
# Still in WSL2, in nsfw directory
chmod +x vm-testing/vm-setup-script.sh
./vm-testing/vm-setup-script.sh
```

**Done!** 🎉

---

## 🐛 Troubleshooting

### "Connection refused" when accessing HTTP server

**On host, check firewall**:
```bash
# Temporarily allow port 8000
sudo iptables -I INPUT -p tcp --dport 8000 -j ACCEPT

# Or if using firewalld
sudo firewall-cmd --add-port=8000/tcp --permanent
sudo firewall-cmd --reload
```

### Can't reach host from VM

**Check VM network mode**:
- **NAT**: VM can access internet but not host directly
  - Solution: Use bridged networking
- **Bridged**: VM on same network as host ✅
- **Host-only**: VM can only talk to host ✅

**Test connectivity**:
```bash
# In VM WSL2
ping HOST_IP
```

### wget not found in WSL2

```bash
# Install it
sudo apt update
sudo apt install -y wget
```

### Download is slow/incomplete

**Create tarball first** (smaller, faster):
```bash
# On host
cd /srv/luminous-dynamics/11-meta-consciousness
tar -czf nsfw.tar.gz nsfw/
python3 -m http.server 8000

# In VM
wget http://HOST_IP:8000/nsfw.tar.gz
tar -xzf nsfw.tar.gz
```

---

## 📊 Method Comparison

| Method | Speed | Difficulty | Requirements |
|--------|-------|------------|--------------|
| **HTTP Server** | ⭐⭐⭐⭐ | ⭐ Easy | Network access |
| **Shared Folder** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ Medium | Virtiofs support |
| **Tarball + HTTP** | ⭐⭐⭐⭐⭐ | ⭐ Easy | Network access |
| **Git Clone** | ⭐⭐⭐ | ⭐ Easy | Git repo exists |
| **USB Drive** | ⭐⭐ | ⭐⭐ Medium | USB passthrough |
| **SCP** | ⭐⭐⭐⭐ | ⭐⭐ Medium | SSH enabled |

**Recommended**: HTTP Server with tarball (fastest + simplest)

---

## ✅ Quick Checklist

After transfer, verify:

```bash
# In WSL2
cd ~/nsfw  # Or wherever you put it
ls Cargo.toml  # ✓ Should exist
ls src/        # ✓ Should exist
ls vm-testing/vm-setup-script.sh  # ✓ Should exist

# If all exist, you're ready!
./vm-testing/vm-setup-script.sh
```

---

## 💡 Pro Tip: Create a Clean Tarball

**On host, create a minimal, clean package**:

```bash
cd /srv/luminous-dynamics/11-meta-consciousness/nsfw

# Create a clean tarball (excludes build artifacts)
tar -czf ../nsfw-clean.tar.gz \
  --exclude='target' \
  --exclude='.git' \
  --exclude='*.log' \
  .

cd ..
ls -lh nsfw-clean.tar.gz
# Should be ~2-5MB

# Serve it
python3 -m http.server 8000
```

**In VM**:
```bash
wget http://HOST_IP:8000/nsfw-clean.tar.gz
tar -xzf nsfw-clean.tar.gz
mv nsfw nsfw  # If extracted to current dir
cd nsfw
```

**This is the fastest method!** ⚡

---

**Choose your method and let's get that folder transferred!** 🚀

**Most likely to work**: HTTP server + tarball
**Fastest**: Shared folder (if supported)
**Simplest**: HTTP server direct download
