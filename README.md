# claude-code-web

A general-purpose multi-crate Rust workspace for Claude Code development.

## Workspace Structure

This workspace follows the standard Rust workspace pattern with the following crates:

```
claude-code-web/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── core/               # Core library crate
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   ├── utils/              # Utilities library crate
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── cli/                # CLI application crate
│       ├── Cargo.toml
│       └── src/main.rs
```

## Crates

### core
Foundational library providing core functionality for the workspace.
- Configuration management
- Serde integration for serialization/deserialization

### utils
Common utility functions and helpers.
- String manipulation
- General-purpose utilities

### cli
Command-line application demonstrating integration of workspace crates.

## Building

Build the entire workspace:
```bash
cargo build
```

Build a specific crate:
```bash
cargo build -p core
cargo build -p utils
cargo build -p cli
```

## Testing

Run all tests:
```bash
cargo test
```

Test a specific crate:
```bash
cargo test -p core
cargo test -p utils
```

## Running

Run the CLI application:
```bash
cargo run --bin cli
```

## Development

This workspace uses:
- Rust Edition 2021
- Workspace-level dependency management
- Resolver version 2 for dependency resolution
