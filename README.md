# claude-code-web

A Rust workspace for scratch and one-off implementations.

## Structure

```
crates/
  scratch/     # General scratch crate for experiments
```

## Setup

Install just (task runner) and recommended cargo tools:

```bash
# Install just
cargo install just

# Install all recommended tools
just install-tools
```

## Quick Commands

```bash
just              # List all available commands
just all          # Run format check, clippy, tests, audit
just check        # Quick compile check
just test         # Run all tests
just clippy       # Run clippy lints
just fmt          # Format code
```

## Security & Dependencies

```bash
just audit        # Check for security vulnerabilities (cargo-audit)
just deny         # License and advisory checks (cargo-deny)
just geiger       # Scan for unsafe code usage
just outdated     # Check for outdated dependencies
just machete      # Find unused dependencies
just semver-check # Check for semver violations
```

## Analysis

```bash
just bloat scratch        # Analyze binary size
just bloat-funcs scratch  # Show largest functions
just bloat-deps scratch   # Show dependency sizes
```

## Adding a New Crate

```bash
cargo new crates/my-crate --lib  # or --bin
```

Then add workspace inheritance in the new `Cargo.toml`:

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[lints]
workspace = true
```

## Tooling

| Tool | Purpose | Command |
|------|---------|---------|
| rustfmt | Code formatting | `just fmt` |
| clippy | Linting (pedantic + nursery) | `just clippy` |
| cargo-deny | License & advisory checks | `just deny` |
| cargo-audit | Security vulnerability checks | `just audit` |
| cargo-geiger | Unsafe code analysis | `just geiger` |
| cargo-outdated | Dependency freshness | `just outdated` |
| cargo-machete | Unused dependency detection | `just machete` |
| cargo-semver-checks | Semver violation detection | `just semver-check` |
| cargo-bloat | Binary size analysis | `just bloat <bin>` |
