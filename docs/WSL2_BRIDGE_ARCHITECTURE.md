# 🌉 WSL2 Bridge Architecture Design

**Purpose**: Enable seamless communication between Windows NSFW CLI and WSL2-based Nix daemon

**Status**: Design Document (Day 11-14)

## 🎯 Goals

1. **Transparent WSL2 Integration**: Windows CLI commands execute Nix operations in WSL2
2. **Testable Without WSL2**: Mock interface allows development/testing on Linux
3. **Robust Error Handling**: Graceful degradation when WSL2 unavailable
4. **Path Translation**: Automatic Windows ↔ Linux path conversion
5. **Performance**: Minimal overhead for command execution

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Windows CLI (nsfw.exe)               │
│  ┌───────────┐  ┌──────────────┐  ┌─────────────────┐ │
│  │  Commands │→│ WSL2 Bridge  │→│ Path Translator │ │
│  └───────────┘  └──────────────┘  └─────────────────┘ │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
            ┌─────────────────────────┐
            │   WSL2 Execution Layer  │
            │  (Real or Mock)         │
            └─────────────────────────┘
                          │
                          ▼
            ┌─────────────────────────┐
            │    WSL2 (NixOS/Nix)     │
            │  ┌──────────────────┐   │
            │  │  Nix Operations  │   │
            │  │  - search        │   │
            │  │  - install       │   │
            │  │  - remove        │   │
            │  │  - list          │   │
            │  └──────────────────┘   │
            └─────────────────────────┘
```

## 🔧 Core Components

### 1. WSL2Bridge Trait

Defines interface for WSL2 communication:

```rust
pub trait WSL2Bridge {
    /// Execute a command in WSL2
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput>;

    /// Check if WSL2 is available
    fn is_available(&self) -> bool;

    /// Get WSL2 version information
    fn version(&self) -> Result<String>;

    /// Translate path from Windows to WSL2
    fn translate_path_to_wsl(&self, windows_path: &str) -> Result<String>;

    /// Translate path from WSL2 to Windows
    fn translate_path_to_windows(&self, wsl_path: &str) -> Result<String>;
}
```

### 2. RealWSL2Bridge

Real implementation using `wsl.exe`:

```rust
pub struct RealWSL2Bridge {
    path_translator: PathTranslator,
}

impl WSL2Bridge for RealWSL2Bridge {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        // Translate any Windows paths in args
        let translated_args = self.translate_args(args)?;

        // Build wsl.exe command
        let output = Command::new("wsl")
            .arg(command)
            .args(&translated_args)
            .output()?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    fn is_available(&self) -> bool {
        Command::new("wsl")
            .arg("--version")
            .output()
            .is_ok()
    }
}
```

### 3. MockWSL2Bridge

Mock implementation for testing:

```rust
pub struct MockWSL2Bridge {
    responses: HashMap<String, CommandOutput>,
    available: bool,
    path_translator: PathTranslator,
}

impl MockWSL2Bridge {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            available: true,
            path_translator: PathTranslator::new(),
        }
    }

    pub fn set_response(&mut self, command: String, output: CommandOutput) {
        self.responses.insert(command, output);
    }

    pub fn set_available(&mut self, available: bool) {
        self.available = available;
    }
}

impl WSL2Bridge for MockWSL2Bridge {
    fn execute(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        let full_command = format!("{} {}", command, args.join(" "));

        self.responses
            .get(&full_command)
            .cloned()
            .ok_or_else(|| anyhow!("No mock response for: {}", full_command))
    }

    fn is_available(&self) -> bool {
        self.available
    }
}
```

### 4. BridgedNixExecutor

Modified NixExecutor that uses WSL2Bridge:

```rust
pub struct BridgedNixExecutor<B: WSL2Bridge> {
    bridge: B,
}

impl<B: WSL2Bridge> BridgedNixExecutor<B> {
    pub fn new(bridge: B) -> Self {
        Self { bridge }
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Check WSL2 available
        if !self.bridge.is_available() {
            return Err(NixError::WSL2NotAvailable.into());
        }

        // Execute via bridge
        let output = self.bridge.execute(
            "nix",
            &["search", "nixpkgs", query, "--json"]
        )?;

        // Parse results (same as before)
        self.parse_search_json(&output.stdout, limit)
    }

    // Similar for install, remove, list, etc.
}
```

## 📊 Command Flow

### Example: `nsfw install firefox`

1. **CLI Parsing**: Parse command-line arguments
2. **Bridge Creation**: Create appropriate bridge (Real or Mock)
3. **Executor Creation**: Create BridgedNixExecutor with bridge
4. **Command Execution**:
   ```rust
   let bridge = RealWSL2Bridge::new();
   let executor = BridgedNixExecutor::new(bridge);
   executor.install("firefox")?;
   ```
5. **WSL2 Bridge**:
   ```rust
   bridge.execute("nix", &["profile", "install", "nixpkgs#firefox"])
   ```
6. **System Call**: `wsl nix profile install nixpkgs#firefox`
7. **Result Processing**: Parse output and return to CLI

### Path Translation Example

```rust
// User provides Windows path
let file = "C:\\Users\\John\\config.nix";

// Bridge translates before execution
let wsl_path = bridge.translate_path_to_wsl(file)?;
// → "/mnt/c/Users/John/config.nix"

// Execute with translated path
bridge.execute("nix", &["build", "-f", &wsl_path])?;
```

## 🧪 Testing Strategy

### Unit Tests
- `test_real_bridge_availability` - Check WSL2 detection
- `test_real_bridge_version` - Get WSL2 version
- `test_real_bridge_command_execution` - Execute simple command
- `test_path_translation_integration` - Paths translated correctly

### Mock Tests
- `test_mock_bridge_responses` - Mock returns expected data
- `test_mock_bridge_unavailable` - Handle WSL2 unavailable
- `test_mock_bridge_errors` - Error conditions handled
- `test_mock_bridge_path_translation` - Mock path translation

### Integration Tests
- `test_bridged_executor_search` - Search via bridge
- `test_bridged_executor_install` - Install via bridge
- `test_bridged_executor_list` - List via bridge
- `test_bridged_executor_remove` - Remove via bridge
- `test_end_to_end_workflow` - Complete workflow

### Error Handling Tests
- `test_wsl2_not_installed` - Graceful error when WSL2 missing
- `test_wsl2_not_running` - Handle WSL2 stopped
- `test_nix_not_in_wsl` - Handle Nix not installed in WSL2
- `test_network_errors` - Handle network failures
- `test_path_translation_errors` - Invalid path handling

## 🔐 Error Handling

### New Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("WSL2 is not available. Please install WSL2.")]
    WSL2NotAvailable,

    #[error("WSL2 command failed: {0}")]
    ExecutionFailed(String),

    #[error("Failed to translate path: {0}")]
    PathTranslationFailed(String),

    #[error("WSL2 returned invalid output: {0}")]
    InvalidOutput(String),

    #[error("Nix is not installed in WSL2")]
    NixNotInstalled,
}
```

### Error Recovery
```rust
impl BridgedNixExecutor {
    fn execute_with_retry(&self, command: &str, args: &[&str]) -> Result<CommandOutput> {
        const MAX_RETRIES: u32 = 3;

        for attempt in 1..=MAX_RETRIES {
            match self.bridge.execute(command, args) {
                Ok(output) => return Ok(output),
                Err(e) if attempt < MAX_RETRIES => {
                    log::warn!("Attempt {} failed: {}, retrying...", attempt, e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                Err(e) => return Err(e),
            }
        }

        unreachable!()
    }
}
```

## 📈 Performance Considerations

### Overhead Analysis
- **WSL2 Startup**: ~50-100ms (first call)
- **Path Translation**: <1ms (cached regex)
- **Command Execution**: Same as native (WSL2 is fast)
- **Result Parsing**: <10ms (JSON parsing)

**Total Overhead**: ~60-110ms first call, <20ms subsequent calls

### Optimizations
1. **Command Batching**: Execute multiple commands in one WSL2 call
2. **Path Cache**: Cache translated paths
3. **Connection Reuse**: Keep WSL2 instance alive
4. **Parallel Execution**: Use tokio for async operations

## 🚀 Implementation Plan

### Day 11: Core Bridge (3-4 hours)
- [x] Create `src/wsl2/` module
- [ ] Implement WSL2Bridge trait
- [ ] Implement RealWSL2Bridge
- [ ] Implement MockWSL2Bridge
- [ ] Add basic tests (5-6 tests)

### Day 12: Integration (3-4 hours)
- [ ] Create BridgedNixExecutor
- [ ] Integrate with existing CLI commands
- [ ] Add path translation integration
- [ ] Add error handling
- [ ] Add integration tests (4-5 tests)

### Day 13: Testing & Polish (3-4 hours)
- [ ] Comprehensive error handling tests
- [ ] End-to-end workflow tests
- [ ] Performance testing
- [ ] Documentation
- [ ] Add retry logic

### Day 14: Final Integration (3-4 hours)
- [ ] Complete CLI integration
- [ ] Final testing on all platforms
- [ ] Update all documentation
- [ ] Create Phase 1 completion document
- [ ] Prepare for Phase 2 (Windows testing)

## ✅ Success Criteria

### Functional
- [ ] All Nix operations work via WSL2 bridge
- [ ] Path translation automatic and transparent
- [ ] Mock bridge enables testing without WSL2
- [ ] Error messages clear and actionable

### Quality
- [ ] 15+ new tests (80+ total)
- [ ] 100% test pass rate
- [ ] All error paths tested
- [ ] Documentation complete

### Performance
- [ ] <200ms overhead for first command
- [ ] <50ms overhead for subsequent commands
- [ ] No memory leaks
- [ ] Proper resource cleanup

## 🎯 Phase 1 Completion

After Days 11-14 complete:
- ✅ CLI skeleton
- ✅ Nix operations layer
- ✅ Path translation
- ✅ Wrapper generation
- ✅ WSL2 bridge architecture

**Ready for Phase 2**: Windows testing and production deployment

---

**Architecture Status**: Design Complete ✅
**Next**: Begin Implementation (Day 11)
**Estimated Total**: 500 lines, 15+ tests, 4 days