# Advanced Usage Guide

**NSFW - Nix Subsystem for Windows**
**Version**: v0.2.0+
**Audience**: Power users and automation

---

## 🎯 Table of Contents

1. [Scripting and Automation](#scripting-and-automation)
2. [JSON Output](#json-output)
3. [Performance Optimization](#performance-optimization)
4. [Caching System](#caching-system)
5. [Environment Variables](#environment-variables)
6. [Integration Patterns](#integration-patterns)
7. [Troubleshooting](#troubleshooting)
8. [API Usage (Developers)](#api-usage-developers)

---

## Scripting and Automation

### Non-Interactive Mode

Skip all confirmations with the `--yes` flag:

```powershell
# Install without confirmation
nsfw install firefox --yes

# Remove without confirmation
nsfw remove vim --yes

# Perfect for scripts
foreach ($pkg in @('git', 'vim', 'curl')) {
    nsfw install $pkg --yes
}
```

### Exit Codes

NSFW uses standard exit codes:

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | Package installed successfully |
| 1 | General error | Package not found |

Example usage:

```powershell
nsfw install firefox --yes
if ($LASTEXITCODE -eq 0) {
    Write-Host "Installation successful"
} else {
    Write-Host "Installation failed"
    exit $LASTEXITCODE
}
```

### Batch Installation Script

```powershell
# install-dev-tools.ps1
$packages = @(
    "git",
    "neovim",
    "ripgrep",
    "fd",
    "bat",
    "exa"
)

Write-Host "Installing development tools..."

foreach ($pkg in $packages) {
    Write-Host "Installing $pkg..."
    nsfw install $pkg --yes

    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to install $pkg" -ForegroundColor Red
        continue
    }
}

Write-Host "All tools installed!" -ForegroundColor Green
```

---

## JSON Output

### Search Results as JSON

```powershell
# Get JSON output
$results = nsfw search firefox --format json | ConvertFrom-Json

# Access fields
foreach ($pkg in $results) {
    Write-Host "$($pkg.pname) - $($pkg.version)"
    Write-Host "  $($pkg.description)"
}
```

### List Packages as JSON

```powershell
# Get installed packages as JSON
$installed = nsfw list --format json | ConvertFrom-Json

# Filter and process
$installed | Where-Object { $_.name -like "*vim*" } | ForEach-Object {
    Write-Host "$($_.name) ($($_.version))"
}
```

### Data Processing Pipeline

```powershell
# Find all python packages and save to file
nsfw search python --limit 100 --format json |
    ConvertFrom-Json |
    Where-Object { $_.pname -like "python*" } |
    ConvertTo-Csv -NoTypeInformation |
    Out-File python-packages.csv
```

---

## Performance Optimization

### Understanding the Cache

NSFW caches search results for 5 minutes:

```powershell
# First search (2-5 seconds)
Measure-Command { nsfw search firefox }
# Seconds: ~3.2

# Repeated search (< 1ms)
Measure-Command { nsfw search firefox }
# Seconds: ~0.001
```

### Optimal Search Strategies

#### Use Specific Queries
```powershell
# Slower: Generic query
nsfw search "editor"  # Returns many results

# Faster: Specific query
nsfw search "neovim"  # Returns exact matches first
```

#### Leverage the Cache
```powershell
# Search once, use results multiple times
nsfw search python > search-results.json

# Process results offline
$results = Get-Content search-results.json | ConvertFrom-Json
# Work with $results without re-querying
```

#### Limit Results
```powershell
# Get just what you need
nsfw search vim --limit 5  # Faster than default 20
nsfw search vim --limit 50 # More results if needed
```

---

## Caching System

### Cache Behavior

- **TTL**: 5 minutes from query time
- **Scope**: Query string + limit
- **Case**: Insensitive (`Firefox` = `firefox`)
- **Storage**: In-memory (cleared on restart)

### Cache Awareness

The cache is completely transparent but understanding it helps:

```powershell
# These use the same cache entry:
nsfw search firefox --limit 10
nsfw search FIREFOX --limit 10

# These use different cache entries:
nsfw search firefox --limit 10
nsfw search firefox --limit 20  # Different limit
nsfw search firefox-esr --limit 10  # Different query
```

### When Cache Helps Most

1. **Interactive Exploration**
   ```powershell
   # Search, review results, search again to reference
   nsfw search python
   # Review...
   nsfw search python  # Instant!
   ```

2. **Scripted Validation**
   ```powershell
   # Check if package exists before installing
   $exists = (nsfw search exact-package --format json | ConvertFrom-Json).Count -gt 0
   if ($exists) {
       nsfw install exact-package --yes
   }
   ```

3. **Repeated Queries**
   ```powershell
   # Dashboard script that checks packages periodically
   while ($true) {
       $available = nsfw search critical-pkg --format json | ConvertFrom-Json
       # Process...
       Start-Sleep -Seconds 60  # Within 5-min cache window
   }
   ```

---

## Environment Variables

### NO_COLOR

Disable colored output:

```powershell
# Temporarily
$env:NO_COLOR = "1"
nsfw search vim

# For session
$env:NO_COLOR = "1"
# All subsequent commands will be plain text
```

### VERBOSE Logging

Enable debug logging:

```powershell
nsfw --verbose search firefox
# Shows:
# - WSL2 bridge operations
# - Cache hits/misses
# - Nix command execution details
```

### CI/CD Environments

NSFW automatically detects non-interactive environments:

```yaml
# GitHub Actions example
- name: Install dependencies
  run: |
    nsfw install git --yes
    nsfw install nodejs --yes
  # Colors and prompts automatically disabled in CI
```

---

## Integration Patterns

### With PowerShell Profiles

Add to `$PROFILE`:

```powershell
# Aliases for common operations
function ns { nsfw search $args }
function ni { nsfw install $args --yes }
function nr { nsfw remove $args --yes }
function nl { nsfw list $args }

# Quick install function
function Install-NixPackages {
    param([string[]]$Packages)

    foreach ($pkg in $Packages) {
        ni $pkg
    }
}

# Usage:
# ns firefox      # Quick search
# ni git          # Quick install
# Install-NixPackages git,vim,curl
```

### With Package Managers

```powershell
# Check if package is in Nix before trying Chocolatey
function Install-Package {
    param([string]$Name)

    $nixResult = nsfw search $Name --limit 1 --format json | ConvertFrom-Json

    if ($nixResult.Count -gt 0) {
        Write-Host "Installing $Name via Nix..."
        nsfw install $Name --yes
    } else {
        Write-Host "Installing $Name via Chocolatey..."
        choco install $Name -y
    }
}
```

### With Development Workflows

```powershell
# Setup development environment
function Setup-DevEnv {
    param([string]$ProjectType)

    $packages = switch ($ProjectType) {
        "node" { @("nodejs", "yarn", "npm") }
        "python" { @("python3", "python3-pip", "python3-venv") }
        "rust" { @("rustc", "cargo") }
        "go" { @("go") }
    }

    Write-Host "Setting up $ProjectType environment..."
    foreach ($pkg in $packages) {
        nsfw install $pkg --yes
    }
}

# Usage:
# Setup-DevEnv -ProjectType node
```

---

## Troubleshooting

### Verbose Mode

Get detailed information:

```powershell
nsfw --verbose search firefox
# Shows:
# - Cache check results
# - WSL2 command execution
# - Nix output parsing
# - Error stack traces
```

### Common Issues

#### Cache Confusion

If you see stale results:

```powershell
# Cache entries expire after 5 minutes automatically
# To force fresh results, use a different query:
nsfw search firefox --limit 21  # Different limit = bypass cache
```

#### Performance Issues

```powershell
# Check WSL2 is responding
wsl echo "test"

# Verify Nix is accessible
wsl nix --version

# Test NSFW's bridge
nsfw --verbose search test
# Look for slow operations
```

#### JSON Parsing Errors

```powershell
# If --format json fails:
nsfw search vim --format json | Out-File test.json
Get-Content test.json  # Examine the output

# May indicate Nix version mismatch
wsl nix --version  # Ensure Nix 2.4+
```

---

## API Usage (Developers)

### Using NSFW as a Library

```rust
use nsfw::cache::SearchCache;
use nsfw::nix_ops::{BridgedNixExecutor, NixExecutor};
use nsfw::wsl2::RealWSL2Bridge;

// Create executor
let bridge = RealWSL2Bridge::new();
let executor = BridgedNixExecutor::new(bridge);

// Check cache first
if let Some(results) = SearchCache::get("firefox", 10) {
    println!("Cache hit: {} results", results.len());
} else {
    // Perform search
    match executor.search("firefox", 10) {
        Ok(results) => {
            SearchCache::put("firefox", 10, results.clone());
            println!("Found: {} packages", results.len());
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

### Cache Management

```rust
use nsfw::cache::SearchCache;

// Get cache statistics
let (total, expired) = SearchCache::stats();
println!("Cache: {} total, {} expired", total, expired);

// Clear expired entries
SearchCache::cleanup_expired();

// Clear all entries
SearchCache::clear();
```

### UI Utilities

```rust
use nsfw::ui::{ProgressIndicator, OutputFormatter, MessageType};

// Show progress
let progress = ProgressIndicator::spinner("Processing...");
progress.set_message("Still working...");
progress.finish_with_message("Done!");

// Format output
let message = OutputFormatter::format_message(
    MessageType::Success,
    "Operation complete"
);
println!("{}", message);
```

---

## Best Practices

### 1. Always Use `--yes` in Scripts

```powershell
# ✅ Good
nsfw install git --yes

# ❌ Bad (will hang waiting for input)
nsfw install git
```

### 2. Check Exit Codes

```powershell
# ✅ Good
nsfw install firefox --yes
if ($LASTEXITCODE -ne 0) {
    Write-Error "Installation failed"
    exit 1
}

# ❌ Bad (ignores failures)
nsfw install firefox --yes
# Continues even if it failed
```

### 3. Use JSON for Parsing

```powershell
# ✅ Good (machine-readable)
$results = nsfw search vim --format json | ConvertFrom-Json
$first = $results[0].pname

# ❌ Bad (fragile text parsing)
$text = nsfw search vim
$first = ($text -split "`n")[2] -replace '^\d+\.\s+', ''
```

### 4. Limit Search Results

```powershell
# ✅ Good (faster)
nsfw search vim --limit 5

# ⚠️ Acceptable but slower
nsfw search vim  # Returns 20 (default)

# ❌ Wasteful
nsfw search vim --limit 1000  # Rarely need this many
```

### 5. Leverage Caching

```powershell
# ✅ Good (check once, use many times)
$packages = nsfw search python --format json | ConvertFrom-Json

# Process $packages multiple times without re-querying

# ❌ Bad (repeats the search)
for ($i = 0; $i -lt 5; $i++) {
    $packages = nsfw search python --format json | ConvertFrom-Json
    # Processes...
}
```

---

## Performance Tips

### 1. Batch Operations

```powershell
# ✅ Fast (single search, multiple decisions)
$all_pkgs = nsfw search python --limit 50 --format json | ConvertFrom-Json

foreach ($pkg in $wanted) {
    if ($all_pkgs.pname -contains $pkg) {
        nsfw install $pkg --yes
    }
}

# ❌ Slow (searches for each package)
foreach ($pkg in $wanted) {
    $exists = (nsfw search $pkg --format json | ConvertFrom-Json).Count -gt 0
    if ($exists) {
        nsfw install $pkg --yes
    }
}
```

### 2. Parallel Execution

```powershell
# Install multiple packages in parallel
$packages = @("git", "vim", "curl")

$packages | ForEach-Object -Parallel {
    & nsfw install $_ --yes
} -ThrottleLimit 3
```

### 3. Optimize Search Queries

```powershell
# ✅ Specific (faster)
nsfw search neovim

# ⚠️ Generic (slower, more results)
nsfw search editor
```

---

## Security Considerations

### 1. Validate Package Names

```powershell
# Sanitize user input
$userInput = Read-Host "Enter package name"

# Only allow alphanumeric, dash, underscore
if ($userInput -match '^[a-zA-Z0-9\-_]+$') {
    nsfw install $userInput --yes
} else {
    Write-Error "Invalid package name"
}
```

### 2. Audit Before Bulk Install

```powershell
# Review before installing
$packages = @("git", "vim", "curl")

Write-Host "Will install: $($packages -join ', ')"
$confirm = Read-Host "Continue? (y/n)"

if ($confirm -eq 'y') {
    foreach ($pkg in $packages) {
        nsfw install $pkg --yes
    }
}
```

### 3. Log Operations

```powershell
# Log all operations
function Install-WithLogging {
    param([string]$Package)

    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    $logEntry = "$timestamp - Installing $Package"

    Add-Content -Path "nsfw-operations.log" -Value $logEntry

    nsfw install $Package --yes

    if ($LASTEXITCODE -eq 0) {
        Add-Content -Path "nsfw-operations.log" -Value "  SUCCESS"
    } else {
        Add-Content -Path "nsfw-operations.log" -Value "  FAILED"
    }
}
```

---

## Next Steps

- **Explore Phase 3**: Configuration files, shell completions, auto-update
- **Contribute**: Submit issues and PRs on GitHub
- **Join Community**: Discussions and support

---

**Built with ❤️ by Luminous Dynamics**
*Making NixOS accessible through consciousness-first technology*
