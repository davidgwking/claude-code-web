# claude-code-web

A Rust workspace for scratch and one-off implementations.

## Structure

```
crates/
  scratch/     # General scratch crate for experiments
```

## Quick Commands

```bash
cargo c        # Check all
cargo t        # Test all
cargo cl       # Clippy all
cargo fc       # Format check
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

- **rustfmt** - formatting
- **clippy** - linting (pedantic + nursery enabled)
- **cargo-deny** - license and advisory checks
