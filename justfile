# =============================================================================
# justfile - Daily development workflow
# =============================================================================
#
# Sonic Flow - Music Player with GPU-Accelerated Audio Visualization
#
# =============================================================================

# Update local main branch
new:
    git switch main && git pull --ff-only

# === Build Commands ===

# Build workspace in debug mode
build:
    cargo build --workspace

# Build app in release mode (optimized)
build-release:
    cargo build -p sonic-flow --release

# === Run Commands ===

# Run app (default)
run:
    cargo run -p sonic-flow

# Run app with debug logging
dev:
    RUST_LOG=debug,wgpu=warn,wgpu_hal=warn,naga=warn cargo run -p sonic-flow

# Run app in release mode
release:
    cargo run -p sonic-flow --release

# === Code Quality ===

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Quick check (format + clippy)
check:
    cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings

# === Testing ===

# Run all tests
test:
    cargo test --workspace

# Run unit tests: all crates / specific crate / specific test
# Examples:
#   just unit-test                              # All unit tests
#   just unit-test sonic-core                   # All unit tests in sonic-core
#   just unit-test sonic-core test_fft          # Specific test
unit-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --lib
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --lib
    else
        cargo test -p {{crate}} --lib {{test}}
    fi

# Run integration tests: all crates / specific crate / specific test
integration-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --tests
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --tests
    else
        cargo test -p {{crate}} --tests {{test}}
    fi

# Run tests sequentially (saves memory)
test-seq:
    cargo test --workspace -- --test-threads=1

# Run criterion benchmarks
bench:
    cargo bench --workspace

# === Clean ===

# Clean build artifacts
clean:
    cargo clean
