# Sonic Shader

Slang shader engine for Sonic Flow GPU-accelerated audio visualization.

## 🚧 Current Status

**Note**: This is a development version with placeholder Slang integration. The actual Slang compiler integration is planned for future releases.

### What's Working

- ✅ Basic shader engine architecture
- ✅ WGSL shader compilation (via wgpu)
- ✅ GPU rendering pipeline
- ✅ Audio visualization uniforms
- ✅ Shader metadata extraction
- ✅ Basic Slang-to-WGSL syntax conversion

### What's Planned

- 🔄 Full Slang compiler integration
- 🔄 Multi-target compilation (Vulkan, Metal, DirectX 12)
- 🔄 Advanced shader optimization
- 🔄 Hot reloading support

## 🛠️ Implementation Details

### Current Approach

Instead of using the actual Slang compiler (which has dependency conflicts), we currently:

1. **Parse Slang-like syntax** from shader files
2. **Convert to WGSL** using basic string replacement
3. **Compile with wgpu** for GPU execution

### Future Slang Integration

When Slang becomes more stable and dependency conflicts are resolved, we plan to:

1. **Use actual Slang compiler** for proper multi-target support
2. **Implement advanced optimizations**
3. **Add reflection capabilities**
4. **Support Slang's advanced features** (generics, parameter blocks, etc.)

## 📁 Project Structure

```
libs/sonic-shader/
├── src/
│   ├── lib.rs              # Main library entry point
│   ├── types.rs            # Common types and error definitions
│   ├── compiler.rs         # Shader compilation (placeholder Slang)
│   ├── engine.rs           # Shader engine management
│   ├── pipeline.rs         # GPU rendering pipeline
│   ├── renderer.rs         # GPU renderer implementation
│   └── shaders/            # Shader files
│       ├── common/
│       │   └── uniforms.slang  # Common uniform definitions
│       └── spectrum_bars.slang # Example spectrum bars shader
```

## 🎯 Usage Example

```rust
use sonic_shader::{ShaderEngine, ShaderCompiler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize shader engine
    let mut engine = ShaderEngine::new();

    // Load a shader (converts Slang-like syntax to WGSL)
    let shader_source = include_str!("shaders/spectrum_bars.slang");
    engine.load_shader("spectrum_bars", shader_source, "vertexMain", "fragmentMain")?;

    // Activate the shader
    engine.activate_shader("spectrum_bars")?;

    println!("Shader engine ready!");
    Ok(())
}
```

## 🔧 Development

### Building

```bash
cargo build --package sonic-shader
```

### Testing

```bash
cargo test --package sonic-shader
```

### Running Examples

```bash
cargo run --example basic_shader
```

## 📋 TODO

- [ ] Resolve Slang dependency conflicts
- [ ] Implement actual Slang compiler integration
- [ ] Add more shader examples
- [ ] Implement shader hot reloading
- [ ] Add performance benchmarks
- [ ] Create shader debugging tools

## 🤝 Contributing

When contributing to this crate:

1. **Follow the placeholder approach** for now
2. **Document any Slang-specific syntax** you add
3. **Update conversion functions** in `compiler.rs`
4. **Add tests** for new functionality
5. **Keep dependency conflicts in mind**

## 📚 References

- [Slang Documentation](https://shader-slang.org/docs/)
- [wgpu Documentation](https://docs.rs/wgpu/)
- [WGSL Specification](https://www.w3.org/TR/WGSL/)
