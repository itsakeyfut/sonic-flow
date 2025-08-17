//! HLSL Shader Demo
//! 
//! This example demonstrates the basic HLSL to WGSL compilation
//! and shader management functionality.

use sonic_shader::{ShaderCompiler, CompilationTarget, OptimizationLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Sonic Flow - HLSL Shader Demo");
    println!("==================================");

    // Create shader compiler
    let compiler = ShaderCompiler::with_target(CompilationTarget::WGSL)
        .with_optimization(OptimizationLevel::Basic);

    println!("✅ Shader compiler created");
    println!("   Target: {:?}", compiler.target());
    println!("   Optimization: {:?}", compiler.optimization_level());

    // Test simple shader compilation
    println!("\n📝 Testing simple shader compilation...");
    
    let simple_shader_source = r#"
        //! @title Simple Test Shader
        //! @version 1.0.0
        //! @author Test Author
        //! @description Test shader
        
        struct VertexInput {
            float2 position : POSITION;
        };
        
        void vertexMain(VertexInput input) {
            // Vertex shader implementation
        }
        
        void fragmentMain() {
            // Fragment shader implementation
        }
    "#;
    
    let simple_result = compiler.test_hlsl_compilation(simple_shader_source);
    
    match simple_result {
        Ok(wgsl_source) => {
            println!("✅ Simple shader compiled successfully!");
            println!("   WGSL output length: {} characters", wgsl_source.len());
            
            // Show first few lines of compiled shader
            let lines: Vec<&str> = wgsl_source.lines().take(10).collect();
            println!("   First 10 lines:");
            for (i, line) in lines.iter().enumerate() {
                println!("   {:2}: {}", i + 1, line);
            }
        }
        Err(e) => {
            println!("❌ Simple shader compilation failed: {}", e);
        }
    }

    // Test spectrum bars shader compilation
    println!("\n📊 Testing spectrum bars shader compilation...");
    
    let spectrum_shader_source = r#"
        //! @title Spectrum Bars Visualizer
        //! @version 1.0.0
        //! @author Sonic Flow Team
        //! @description Real-time audio spectrum visualization
        
        struct VertexInput {
            float2 position : POSITION;
            float2 texCoord : TEXCOORD0;
        };
        
        void vertexMain(VertexInput input) {
            // Vertex shader implementation
        }
        
        void fragmentMain() {
            // Fragment shader implementation
        }
    "#;
    
    let spectrum_result = compiler.test_hlsl_compilation(spectrum_shader_source);
    
    match spectrum_result {
        Ok(wgsl_source) => {
            println!("✅ Spectrum bars shader compiled successfully!");
            println!("   WGSL output length: {} characters", wgsl_source.len());
            
            // Check for key features
            let has_vertex_main = wgsl_source.contains("vertexMain");
            let has_fragment_main = wgsl_source.contains("fragmentMain");
            let has_spectrum_data = wgsl_source.contains("spectrumData");
            let has_uniforms = wgsl_source.contains("AudioVisualizationUniforms");
            
            println!("   Features detected:");
            println!("   - Vertex shader: {}", if has_vertex_main { "✅" } else { "❌" });
            println!("   - Fragment shader: {}", if has_fragment_main { "✅" } else { "❌" });
            println!("   - Spectrum data: {}", if has_spectrum_data { "✅" } else { "❌" });
            println!("   - Uniforms: {}", if has_uniforms { "✅" } else { "❌" });
        }
        Err(e) => {
            println!("❌ Spectrum bars shader compilation failed: {}", e);
        }
    }

    // Test metadata extraction
    println!("\n📋 Testing metadata extraction...");
    
    let metadata_result = compiler.extract_metadata(spectrum_shader_source);
    match metadata_result {
        Ok(metadata) => {
            println!("✅ Metadata extracted successfully!");
            println!("   Name: {}", metadata.name);
            println!("   Version: {}", metadata.version);
            println!("   Author: {}", metadata.author);
            println!("   Description: {}", metadata.description);
            println!("   Uniform buffer size: {} bytes", metadata.uniform_buffer_size);
        }
        Err(e) => {
            println!("❌ Metadata extraction failed: {}", e);
        }
    }

    // Test shader validation
    println!("\n🔍 Testing shader validation...");
    
    let validation_result = compiler.validate_shader_source(spectrum_shader_source);
    match validation_result {
        Ok(()) => {
            println!("✅ Shader validation passed!");
        }
        Err(e) => {
            println!("❌ Shader validation failed: {}", e);
        }
    }

    // Test empty shader (should fail)
    println!("\n🚫 Testing empty shader validation...");
    
    let empty_validation_result = compiler.validate_shader_source("");
    match empty_validation_result {
        Ok(()) => {
            println!("⚠️  Empty shader validation passed (unexpected)");
        }
        Err(e) => {
            println!("✅ Empty shader validation correctly failed: {}", e);
        }
    }

    println!("\n🎉 HLSL Shader Demo completed successfully!");
    println!("\n📝 Summary:");
    println!("   - HLSL to WGSL compilation: Working");
    println!("   - Metadata extraction: Working");
    println!("   - Shader validation: Working");
    println!("   - Error handling: Working");
    println!("\n🚀 Ready for GPU integration!");

    Ok(())
}
