//! @title Waveform Visualization
//! @version 1.0.0
//! @author Sonic Flow
//! @description Real-time audio waveform visualization with advanced effects

// Uniform buffer for audio data
struct AudioVisualizationUniforms {
    float time;                    // Current time in seconds
    float sensitivity;             // Audio sensitivity multiplier
    float spectrum_data[128];      // Frequency spectrum data
    float audio_levels[4];         // Audio levels [L, R, LFE, C]
    float2 resolution;             // Screen resolution
    float2 mouse;                  // Mouse position (normalized)
};

// Vertex input
struct VertexInput {
    float2 position : POSITION;
    float2 tex_coord : TEXCOORD0;
};

// Vertex output
struct VertexOutput {
    float4 position : SV_POSITION;
    float2 tex_coord : TEXCOORD0;
    float2 screen_pos : TEXCOORD1;
};

// Bind group layout
[[group(0), binding(0)]]
var<uniform> uniforms : AudioVisualizationUniforms;

// Vertex shader
@vertex
fn vertexMain(input : VertexInput) -> VertexOutput {
    var output : VertexOutput;
    output.position = float4(input.position, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.screen_pos = input.position;
    return output;
}

// Fragment shader
@fragment
fn fragmentMain(input : VertexOutput) -> @location(0) float4 {
    let uv = input.tex_coord;
    let screen_pos = input.screen_pos;
    let time = uniforms.time;
    
    // Create waveform effect
    let waveform_center = 0.5;
    let waveform_height = 0.3;
    
    // Calculate waveform based on spectrum data
    let spectrum_index = int(uv.x * 128.0);
    let spectrum_value = uniforms.spectrum_data[spectrum_index] * uniforms.sensitivity;
    
    // Create multiple waveform layers
    let layer1 = sin(uv.x * 50.0 + time * 2.0) * spectrum_value * 0.1;
    let layer2 = sin(uv.x * 25.0 + time * 1.5) * spectrum_value * 0.15;
    let layer3 = sin(uv.x * 75.0 + time * 3.0) * spectrum_value * 0.05;
    
    let waveform = layer1 + layer2 + layer3;
    let waveform_y = waveform_center + waveform * waveform_height;
    
    // Distance from current pixel to waveform
    let distance_to_waveform = abs(uv.y - waveform_y);
    
    // Create glow effect
    let glow_intensity = exp(-distance_to_waveform * 20.0) * spectrum_value;
    let glow_color = float3(0.2, 0.8, 1.0) * glow_intensity;
    
    // Create ripple effect
    let ripple_distance = length(uv - float2(0.5, waveform_y));
    let ripple = sin(ripple_distance * 30.0 - time * 5.0) * 0.5 + 0.5;
    let ripple_color = float3(0.1, 0.4, 0.8) * ripple * spectrum_value * 0.3;
    
    // Background gradient
    let bg_gradient = mix(float3(0.02, 0.02, 0.05), float3(0.05, 0.05, 0.1), uv.y);
    
    // Combine effects
    let final_color = bg_gradient + glow_color + ripple_color;
    
    // Add subtle noise
    let noise = fract(sin(dot(screen_pos, float2(12.9898, 78.233))) * 43758.5453);
    let final_color_with_noise = final_color + (noise - 0.5) * 0.02;
    
    return float4(final_color_with_noise, 1.0);
}
