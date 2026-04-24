---
description: Start implementing a GitHub issue
allowed-tools: ["bash", "read", "write", "edit", "glob", "grep", "task"]
argument-hint: "<issue-number>"
---

First, fetch the issue details:

```bash
gh issue view $1
```

Now proceed with implementing this issue.

**Development Guidelines:**
- All comments and documentation must be written in English
- Follow Rust best practices and idiomatic patterns
- Prioritize real-time performance (audio latency, GPU rendering)
- No unwrap() in production code

**Before starting:**
1. Review the issue requirements carefully
2. Identify affected components:
   - Audio Engine (decoding, playback, FFT analysis)
   - Shader Pipeline (wgpu, WGSL shaders, GPU rendering)
   - Visualizer (spectrum bars, waveform, particles, effects)
   - UI (Slint components, callbacks, data binding)
   - Configuration (TOML settings, runtime config)
3. Review related documentation in `docs/specs/` and `docs/rules/`
4. Plan the implementation approach

**Crate placement:**
- `sonic-core` (`crates/sonic-core/`) — audio processing, FFT, decoding, metadata
- `sonic-shader` (`crates/sonic-shader/`) — wgpu, WGSL shaders, GPU rendering
- `sonic-visualizer` (`crates/sonic-visualizer/`) — visualizer logic, plugins
- `app/` — Slint UI, main entry point, audio-UI bridge

**Implementation checklist:**
- [ ] Place code in the appropriate crate
- [ ] Add unit tests in `#[cfg(test)]` sections where appropriate
- [ ] Document public APIs with rustdoc comments
- [ ] Use proper error handling with `Result` and `thiserror`
- [ ] Run `just fmt` before committing
- [ ] Run `just clippy` to check warnings
- [ ] Run `just test` to verify all tests pass
- [ ] Run `just build` to ensure it compiles

**Shader Guidelines (WGSL):**
- All visualizers are rendered via wgpu + WGSL (no image assets)
- FFT data is passed to shaders via uniform buffers
- Target 120fps for visualizer rendering (8.3ms per frame)
- Reference: `docs/specs/graphics.md`

**Commit Scopes:**
- `audio`: Audio processing, decoding, FFT
- `shader`: WGSL shaders, wgpu pipeline
- `visualizer`: Visualizer logic and plugins
- `ui`: Slint UI components
- `config`: Configuration system
- `app`: Application entry point
- `docs`: Documentation
- `chore`: Build, dependencies, tooling

Please proceed with the implementation.
