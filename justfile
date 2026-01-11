# Default recipe: list all available commands
default:
    @just --list

# Run all checks (format, clippy, test, audit)
all: fmt-check clippy test audit

# Format all code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run clippy on all targets
clippy:
    cargo clippy --workspace --all-targets

# Run clippy and fix what can be fixed
clippy-fix:
    cargo clippy --workspace --all-targets --fix --allow-dirty

# Run all tests
test:
    cargo test --workspace

# Check all targets compile
check:
    cargo check --workspace --all-targets

# Build all targets
build:
    cargo build --workspace --all-targets

# Build release
build-release:
    cargo build --workspace --release

# --- Security & Dependency Tools ---

# Run cargo-audit for security vulnerabilities
audit:
    cargo audit

# Run cargo-deny for license and advisory checks
deny:
    cargo deny check

# Scan for unsafe code usage
geiger:
    cargo geiger --all-features --all-targets

# --- Dependency Management ---

# Check for outdated dependencies
outdated:
    cargo outdated --workspace

# Find unused dependencies
machete:
    cargo machete

# Check for semver violations (run before publishing)
semver-check:
    cargo semver-checks check-release

# --- Analysis ---

# Analyze binary size (specify binary with: just bloat <binary>)
bloat bin="scratch":
    cargo bloat --release -p {{bin}}

# Show largest functions (specify binary with: just bloat-funcs <binary>)
bloat-funcs bin="scratch":
    cargo bloat --release -p {{bin}} --filter '^(?!std::)'

# Show dependency sizes
bloat-deps bin="scratch":
    cargo bloat --release -p {{bin}} --crates

# --- Maintenance ---

# Update all dependencies
update:
    cargo update

# Clean build artifacts
clean:
    cargo clean

# Install all recommended tools
install-tools:
    cargo install cargo-audit cargo-deny cargo-geiger cargo-outdated cargo-machete cargo-semver-checks cargo-bloat

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open
