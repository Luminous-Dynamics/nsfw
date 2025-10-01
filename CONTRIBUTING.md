# Contributing to NSFW

Thank you for your interest in contributing to NSFW (Nix Subsystem for Windows)! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Code Style](#code-style)
- [Submitting Changes](#submitting-changes)
- [Reporting Bugs](#reporting-bugs)
- [Feature Requests](#feature-requests)

## Code of Conduct

This project follows the principles of respect, inclusivity, and constructive collaboration. Please:

- Be respectful and considerate in all interactions
- Welcome newcomers and help them get started
- Focus on constructive feedback
- Respect different perspectives and experiences

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/nsfw.git
   cd nsfw
   ```
3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/Luminous-Dynamics/nsfw.git
   ```

## Development Setup

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/)
- **WSL2**: Required for Windows testing
- **Nix**: Install in WSL2 via `curl -L https://nixos.org/nix/install | sh`

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run without building
cargo run -- search firefox
```

## Project Structure

```
nsfw/
├── src/
│   ├── cli/              # CLI command implementations
│   ├── nix_ops/          # Nix operations (search, install, etc.)
│   ├── path_translation/ # Windows ↔ WSL2 path conversion
│   ├── templates/        # Wrapper script generation
│   ├── wsl2/             # WSL2 bridge layer
│   ├── ui/               # User interface components
│   ├── cache/            # Search result caching
│   ├── lib.rs            # Library exports
│   └── main.rs           # CLI entry point
├── tests/
│   ├── integration_tests.rs  # Integration tests
│   └── edge_cases.rs         # Edge case tests
├── benches/
│   └── performance.rs        # Performance benchmarks
└── docs/                     # Documentation
```

### Key Components

- **WSL2Bridge**: Abstraction for WSL2 communication (trait-based, mockable)
- **PathTranslator**: Bidirectional Windows ↔ Linux path conversion
- **BridgedNixExecutor**: Executes Nix commands through WSL2 bridge
- **CLI**: Command-line interface built with Clap

## Development Workflow

1. **Create a branch** for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** with descriptive commits:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

3. **Keep your branch updated**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

### Commit Message Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test additions or changes
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `chore:` - Build/tooling changes

Examples:
```
feat: add support for NixOS packages
fix: correct path translation for UNC paths
docs: update installation instructions
test: add edge cases for path translation
```

## Testing

### Running Tests

```bash
# Run all tests (except doctests)
cargo test --lib --bins --tests

# Run specific test suite
cargo test --test integration_tests
cargo test --test edge_cases

# Run unit tests for specific module
cargo test path_translation::

# Run with verbose output
cargo test -- --nocapture
```

### Writing Tests

- **Unit tests**: Place in the same file as the code (`#[cfg(test)] mod tests`)
- **Integration tests**: Add to `tests/` directory
- **Edge cases**: Add to `tests/edge_cases.rs`

All new features should include:
- Unit tests for individual functions
- Integration tests for workflows
- Edge case tests for boundary conditions

### Test Coverage

Current coverage: **136 tests** (107 unit + 16 edge case + 13 integration)

Aim for:
- 100% coverage of public APIs
- Edge case coverage for path translation
- Error handling coverage

### Performance Benchmarks

Run benchmarks to verify performance:

```bash
cargo bench
```

Current benchmarks:
- Cache operations (read/write)
- Path translation (Windows ↔ Linux)
- Path type detection

## Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write idiomatic Rust code

### Style Requirements

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Fix common issues
cargo fix
```

### Documentation

- Add doc comments for public APIs (`///`)
- Include examples in doc comments
- Document error conditions
- Keep comments up-to-date

Example:
```rust
/// Translates a Windows path to a WSL2 Linux path.
///
/// # Arguments
///
/// * `path` - Windows path (e.g., "C:\\Users\\name\\file.txt")
///
/// # Returns
///
/// Linux path (e.g., "/mnt/c/Users/name/file.txt")
///
/// # Errors
///
/// Returns error if path is invalid or cannot be translated.
///
/// # Examples
///
/// ```
/// let translator = PathTranslator::new();
/// let result = translator.to_linux("C:\\Users\\Test\\file.txt")?;
/// assert_eq!(result, "/mnt/c/Users/Test/file.txt");
/// ```
pub fn to_linux(&self, path: &str) -> Result<String>
```

## Submitting Changes

### Pull Request Process

1. **Ensure all tests pass**:
   ```bash
   cargo test --lib --bins --tests
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

2. **Update documentation**:
   - Add/update doc comments
   - Update README.md if needed
   - Add entry to CHANGELOG (if exists)

3. **Create pull request**:
   - Use descriptive title following commit conventions
   - Describe changes and motivation
   - Reference related issues
   - Include test results

4. **Pull request template**:
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] All tests pass
   - [ ] New tests added
   - [ ] Benchmarks run (if applicable)

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Documentation updated
   - [ ] No new warnings
   ```

### Review Process

- Maintainers will review your PR
- Address feedback promptly
- Be open to suggestions
- Update PR based on review comments

## Reporting Bugs

### Before Reporting

1. Check [existing issues](https://github.com/Luminous-Dynamics/nsfw/issues)
2. Update to latest version
3. Verify bug is reproducible

### Bug Report Template

```markdown
**Describe the bug**
Clear description of the bug

**To Reproduce**
Steps to reproduce:
1. Run command '...'
2. See error '...'

**Expected behavior**
What you expected to happen

**Environment:**
- OS: [e.g., Windows 11]
- WSL2 version: [e.g., 2.0.0]
- NSFW version: [e.g., 0.1.0]
- Nix version: [e.g., 2.18.1]

**Additional context**
Any other relevant information
```

## Feature Requests

We welcome feature requests! Please:

1. Check [existing requests](https://github.com/Luminous-Dynamics/nsfw/issues?q=is%3Aissue+label%3Aenhancement)
2. Describe the feature and use case
3. Explain why it would be valuable
4. Consider implementation approach

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
Description of the problem

**Describe the solution you'd like**
Clear description of desired feature

**Describe alternatives you've considered**
Alternative solutions or features

**Additional context**
Mockups, examples, or other context
```

## Development Tips

### Using Mock Bridge for Testing

When testing Nix operations without WSL2:

```rust
use nsfw::wsl2::MockWSL2Bridge;
use nsfw::nix_ops::BridgedNixExecutor;

let mut bridge = MockWSL2Bridge::new();
bridge.set_response("nix search ...", r#"{"package": "..."}"#);

let executor = BridgedNixExecutor::new(bridge);
// Test your code
```

### Path Translation Testing

Test both directions and edge cases:

```rust
use nsfw::path_translation::PathTranslator;

let translator = PathTranslator::new();

// Windows → Linux
assert_eq!(
    translator.to_linux("C:\\Users\\Test")?,
    "/mnt/c/Users/Test"
);

// Linux → Windows
assert_eq!(
    translator.to_windows("/mnt/c/Users/Test")?,
    "C:\\Users\\Test"
);
```

### Debugging

Enable verbose logging:

```bash
# Debug logs
RUST_LOG=debug cargo run -- search firefox

# Trace logs (very detailed)
RUST_LOG=trace cargo run -- search firefox
```

## Questions?

- **Issues**: [GitHub Issues](https://github.com/Luminous-Dynamics/nsfw/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Luminous-Dynamics/nsfw/discussions)
- **Email**: tristan.stoltz@evolvingresonantcocreationism.com

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:
- Release notes
- Contributors list
- Project README

Thank you for contributing to NSFW! 🎉
