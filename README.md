# Sonic Flow

A high-quality music player with advanced audio spectrum visualizers, built with Rust and Slint.

## ✨ Features

### 🎵 Audio Playback

- **High-Quality Audio**: Support for FLAC, WAV, MP3, OGG, and AAC formats
- **Bit-Perfect Playback**: Up to 24-bit/192kHz audio support
- **Low Latency**: Sub-50ms audio latency for responsive playback
- **Advanced Effects**: 10-band equalizer, reverb, 3D audio processing

### 🎨 Advanced Visualizers

- **Spectrum Bars**: Classic frequency domain visualization
- **Waveform Display**: Time domain oscilloscope view
- **Circle Spectrum**: Circular frequency visualization
- **Particle System**: Dynamic particle-based visualization
- **3D Spectrum**: Three-dimensional frequency landscape
- **VU Meters**: Professional-style level meters

### 🔧 Extensibility

- **Plugin System**: Load custom visualizers at runtime
- **Theme Support**: Dark, light, and custom themes
- **Configurable**: Extensive customization options
- **Cross-Platform**: Windows, macOS, and Linux support

## 🚀 Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Git

### Installation

#### From Source

```bash
git clone https://github.com/sonic-flow/sonic-flow.git
cd sonic-flow
cargo build --release
```

#### From Crates.io (Coming Soon)

```bash
cargo install sonic-flow
```

### Quick Start

```bash
# Run the application
cargo run

# Or run the release build
./target/release/sonic-flow
```

## 🏗️ Architecture

Sonic Flow follows a layered architecture:

```
┌─────────────────────────────────────────────┐
│                UI Layer (Slint)             │
├─────────────────────────────────────────────┤
│            Application Layer                │
├─────────────────────────────────────────────┤
│           Business Logic Layer              │
├─────────────────────────────────────────────┤
│          Infrastructure Layer               │
└─────────────────────────────────────────────┘
```

For detailed architecture information, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## 📖 Documentation

- **[Architecture Guide](docs/ARCHITECTURE.md)** - System design and architecture
- **[Feature Specifications](docs/SPECIFICATION.md)** - Detailed feature specifications
- **[Directory Structure](docs/DIRECTORY.md)** - Project organization
- **[System Design](docs/SYSTEM.md)** - Low-level system design
- **[Development Guide](docs/CLAUDE.md)** - Development guidelines and context

## 🛠️ Development

### Development Setup

```bash
# Clone the repository
git clone https://github.com/sonic-flow/sonic-flow.git
cd sonic-flow

# Install dependencies
cargo check

# Run in development mode
cargo run

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
sonic-flow/
├── src/
│   ├── app/          # Application layer
│   ├── audio/        # Audio engine
│   ├── visualizer/   # Visualizer system
│   ├── ui/           # User interface
│   ├── config/       # Configuration management
│   └── ...
├── docs/             # Documentation
├── tests/            # Tests
└── assets/           # Static resources
```

### Development Workflow

This project uses a feature-branch development model:

1. **Create feature branch**: `git checkout -b feature/your-feature-name`
2. **Implement feature**: Follow the coding guidelines in [docs/CLAUDE.md](docs/CLAUDE.md)
3. **Write tests**: Ensure good test coverage
4. **Create PR**: Use the provided PR template

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# With specific features
cargo build --features "gpu-acceleration"
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_audio_engine

# Run benchmarks
cargo bench
```

## 🎯 Roadmap

### Phase 1: Core Foundation ✅

- [x] Project structure and build system
- [x] Error handling framework
- [x] Logging system
- [x] Basic application lifecycle

### Phase 2: Audio Engine ✅

- [x] Basic audio playback (MP3, FLAC, WAV)
- [x] Audio device management
- [x] Real-time FFT analysis
- [x] Basic spectrum analyzer

### Phase 3: Visualizers

- [ ] Spectrum bars visualizer
- [ ] Waveform visualizer
- [ ] Plugin system foundation
- [ ] Basic UI integration

### Phase 4: Advanced Features

- [ ] 3D visualizers
- [ ] Audio effects pipeline
- [ ] Advanced UI/UX
- [ ] Performance optimization

## 📊 Performance

Sonic Flow is designed for high performance:

- **Memory Usage**: <100MB idle, <150MB with visualizers
- **CPU Usage**: <5% during playback with visualizers
- **Audio Latency**: <50ms end-to-end
- **Visualizer FPS**: 60-120 FPS depending on complexity

## 🔧 Configuration

Configuration is managed through TOML files:

```toml
# ~/.config/sonic-flow/config.toml

[audio]
sample_rate = 44100
buffer_size = 512
device = "default"

[visualizer]
default_type = "spectrum_bars"
update_rate = 60
sensitivity = 1.0

[ui]
theme = "dark"
window_width = 1200
window_height = 800
```
