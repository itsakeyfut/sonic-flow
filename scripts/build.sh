#!/bin/bash

# Build and test script for Sonic Flow

set -e  # Exit on any error

echo "🎵 Sonic Flow - Build and Test Script"
echo "====================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Check Rust toolchain
print_status "Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    print_error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi

RUST_VERSION=$(rustc --version)
print_success "Found: $RUST_VERSION"

# Check for required system dependencies
print_status "Checking system dependencies..."

# Linux dependencies
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    print_status "Detected Linux - checking for audio libraries..."
    
    if ! pkg-config --exists alsa; then
        print_warning "ALSA development libraries not found. Install with:"
        print_warning "  Ubuntu/Debian: sudo apt-get install libasound2-dev"
        print_warning "  Fedora: sudo dnf install alsa-lib-devel"
    fi
    
    if ! pkg-config --exists libpulse; then
        print_warning "PulseAudio development libraries not found. Install with:"
        print_warning "  Ubuntu/Debian: sudo apt-get install libpulse-dev"
        print_warning "  Fedora: sudo dnf install pulseaudio-libs-devel"
    fi
fi

# Check dependencies
print_status "Updating and checking dependencies..."
cargo check --all-features
if [ $? -eq 0 ]; then
    print_success "Dependencies check passed"
else
    print_error "Dependencies check failed"
    exit 1
fi

# Format code
print_status "Formatting code..."
cargo fmt --all
print_success "Code formatting completed"

# Linting
print_status "Running linter (clippy)..."
cargo clippy --all-features --all-targets -- -D warnings
if [ $? -eq 0 ]; then
    print_success "Linting passed"
else
    print_warning "Linting found issues (non-fatal)"
fi

# Build in debug mode
print_status "Building in debug mode..."
cargo build --all-features
if [ $? -eq 0 ]; then
    print_success "Debug build completed"
else
    print_error "Debug build failed"
    exit 1
fi

# Run tests
print_status "Running tests..."
cargo test --all-features -- --nocapture
if [ $? -eq 0 ]; then
    print_success "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi

# Build documentation
print_status "Building documentation..."
cargo doc --all-features --no-deps
if [ $? -eq 0 ]; then
    print_success "Documentation built successfully"
    print_status "Documentation available at: target/doc/sonic_flow/index.html"
else
    print_warning "Documentation build failed (non-fatal)"
fi

# Build release version (optional)
if [ "$1" == "--release" ]; then
    print_status "Building release version..."
    cargo build --release --all-features
    if [ $? -eq 0 ]; then
        print_success "Release build completed"
        print_status "Binary available at: target/release/sonic-flow"
    else
        print_error "Release build failed"
        exit 1
    fi
fi

# Run benchmarks (if requested)
if [ "$1" == "--bench" ]; then
    print_status "Running benchmarks..."
    cargo bench --all-features
    if [ $? -eq 0 ]; then
        print_success "Benchmarks completed"
    else
        print_warning "Benchmarks failed or not available"
    fi
fi

# Check binary size (for release builds)
if [ -f "target/release/sonic-flow" ]; then
    BINARY_SIZE=$(du -h target/release/sonic-flow | cut -f1)
    print_status "Release binary size: $BINARY_SIZE"
fi

echo ""
print_success "🎉 Build process completed successfully!"
echo ""
print_status "Next steps:"
echo "  1. Run the application: cargo run"
echo "  2. Or use release build: ./target/release/sonic-flow"
echo "  3. Load an audio file and enjoy the visualizer!"
echo ""
print_status "For development:"
echo "  • Watch mode: cargo watch -x run"
echo "  • Debug logging: RUST_LOG=debug cargo run"
echo "  • Visualizer debug: RUST_LOG=sonic_flow::visualizer=trace cargo run"
echo ""

# Create a demo audio file if none exists
if [ ! -f "demo.wav" ] && [ ! -f "demo.mp3" ] && [ ! -f "demo.flac" ]; then
    print_status "No demo audio files found. You can:"
    print_status "  1. Place a music file named 'demo.mp3' in the project root"
    print_status "  2. Or use the Load Track button in the application"
    print_status "  3. The app can generate a demo audio file for testing"
fi
