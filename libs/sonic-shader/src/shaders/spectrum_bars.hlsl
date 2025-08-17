//! @title Spectrum Bars Visualization
//! @version 1.0.0
//! @author Sonic Flow
//! @description Real-time audio spectrum visualization with animated bars

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
    
    // Calculate bar index based on X position
    let bar_count = 64.0;
    let bar_width = 1.0 / bar_count;
    let bar_index = floor(uv.x / bar_width);
    let bar_uv_x = fract(uv.x / bar_width);
    
    // Get spectrum data for this bar
    let spectrum_index = min(int(bar_index), 127);
    let spectrum_value = uniforms.spectrum_data[spectrum_index] * uniforms.sensitivity;
    
    // Calculate bar height based on spectrum value
    let bar_height = spectrum_value * 0.8; // Scale to 80% of screen height
    let bar_y = uv.y;
    
    // Create animated bar effect
    let time = uniforms.time;
    let animation_speed = 2.0;
    let wave_offset = sin(time * animation_speed + bar_index * 0.1) * 0.1;
    
    // Bar color based on frequency and intensity
    let frequency_factor = bar_index / bar_count;
    let intensity = spectrum_value;
    
    // Color gradient from low (blue) to high (red) frequencies
    let color_low = float3(0.0, 0.5, 1.0);   // Blue
    let color_mid = float3(0.0, 1.0, 0.5);   // Green
    let color_high = float3(1.0, 0.0, 0.0);  // Red
    
    var bar_color : float3;
    if (frequency_factor < 0.5) {
        bar_color = mix(color_low, color_mid, frequency_factor * 2.0);
    } else {
        bar_color = mix(color_mid, color_high, (frequency_factor - 0.5) * 2.0);
    }
    
    // Add brightness based on intensity
    bar_color *= (0.5 + intensity * 0.5);
    
    // Create bar shape
    let bar_threshold = bar_height + wave_offset;
    let bar_alpha = step(bar_y, bar_threshold);
    
    // Add glow effect
    let glow_distance = abs(bar_y - bar_threshold);
    let glow_intensity = exp(-glow_distance * 10.0) * intensity;
    let glow_color = bar_color * glow_intensity;
    
    // Background gradient
    let bg_gradient = mix(float3(0.05, 0.05, 0.1), float3(0.1, 0.1, 0.2), uv.y);
    
    // Combine bar and background
    let final_color = mix(bg_gradient, bar_color + glow_color, bar_alpha);
    
    // Add subtle noise for texture
    let noise = fract(sin(dot(screen_pos, float2(12.9898, 78.233))) * 43758.5453);
    let noise_factor = 0.02;
    let final_color_with_noise = final_color + (noise - 0.5) * noise_factor;
    
    return float4(final_color_with_noise, 1.0);
}
