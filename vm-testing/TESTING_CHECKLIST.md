# NSFW Phase 2 Testing Checklist

**VM**: nsfw-test-win11
**Windows Version**: 11 Pro (Evaluation)
**Tester**: _____________
**Date**: _____________

## Pre-Test Setup

- [ ] VM created and Windows 11 installed
- [ ] WSL2 installed and working (`wsl --version`)
- [ ] Ubuntu distribution installed (`wsl -l -v`)
- [ ] Nix installed in WSL2 (`nix --version`)
- [ ] NSFW binary copied to Windows or built in WSL2
- [ ] Snapshot created: `virsh snapshot-create-as nsfw-test-win11 pre-test`

## Environment Verification

### Windows Environment
- [ ] Windows version: _________ (run `winver`)
- [ ] WSL version: _________ (run `wsl --version`)
- [ ] Distributions installed: _________ (run `wsl -l -v`)

### WSL2 Environment
```bash
# Run these in WSL2 and record results
uname -a     # Linux kernel version: _________
nix --version # Nix version: _________
which nix     # Nix path: _________
```

### NSFW Binary
- [ ] Binary location: _________
- [ ] Binary size: _________ KB
- [ ] Runs without error: `nsfw.exe --version`
- [ ] Version shown: _________

## Core Functionality Tests

### Test 1: Help Command
```powershell
nsfw.exe --help
```
- [ ] Help text displays
- [ ] All commands listed
- [ ] No errors
- **Time taken**: _________ seconds
- **Notes**: _________

### Test 2: Version Command
```powershell
nsfw.exe --version
```
- [ ] Version displays
- [ ] Matches expected version
- [ ] No errors
- **Version**: _________

### Test 3: Package Search
```powershell
nsfw.exe search firefox
```
- [ ] Search executes
- [ ] Results displayed
- [ ] Package info looks correct (name, version, description)
- [ ] No WSL2 errors
- **Time taken**: _________ seconds
- **Results count**: _________
- **Notes**: _________

### Test 4: Package Search with Limit
```powershell
nsfw.exe search python --limit 5
```
- [ ] Limit respected (max 5 results)
- [ ] Results formatted correctly
- **Time taken**: _________ seconds
- **Results count**: _________

### Test 5: List Installed Packages
```powershell
nsfw.exe list
```
- [ ] Command executes
- [ ] Lists installed packages (or shows empty list)
- [ ] Format is readable
- **Time taken**: _________ seconds
- **Packages found**: _________
- **Notes**: _________

### Test 6: Package Installation
```powershell
nsfw.exe install hello
```
- [ ] Confirmation prompt appears
- [ ] Installation proceeds when confirmed
- [ ] Success message displayed
- [ ] No errors during install
- **Time taken**: _________ seconds
- **Notes**: _________

### Test 7: Verify Installation
```powershell
nsfw.exe list
```
- [ ] Previously installed package appears in list
- [ ] Version information correct
- **Package shown**: _________

### Test 8: Package Removal
```powershell
nsfw.exe remove hello
```
- [ ] Confirmation prompt appears
- [ ] Removal proceeds when confirmed
- [ ] Success message displayed
- **Time taken**: _________ seconds
- **Notes**: _________

### Test 9: Skip Confirmation Flags
```powershell
nsfw.exe install cowsay --yes
nsfw.exe remove cowsay --yes
```
- [ ] `--yes` flag skips prompts
- [ ] Installation works
- [ ] Removal works
- **Notes**: _________

## Path Translation Tests

### Test 10: Windows Path Handling
```powershell
cd C:\Users\$env:USERNAME
nsfw.exe search vim
```
- [ ] Works from C:\ paths
- [ ] No path translation errors
- **Current dir**: _________
- **Result**: PASS / FAIL

### Test 11: Different Drive Letters
```powershell
# If you have D:\ drive
D:
nsfw.exe search git
```
- [ ] Works from different drives
- [ ] Path conversion correct
- **Result**: PASS / FAIL / SKIPPED

## Error Handling Tests

### Test 12: Non-existent Package
```powershell
nsfw.exe search thisdoesnotexist12345
```
- [ ] Handles gracefully
- [ ] Shows helpful message
- [ ] No crash
- **Error message**: _________

### Test 13: Invalid Command
```powershell
nsfw.exe invalidcommand
```
- [ ] Shows error
- [ ] Suggests help
- [ ] No crash
- **Error message**: _________

### Test 14: Network Interruption
- [ ] Disconnect network
- [ ] Run `nsfw.exe search test`
- [ ] Error message is clear
- [ ] Reconnect network
- [ ] Verify recovery
- **Error message**: _________
- **Recovery**: PASS / FAIL

## Performance Tests

### Test 15: Search Performance
Run each search 3 times and record times:

```powershell
Measure-Command { nsfw.exe search firefox }
```

| Attempt | Time (seconds) |
|---------|----------------|
| 1       | _________      |
| 2       | _________      |
| 3       | _________      |
| Average | _________      |

**Target**: < 5 seconds
**Result**: PASS / FAIL

### Test 16: Install Performance
```powershell
Measure-Command { nsfw.exe install hello --yes }
```
- **Time**: _________ seconds
- **Acceptable**: < 60 seconds for small package
- **Result**: PASS / FAIL

### Test 17: List Performance
```powershell
Measure-Command { nsfw.exe list }
```
- **Time**: _________ seconds
- **Target**: < 2 seconds
- **Result**: PASS / FAIL

## User Experience Tests

### Test 18: Output Readability
- [ ] Text is clearly formatted
- [ ] Colors/emojis display correctly (if used)
- [ ] No garbled characters
- [ ] Tables align properly
- **Rating**: 1-5 stars: _________

### Test 19: Error Messages
- [ ] Errors are understandable
- [ ] Suggest next steps
- [ ] Not overly technical
- **Sample error**: _________
- **Clarity rating**: 1-5 stars: _________

### Test 20: Help Documentation
- [ ] Help text is comprehensive
- [ ] Examples are clear
- [ ] Options are documented
- **Missing info**: _________

## Edge Cases

### Test 21: Very Long Package Names
```powershell
nsfw.exe search averylongpackagenamethatprobablydoesntexist
```
- [ ] Handles without crash
- [ ] Response is reasonable
- **Result**: PASS / FAIL

### Test 22: Special Characters
```powershell
nsfw.exe search "python3.11"
```
- [ ] Handles dots/special chars
- [ ] Results are relevant
- **Result**: PASS / FAIL

### Test 23: Multiple Rapid Commands
```powershell
nsfw.exe search vim
nsfw.exe search emacs
nsfw.exe search nano
nsfw.exe list
nsfw.exe search firefox
```
- [ ] All commands succeed
- [ ] No state corruption
- [ ] Performance consistent
- **Result**: PASS / FAIL

### Test 24: Large Result Sets
```powershell
nsfw.exe search python --limit 100
```
- [ ] Handles large results
- [ ] Performance acceptable
- [ ] Display not overwhelming
- **Time**: _________ seconds
- **Result**: PASS / FAIL

## Integration Tests

### Test 25: End-to-End Workflow
```powershell
# Complete workflow
nsfw.exe search firefox
nsfw.exe install firefox --yes
nsfw.exe list
firefox  # Launch from WSL2
nsfw.exe remove firefox --yes
nsfw.exe list
```
- [ ] All steps work
- [ ] Firefox actually installs and runs
- [ ] Removal is clean
- **Result**: PASS / FAIL
- **Notes**: _________

### Test 26: Multiple Package Management
```powershell
nsfw.exe install vim --yes
nsfw.exe install git --yes
nsfw.exe install curl --yes
nsfw.exe list
nsfw.exe remove vim --yes
nsfw.exe remove git --yes
nsfw.exe remove curl --yes
```
- [ ] Multiple installs work
- [ ] List shows all packages
- [ ] Removals work
- **Result**: PASS / FAIL

## WSL2-Specific Tests

### Test 27: WSL2 Detection
```powershell
# Without WSL2 (if possible to test)
wsl --unregister Ubuntu
nsfw.exe search vim
```
- [ ] Detects WSL2 absence
- [ ] Shows helpful error
- [ ] Suggests installation
- **Error message**: _________
- **Helpful**: YES / NO

### Test 28: Nix Detection
```bash
# In WSL2, temporarily hide Nix
mv ~/.nix-profile ~/.nix-profile.bak
```
```powershell
nsfw.exe search vim
```
- [ ] Detects Nix absence
- [ ] Shows helpful error
- [ ] Suggests installation
```bash
# Restore Nix
mv ~/.nix-profile.bak ~/.nix-profile
```
- **Error message**: _________
- **Helpful**: YES / NO

## Stress Tests

### Test 29: Rapid Fire Searches
Run this PowerShell loop:
```powershell
1..10 | ForEach-Object {
    Write-Host "Search $_"
    nsfw.exe search vim
}
```
- [ ] All searches complete
- [ ] No crashes
- [ ] Performance consistent
- **Average time**: _________ seconds
- **Result**: PASS / FAIL

### Test 30: Long Running Session
- [ ] Open PowerShell
- [ ] Run various commands for 30 minutes
- [ ] No memory leaks
- [ ] Performance stays consistent
- **Result**: PASS / FAIL

## Bug Reports

### Bug 1
- **Description**: _________
- **Steps to reproduce**: _________
- **Expected**: _________
- **Actual**: _________
- **Severity**: CRITICAL / HIGH / MEDIUM / LOW

### Bug 2
- **Description**: _________
- **Steps to reproduce**: _________
- **Expected**: _________
- **Actual**: _________
- **Severity**: CRITICAL / HIGH / MEDIUM / LOW

### Bug 3
- **Description**: _________
- **Steps to reproduce**: _________
- **Expected**: _________
- **Actual**: _________
- **Severity**: CRITICAL / HIGH / MEDIUM / LOW

## Performance Summary

| Operation | Target | Actual | Result |
|-----------|--------|--------|--------|
| Search    | <5s    | ___s   | ______ |
| Install   | <60s   | ___s   | ______ |
| Remove    | <30s   | ___s   | ______ |
| List      | <2s    | ___s   | ______ |

## Overall Assessment

### Strengths
1. _________
2. _________
3. _________

### Weaknesses
1. _________
2. _________
3. _________

### Critical Issues
1. _________
2. _________
3. _________

### Nice to Have
1. _________
2. _________
3. _________

## Test Results Summary

- **Total Tests**: 30
- **Passed**: _________
- **Failed**: _________
- **Skipped**: _________
- **Pass Rate**: _________%

## Production Readiness

- [ ] **YES** - Ready for production use
- [ ] **NO** - Needs more work
- [ ] **MAYBE** - Ready with caveats

**Reason**: _________

## Tester Notes

_________
_________
_________

## Recommendations for Phase 3

1. _________
2. _________
3. _________

---

**Checklist Version**: 1.0
**NSFW Version Tested**: _________
**Date Completed**: _________
**Tester Signature**: _________
