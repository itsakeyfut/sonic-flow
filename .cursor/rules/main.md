# Sonic Flow - Main Development Rules

## Project Overview

Sonic Flow is a high-quality music player with advanced audio visualization capabilities built with Rust and Slint UI framework.

### Core Principles

- **Visualizer-First**: Audio visualization is the primary feature
- **Quality-Focused**: Release-level quality standards
- **Real-time Performance**: 120fps visualizer target, 60fps minimum
- **Type Safety**: Leverage Rust's compile-time guarantees

## Technology Stack

- **Language**: Rust (2021 edition or later)
- **UI Framework**: Slint (mandatory - no other UI frameworks)
- **Architecture**: Layered architecture (UI/App/Business/Infrastructure)
- **Async Runtime**: tokio with multi-thread runtime

## 📚 External References

Always reference these official documentations for accurate implementation:

### Core Technologies

- **Rust Language**: https://doc.rust-lang.org/
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
- **Tokio Documentation**: https://docs.rs/tokio/latest/tokio/
- **Slint Documentation**: https://slint.dev/releases/1.8/docs/rust/slint/
- **Slint Language Reference**: https://slint.dev/releases/1.8/docs/slint/

### Audio Processing

- **Symphonia Documentation**: https://docs.rs/symphonia/latest/symphonia/
- **CPAL Documentation**: https://docs.rs/cpal/latest/cpal/
- **RustFFT Documentation**: https://docs.rs/rustfft/latest/rustfft/

### Data & Storage

- **SQLx Documentation**: https://docs.rs/sqlx/latest/sqlx/
- **Serde Documentation**: https://docs.rs/serde/latest/serde/
- **TOML Documentation**: https://docs.rs/toml/latest/toml/

### Error Handling & Utilities

- **Thiserror Documentation**: https://docs.rs/thiserror/latest/thiserror/
- **Anyhow Documentation**: https://docs.rs/anyhow/latest/anyhow/
- **Tracing Documentation**: https://docs.rs/tracing/latest/tracing/

### Testing & Benchmarking

- **Criterion Documentation**: https://docs.rs/criterion/latest/criterion/
- **Proptest Documentation**: https://docs.rs/proptest/latest/proptest/
- **Mockall Documentation**: https://docs.rs/mockall/latest/mockall/

**IMPORTANT**: When implementing features, always check the latest documentation for API changes and best practices.

## Documentation References

For comprehensive development guidance, reference:

- **README.md**: Project overview and setup instructions
- **CLAUDE.md**: Development guidelines and project context
- **.cursor/rules/**: Specialized development rules for each domain
  - `git.md`: Git workflow and commit strategies
  - `rust.md`: Rust coding standards
  - `architecture.md`: Architecture patterns
  - `performance.md`: Performance optimization
  - `ui.md`: Slint UI development
  - `testing.md`: Testing standards
  - `config.md`: Configuration management

**IMPORTANT**: Use project knowledge search to access implementation details and specifications.

## Performance Requirements (Non-Negotiable)

- **Audio Latency**: ≤ 50ms
- **UI Responsiveness**: ≤ 16ms (60fps)
- **Visualizer Rendering**: ≤ 8.3ms (120fps target)
- **Memory Usage**: ≤ 100MB (idle), ≤ 200MB (active)
- **CPU Usage**: ≤ 5% (during playback)

## Critical Restrictions

- **NO unwrap()**: Use proper error handling in production code
- **NO blocking operations**: In async contexts, especially UI thread
- **NO global mutable state**: Use proper state management patterns
- **NO localStorage/sessionStorage**: Not supported in artifacts
- **NO panic!()**: Handle all errors gracefully

## Implementation Response Format

When implementing features, provide responses in this structured format:

### 📋 Implementation Overview

Brief description and key design decisions

### 📁 File Changes

List of files created/modified with their specific purpose

### 🔨 Step-by-Step Implementation

1. **Dependencies** (if needed): Show Cargo.toml changes
2. **Core Implementation**: Main feature code with examples
3. **Tests**: Comprehensive test coverage
4. **Integration**: How it connects to existing system

### 🧪 Usage Example

Working code example demonstrating the feature

### 📝 Next Steps

Recommendations for follow-up implementations or improvements

## Module Architecture

```
src/
├── main.rs              # Application entry point
├── lib.rs              # Library root with public API
├── app/                # Application control layer
├── audio/              # Audio processing (FFT, decoding, effects)
├── visualizer/         # Visualization engine and plugins
├── ui/                 # Slint UI components
├── playlist/           # Playlist management
├── library/            # Music library and metadata
├── config/             # Configuration management
├── plugin/             # Plugin system
├── storage/            # Data persistence (SQLite)
├── utils/              # Shared utilities
├── error/              # Error types and handling
└── telemetry/          # Performance monitoring
```

## Development Workflow

1. **Check external docs**: Reference relevant official documentation
2. **Check project context**: Reference CLAUDE.md for project guidelines
3. **Review relevant rules**: Use appropriate .cursor/rules/ file for domain-specific guidance
4. **Search project knowledge**: Access detailed specifications and existing implementations
5. **Follow commit strategy**: Apply git workflow from git.md
6. **Ensure quality**: Meet all performance requirements and coding standards

## When in Doubt

1. Check official documentation for the relevant technology
2. Reference CLAUDE.md for overall project guidance
3. Check relevant .cursor/rules/ files for specific domain guidance
4. Use project knowledge search for implementation details
5. Follow Rust community best practices and API guidelines
6. Prioritize code clarity and real-time performance
