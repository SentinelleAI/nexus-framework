# Contributing to Nexus Framework

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork and clone the repository
2. Install Rust (1.75+): https://rustup.rs/
3. Build the project: `cargo build`
4. Run tests: `cargo test --workspace`

## Development Workflow

### Branch Naming

- `feat/description` — New features
- `fix/description` — Bug fixes
- `docs/description` — Documentation changes
- `refactor/description` — Code refactoring

### Code Quality

Before submitting a PR, ensure:

```bash
# Check compilation
cargo check --workspace

# Run tests
cargo test --workspace

# Run clippy linter
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build documentation
cargo doc --workspace --no-deps
```

### Commit Messages

Use clear, descriptive commit messages:

```
feat: add ValidatedJson extractor
fix: container panics when service not found
docs: add configuration guide
refactor: split lib.rs into modules
```

## Project Structure

```
nexus_framework/src/       # Core framework
nexus_framework_macros/src/ # Procedural macros
docs/                       # Documentation
tests/                      # Integration tests
```

## Code Style

- Follow standard Rust conventions (`rustfmt`)
- Add doc comments (`///`) to all public items
- Include module-level documentation (`//!`)
- Use `tracing` for logging, not `println!`

## Adding a New Feature

1. Add the implementation in the appropriate module
2. Export it through `lib.rs`
3. Add it to `prelude.rs` if commonly used
4. Write unit tests
5. Update documentation
6. Update `CHANGELOG.md`

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
