# NSFW - Nix Subsystem for Windows

**N**ix **S**ubsystem **F**or **W**indows - Natural language Nix package management for Windows via WSL2.

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-124%20passing-brightgreen.svg)](#testing)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](#building)

## What is NSFW?

NSFW is a Windows CLI tool that makes Nix package management accessible and easy. It bridges Windows and WSL2/Nix, translating paths automatically and providing a friendly interface to the powerful Nix package manager.

### Key Features

- 🪟 **Windows Native**: Run Nix operations from Windows PowerShell/CMD
- 🔄 **Automatic Path Translation**: Seamlessly converts Windows paths to WSL2 paths
- 📦 **Package Management**: Search, install, remove, and list Nix packages
- 🧪 **Well Tested**: 124 tests ensuring reliability
- 🚀 **Fast**: Lightweight CLI with minimal overhead
- 🎯 **User Friendly**: Clear output and helpful error messages

## Prerequisites

- Windows 10/11 with WSL2 enabled
- A WSL2 Linux distribution (Ubuntu recommended)
- Nix package manager installed in WSL2

### Installing Prerequisites

```powershell
# Install WSL2 (requires restart)
wsl --install

# Restart your computer

# Install Nix in WSL2
wsl
curl -L https://nixos.org/nix/install | sh
source ~/.nix-profile/etc/profile.d/nix.sh
```

## Installation

### From Source

```powershell
# Clone the repository
git clone https://github.com/Luminous-Dynamics/nsfw.git
cd nsfw

# Build the project
cargo build --release

# The binary will be in target/release/nsfw.exe
# Add it to your PATH or copy it to a directory in your PATH
```

### Pre-built Binary (Coming Soon)

Download the latest release from the [Releases](https://github.com/Luminous-Dynamics/nsfw/releases) page.

## Quick Start

```powershell
# Search for a package
nsfw search firefox

# Install a package
nsfw install firefox

# List installed packages
nsfw list

# Remove a package
nsfw remove firefox

# Get help
nsfw --help
```

## Usage

### Search for Packages

```powershell
# Basic search
nsfw search <package-name>

# Search with custom limit
nsfw search firefox --limit 50

# Search with JSON output
nsfw search python --format json
```

### Install Packages

```powershell
# Interactive install (will prompt for confirmation)
nsfw install firefox

# Skip confirmation
nsfw install firefox --yes

# Alias: add
nsfw add python3
```

### List Installed Packages

```powershell
# Basic list
nsfw list

# Detailed information
nsfw list --detailed

# JSON output
nsfw list --format json

# Alias: ls
nsfw ls -d
```

### Remove Packages

```powershell
# Interactive removal (will prompt for confirmation)
nsfw remove firefox

# Skip confirmation
nsfw remove firefox --yes

# Alias: uninstall
nsfw uninstall python3
```

### Generate Wrapper Scripts

NSFW can generate Windows batch wrappers for Nix packages:

```powershell
# After installing a package, generate a wrapper
nsfw generate-wrapper firefox /nix/store/path-to-firefox
```

This creates a `.bat` file that allows you to run the Nix package from Windows.

### Advanced Options

```powershell
# Enable verbose logging
nsfw --verbose search firefox

# Show help for specific command
nsfw install --help

# Check version
nsfw --version
```

## How It Works

```
Windows CLI (nsfw.exe)
        ↓
    WSL2Bridge
        ↓
    WSL2 Environment
        ↓
    Nix Commands
```

1. **Path Translation**: Automatically converts Windows paths (C:\Users) to WSL2 paths (/mnt/c/Users)
2. **Command Execution**: Routes commands through WSL2 using `wsl.exe`
3. **Result Processing**: Parses JSON output from Nix commands
4. **User Display**: Formats and displays results in a user-friendly way

## Architecture

NSFW is built with a clean, modular architecture:

- **CLI Layer**: User interface and command handling
- **Nix Operations**: Package search, installation, removal
- **WSL2 Bridge**: Abstraction layer for WSL2 communication
- **Path Translation**: Bidirectional Windows ↔ WSL2 path conversion
- **Template Generator**: Creates Windows wrappers for Nix packages

See [docs/WSL2_BRIDGE_ARCHITECTURE.md](docs/WSL2_BRIDGE_ARCHITECTURE.md) for detailed architecture documentation.

## Testing

NSFW has comprehensive test coverage:

```powershell
# Run all tests
cargo test

# Run only library and integration tests (skip doctests)
cargo test --lib --bins --tests

# Run specific test suite
cargo test --test integration_tests
cargo test --test edge_cases
```

### Test Statistics
- **124 total tests**
- **100% pass rate**
- **0.03s execution time**

Test coverage includes:
- 95 unit tests
- 13 integration tests
- 16 edge case tests

## Development

### Building

```powershell
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run without building
cargo run -- search firefox
```

### Project Structure

```
nsfw/
├── src/
│   ├── cli/              # CLI command implementations
│   ├── nix_ops/          # Nix operations (search, install, etc.)
│   ├── path_translation/ # Windows ↔ WSL2 path conversion
│   ├── templates/        # Wrapper script generation
│   ├── wsl2/             # WSL2 bridge layer
│   ├── lib.rs            # Library exports
│   └── main.rs           # CLI entry point
├── tests/
│   ├── integration_tests.rs  # Integration tests
│   └── edge_cases.rs         # Edge case tests
└── docs/                 # Documentation
```

### Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Ensure all tests pass
5. Submit a pull request

See our [Contributing Guide](CONTRIBUTING.md) for more details.

## Troubleshooting

### "WSL2 is not available"

Ensure WSL2 is installed and working:
```powershell
wsl --version
wsl --list
```

### "Nix not found"

Install Nix in your WSL2 distribution:
```bash
wsl
curl -L https://nixos.org/nix/install | sh
```

### "Package not found"

Update your Nix channels:
```bash
wsl
nix-channel --update
```

### Command hangs or is slow

This is usually due to Nix updating its database. The first operation after a channel update may take several minutes.

## Roadmap

### Phase 1: Foundation ✅ (Complete)
- CLI interface
- WSL2 bridge
- Path translation
- Basic Nix operations
- Comprehensive testing

### Phase 2: Polish (In Progress)
- Windows testing
- Performance optimization
- Enhanced error messages
- Configuration support

### Phase 3: Advanced Features (Planned)
- Package dependency visualization
- Search result caching
- Interactive package selection
- Shell completions
- Auto-update notifications

## FAQ

**Q: Do I need to know Nix to use NSFW?**
A: No! NSFW provides a simple interface to Nix. Just search and install like any other package manager.

**Q: Can I use NSFW without WSL2?**
A: No, NSFW requires WSL2 as it bridges to Nix running in WSL2.

**Q: Will this work with WSL1?**
A: No, WSL2 is required for proper file system integration.

**Q: How is this different from running Nix directly in WSL2?**
A: NSFW provides Windows-native access, automatic path translation, and a friendlier interface.

**Q: Can I use this in scripts?**
A: Yes! Use `--yes` to skip confirmation prompts and `--format json` for machine-readable output.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [NixOS](https://nixos.org/) - The Nix package manager
- [Microsoft](https://microsoft.com/) - WSL2 technology
- [Anthropic](https://anthropic.com/) - Claude AI assistance
- [Luminous Dynamics](https://luminousdynamics.org/) - Project sponsor

## Support

- **Issues**: [GitHub Issues](https://github.com/Luminous-Dynamics/nsfw/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Luminous-Dynamics/nsfw/discussions)
- **Email**: tristan.stoltz@evolvingresonantcocreationism.com

## Project Status

- **Current Phase**: 1 (Complete)
- **Stability**: Alpha
- **Production Ready**: Not yet (pending Phase 2 Windows testing)
- **Active Development**: Yes

---

**Built with ❤️ by [Luminous Dynamics](https://luminousdynamics.org/)**

*Making NixOS accessible to all through consciousness-first technology.*
