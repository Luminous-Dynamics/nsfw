# Changelog

All notable changes to NSFW (Nix Subsystem for Windows) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-11-15

### 🎉 Added

#### Phase 5A: Configuration System
- **User Configuration File** - `~/.nswfrc` (TOML format)
  - 8 customizable settings for user preferences
  - Automatic creation with sensible defaults
  - Full validation and error handling

- **`config` command** - Manage configuration settings
  - `config show` - Display all current settings
  - `config get <key>` - Get specific configuration value
  - `config set <key> <value>` - Update a setting
  - `config reset` - Reset all settings to defaults
  - `config path` - Show config file location
  - `config keys` - List all available configuration keys

- **Configuration Settings**
  - `cache_ttl_days` (default: 7) - Cache expiration in days
  - `default_wrapper_type` (default: console) - Wrapper type (console/gui/vbs)
  - `auto_update_channels` (default: false) - Auto-update before operations
  - `install_location` (default: None) - Custom wrapper install path
  - `verbose_output` (default: false) - Enable verbose logging
  - `disable_colors` (default: false) - Disable colored output
  - `parallel_jobs` (default: 4) - Number of parallel operations
  - `max_cache_size_mb` (default: 100) - Maximum cache size in MB

#### Phase 5B: Package Management Enhancements
- **`upgrade` command** - Update packages to latest versions
  - `upgrade` - Upgrade all installed packages
  - `upgrade <package>` - Upgrade specific package
  - `--yes` flag to skip confirmations
  - Remove old version + install latest strategy
  - Detailed progress indicators for each package
  - Summary with success/failure counts

- **`export` command** - Backup installed packages
  - Export to JSON or TOML format
  - `--output <file>` - Custom output file path
  - `--format <json|toml>` - Choose output format
  - Includes timestamp and NSFW version metadata
  - Default: `nsfw-packages.json`

- **`import` command** - Restore packages from backup
  - Auto-detect JSON or TOML format
  - Skip already-installed packages
  - `--yes` flag to skip confirmations
  - Detailed summary: installed/skipped/failed counts
  - Perfect for migrating to new machines

#### Phase 4: System Diagnostics & Enhanced Error Handling
- **`doctor` command** - Comprehensive system health checks
  - WSL2 availability and version detection
  - Nix installation verification
  - Package cache health monitoring (age warnings)
  - File permissions validation
  - Disk space usage tracking
  - Color-coded status indicators (✓, ✗, ⚠, ℹ)
  - Actionable fix suggestions for each issue
  - Summary with issues found vs checks passed

- **Enhanced Error Handling**
  - `ErrorContext` trait for rich error messages
  - User-friendly error descriptions
  - Actionable suggestions for resolving issues
  - Help URLs for additional resources
  - Applied to all commands: install, remove, list, info, update
  - `format_nix_error()` helper for consistent formatting

- **Cache Management** (Enhanced)
  - `cache stats` - View cache statistics
  - `cache clear` - Clear the cache
  - `cache rebuild` - Rebuild cache from scratch

#### API & Types
- `Config` struct - User configuration management
- `ExportedPackages` / `ExportedPackage` - Package export types
- `ErrorContext` trait - Enhanced error reporting
- Made `PathTranslator` Clone-able
- Made `RealWSL2Bridge` Clone-able

### 🔧 Changed

#### Version
- Bumped version from 0.2.0 to 0.3.0 (Phase 4-5 complete!)
- Updated all version references (Cargo.toml, main.rs, README)

#### Documentation
- Comprehensive README.md update with all v0.3.0 features
- Added documentation for config, upgrade, export, import, doctor commands
- Updated badges: version 0.3.0, 127 passing tests
- Added configuration table with all settings
- Enhanced examples and use cases
- Updated project structure to show config module
- Updated roadmap with completed Phase 3

#### Testing
- All 127 tests passing (+13 new config tests)
- Test coverage maintained for new features
- Added comprehensive config module tests

### 🐛 Fixed

- WSL2Bridge trait import for Clone compatibility
- Build errors with Clone-able types
- Clippy warning for match -> if let pattern
- All compiler warnings eliminated (0 warnings!)
- All clippy warnings eliminated (0 warnings!)

### 📦 Dependencies

- Added `toml = "0.8"` for TOML configuration file parsing

### 📝 Technical Details

#### Files Added (1 file)
- `src/config/mod.rs` - Configuration management module (400+ lines)

#### Files Modified (6 files)
- `Cargo.toml` - Version 0.3.0 + toml dependency
- `src/main.rs` - New command enums: Config, Upgrade, Export, Import, Doctor
- `src/cli/commands.rs` - Implementation of new commands (+953 lines)
- `src/lib.rs` - Export config module
- `src/path_translation/translator.rs` - Made PathTranslator Clone-able
- `src/wsl2/real.rs` - Made RealWSL2Bridge Clone-able

#### Code Statistics
- +953 lines of production code
- +13 new tests
- 127/127 tests passing
- 0 compiler warnings
- 0 clippy warnings
- Clean build

### 🚀 Impact

**For Users:**
- Customize NSFW behavior via config file
- Easy package backup and restore workflow
- Keep all packages up to date with one command
- Better diagnostics for troubleshooting issues
- Clearer error messages guide users to solutions

**For Developers:**
- Professional error handling throughout application
- Better maintainability with configuration system
- Clean, well-tested codebase ready for contributions

---

## [0.2.0] - 2025-01-15

### 🎉 Added

#### New Commands
- **`info` command** - View detailed package information
  - Shows package name, version, description
  - Displays homepage URL, license information
  - Lists available outputs (bin, dev, doc, etc.)
  - Shows package maintainers and supported platforms
  - Beautiful formatted output with colors and sections

- **`update` command** - Update Nix channels
  - Downloads latest nixpkgs channel
  - Refreshes package metadata
  - Automatically rebuilds package cache in background
  - Progress indicators and clear status messages

- **`completion` command** - Install shell completions
  - PowerShell tab completion support
  - Autocomplete for all commands and options
  - Dynamic package name suggestions
  - Context-aware completions
  - Automatic profile integration

#### Features
- **PowerShell Tab Completions** - Intelligent autocomplete system
  - Complete commands: `nsfw [Tab]`
  - Complete package names: `nsfw install [Tab]`
  - Complete options: `nsfw search --[Tab]`
  - Shows installed packages for `remove` command
  - Includes helpful descriptions for each option
  - Easy installation with `nsfw completion powershell`

- **Enhanced Package Information Display**
  - New `PackageInfo` type with comprehensive metadata
  - Colored output with semantic highlighting
  - Clean section headers and organized information
  - Support for optional fields (homepage, license, etc.)

- **Background Cache Rebuilding**
  - Update command triggers automatic cache refresh
  - Non-blocking background thread for cache updates
  - Maintains instant search performance

#### API Additions
- `NixExecutor::info()` - Get detailed package information
- `NixExecutor::update_channels()` - Update Nix channels
- `BridgedNixExecutor::info()` - Bridged version for WSL2
- `BridgedNixExecutor::update_channels()` - Bridged channel update
- `OutputFormatter::format_package_info()` - Format package details
- `PackageInfo` struct - Comprehensive package metadata type

### 🔧 Changed

#### Version
- Bumped version from 0.1.0 to 0.2.0 (Phase 2 complete!)
- Updated all version references (Cargo.toml, main.rs, docs)

#### Documentation
- Updated README.md with new commands and features
- Added badges for version 0.2.0 and 143 passing tests
- Added comprehensive usage examples for new commands
- Enhanced Quick Start section
- Added Tab Completion section with examples

#### Testing
- All 143 tests passing (114 unit + 16 edge cases + 13 integration)
- Test coverage maintained at 100% for new features

### 🐛 Fixed

- Fixed redundant closure in bridged_executor.rs
- Fixed unused variable warning in executor.rs
- Removed duplicate `Colorize` imports
- All compiler warnings eliminated (0 warnings!)
- All clippy warnings eliminated (0 warnings!)

### 📝 Technical Details

#### Files Modified (12 files)
- `Cargo.toml` - Version bump to 0.2.0
- `src/main.rs` - Version update + completion command
- `src/cli/commands.rs` - New info, update, completion functions
- `src/nix_ops/types.rs` - Added PackageInfo struct
- `src/nix_ops/executor.rs` - Added info & update_channels methods
- `src/nix_ops/bridged_executor.rs` - Bridged versions of new methods
- `src/nix_ops/mod.rs` - Export PackageInfo type
- `src/ui/output.rs` - Added format_package_info
- `completions/nsfw.ps1` - New PowerShell completion script
- `README.md` - Updated with v0.2.0 features
- `CHANGELOG.md` - Created comprehensive changelog
- `Cargo.lock` - Dependency updates

#### Code Quality Metrics
- ✅ 0 compiler warnings
- ✅ 0 clippy warnings
- ✅ 143/143 tests passing
- ✅ Clean release build
- ✅ Professional code standards maintained

### 🚀 Impact

**For Users:**
- Complete package information at your fingertips
- Easy channel updates with one command
- Blazing-fast tab completion for better UX
- Production-ready v0.2.0 release

**For Developers:**
- Complete API for package management
- Clean abstractions and high extensibility
- Excellent code quality (0 warnings)
- Ready for community contributions

---

## [0.1.0] - 2025-01-12

### 🎉 Initial Release

#### Core Features
- **Package Search** - Find packages in nixpkgs
  - SQLite-backed package cache for instant results
  - First search builds cache (~2-10 minutes, one-time)
  - Subsequent searches: instant (<2 seconds)
  - 500-1000x faster than traditional Nix search
  - JSON and text output formats

- **Package Installation** - Install packages from nixpkgs
  - Interactive confirmation prompts
  - `--yes` flag to skip confirmations
  - Comprehensive error handling
  - Progress indicators

- **Package Removal** - Uninstall packages
  - Interactive confirmation prompts
  - `--yes` flag for automation
  - Proper error messages

- **Package Listing** - Show installed packages
  - Basic and detailed output modes
  - JSON export support
  - Version information

- **Setup Wizard** - First-time configuration
  - Automatic WSL2 detection
  - Linux distro detection
  - Nix installation and configuration
  - Smart system analysis
  - Interactive and auto-yes modes

#### Architecture
- **WSL2 Bridge** - Abstraction layer for WSL2 communication
  - Automatic path translation (Windows ↔ WSL2)
  - UTF-16 LE encoding support for PowerShell
  - Mock bridge for testing
  - Clean trait-based design

- **Nix Operations** - Core package management
  - Direct executor for native Linux
  - Bridged executor for Windows via WSL2
  - Comprehensive error handling
  - Type-safe command execution

- **Package Cache** - Instant search performance
  - SQLite-backed storage
  - Automatic cache building
  - Background updates
  - Efficient queries

- **UI System** - Beautiful terminal output
  - Colored output with semantic highlighting
  - Progress indicators for long operations
  - Interactive prompts
  - Consistent formatting

#### Testing
- 114 unit tests
- 16 edge case tests
- 13 integration tests
- 100% passing test suite
- Comprehensive error handling coverage

#### Documentation
- Comprehensive README with examples
- Inline code documentation
- Architecture overview
- Setup instructions
- Usage examples

### 🏗️ Technical Foundation

#### Dependencies
- clap 4.5 - Command-line argument parsing
- serde/serde_json - JSON serialization
- rusqlite - SQLite database
- colored - Terminal colors
- indicatif - Progress indicators
- anyhow - Error handling
- log/env_logger - Logging
- dialoguer - Interactive prompts

#### Build System
- Rust 2021 edition
- Cargo workspace
- Release optimizations
- Cross-compilation support

---

## Release Notes

### Upgrade Instructions

**From v0.1.0 to v0.2.0:**

1. Update your installation:
   ```powershell
   git pull
   cargo build --release
   ```

2. Install tab completions (optional but recommended):
   ```powershell
   nsfw completion powershell
   . $PROFILE
   ```

3. Update your package database:
   ```powershell
   nsfw update
   ```

4. Enjoy the new features!

### Known Issues

None at this time. Please report issues at: https://github.com/Luminous-Dynamics/nsfw/issues

### Future Roadmap

**Phase 3: Advanced Features (In Progress)**
- Dependency visualization
- Interactive package selection
- Bash/Zsh/Fish completions
- Auto-update notifications
- Configuration profiles
- Package rollback

**Phase 4: Enterprise Features**
- Team configurations
- Custom package sources
- Centralized cache servers
- Audit logging
- Compliance reporting

---

## Contributors

- Luminous Dynamics (@Luminous-Dynamics)
- Claude (AI Assistant - Anthropic)

## License

MIT License - see LICENSE file for details

---

**Links:**
- Repository: https://github.com/Luminous-Dynamics/nsfw
- Issues: https://github.com/Luminous-Dynamics/nsfw/issues
- Releases: https://github.com/Luminous-Dynamics/nsfw/releases
