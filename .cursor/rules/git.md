# Sonic Flow - Git & Commit Strategy

## Commit Message Format

```
<type>(<scope>): <subject>

[optional body]
```

### Types

- `feat`: New feature implementation
- `fix`: Bug fixes
- `refactor`: Code restructuring without behavior change
- `perf`: Performance improvements
- `style`: Formatting, linting fixes
- `test`: Adding or fixing tests
- `docs`: Documentation updates
- `deps`: Dependency changes
- `config`: Configuration file changes

### Scopes

- `audio`: Audio processing, decoding, effects
- `visualizer`: Visualization engine and plugins
- `playlist`: Playlist management
- `ui`: User interface components
- `config`: Configuration system
- `plugin`: Plugin system
- `db`: Database operations
- `app`: Application layer

## Commit Granularity Rules

### 1. One Feature Per Commit

Each commit should represent one complete, functional change:

```bash
# ✅ Good - single focused change
git commit -m "feat(visualizer): implement spectrum bars plugin

- Add SpectrumBarsVisualizer struct with logarithmic scaling
- Implement VisualizerPlugin trait methods
- Add configuration for bar count and sensitivity
- Include comprehensive unit tests"

# ❌ Bad - multiple unrelated changes
git commit -m "feat: add visualizer and fix audio bug and update docs"
```

### 2. Dependencies Separate

Cargo.toml changes should be in separate commits:

```bash
# Commit 1: Add dependencies
git commit -m "deps: add rustfft 6.2 for spectrum analysis"

# Commit 2: Use dependencies
git commit -m "feat(audio): implement FFT spectrum analyzer"
```

### 3. Quality Improvements Separate

Formatting, linting, and refactoring in separate commits:

```bash
# Commit 1: Feature implementation
git commit -m "feat(audio): add audio buffer management"

# Commit 2: Code quality
git commit -m "style: apply rustfmt and clippy fixes"

# Commit 3: Refactoring (if needed)
git commit -m "refactor(audio): extract buffer pool utilities"
```

### 4. Tests With Features

Tests should be included with feature implementation, not separate:

```bash
# ✅ Good - tests included with feature
git add src/visualizer/spectrum_bars.rs tests/visualizer/spectrum_bars_test.rs
git commit -m "feat(visualizer): implement spectrum bars visualizer

- Add SpectrumBarsVisualizer with configurable parameters
- Support logarithmic and linear frequency scaling
- Include unit tests with mock spectrum data
- Add integration tests for plugin interface"
```

## Implementation Commit Sequence

### For Simple Features

```bash
# 1. Dependencies (if needed)
git commit -m "deps: add [crate] for [purpose]"

# 2. Feature implementation (with tests)
git commit -m "feat([scope]): [description]"

# 3. Style fixes (if needed)
git commit -m "style: apply rustfmt and clippy fixes"
```

### For Complex Features

```bash
# 1. Dependencies
git commit -m "deps: add [crates] for [purpose]"

# 2. Core implementation
git commit -m "feat([scope]): implement [core feature]"

# 3. Additional functionality
git commit -m "feat([scope]): add [additional feature]"

# 4. Integration
git commit -m "feat([scope]): integrate with [existing system]"

# 5. Style and refactoring
git commit -m "style: apply formatting and linting fixes"
git commit -m "refactor([scope]): extract common utilities"

# 6. Documentation
git commit -m "docs: update [module] API documentation"
```

## Commit Examples

### Good Commits

```bash
# Feature implementation
feat(audio): implement real-time FFT processor
feat(visualizer): add particle system visualization
feat(ui): implement dark theme support

# Bug fixes
fix(audio): resolve buffer underrun in low-latency mode
fix(playlist): prevent duplicate track insertion

# Performance improvements
perf(visualizer): optimize spectrum bar rendering with SIMD
perf(audio): reduce memory allocations in decode loop

# Refactoring
refactor(plugin): extract common plugin validation logic
refactor(config): simplify configuration loading

# Dependencies
deps: add symphonia 0.5 for audio decoding
deps: update slint to 1.6 for performance improvements
```

### Bad Commits (Avoid These)

```bash
# ❌ Too vague
git commit -m "fix stuff"
git commit -m "update code"
git commit -m "wip"

# ❌ Multiple responsibilities
git commit -m "feat: add visualizer and fix audio and update UI"

# ❌ Incomplete work
git commit -m "feat: half-implemented audio engine"
```

## Quality Checklist

Before each commit, verify:

- [ ] **Builds successfully**: `cargo build` passes
- [ ] **Tests pass**: `cargo test` passes
- [ ] **Linting clean**: `cargo clippy` shows no warnings
- [ ] **Single responsibility**: Commit does one thing well
- [ ] **Clear message**: Commit message explains what and why
- [ ] **Complete feature**: No half-implemented functionality

## Branch Strategy

- `main`: Production-ready code
- `develop`: Integration branch
- `feature/*`: Feature development branches
- `hotfix/*`: Critical bug fixes

## When Providing Commit Suggestions

Always suggest the complete sequence of commits needed for an implementation, following the granularity rules above. Include the exact commit messages that should be used.
