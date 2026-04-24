//! @title Particle System Visualization
//! @version 1.0.0
//! @author Sonic Flow
//! @description Real-time audio particle system with physics simulation

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
    
    // Create particle system
    let particle_count = 50.0;
    let particle_size = 0.02;
    var final_color = float3(0.0, 0.0, 0.0);
    
    // Generate particles based on audio data
    for (var i = 0.0; i < particle_count; i += 1.0) {
        // Particle position based on time and audio
        let particle_id = i / particle_count;
        let spectrum_index = int(particle_id * 128.0);
        let spectrum_value = uniforms.spectrum_data[spectrum_index] * uniforms.sensitivity;
        
        // Animated particle position
        let particle_x = fract(particle_id + time * 0.5);
        let particle_y = fract(sin(particle_id * 10.0 + time * 2.0) * 0.5 + 0.5);
        
        // Add audio-driven movement
        let audio_offset = spectrum_value * 0.2;
        let particle_pos = float2(particle_x, particle_y + audio_offset);
        
        // Distance to particle
        let distance = length(uv - particle_pos);
        
        // Particle color based on frequency
        let frequency_factor = particle_id;
        let particle_color = mix(float3(1.0, 0.2, 0.5), float3(0.2, 0.8, 1.0), frequency_factor);
        
        // Particle intensity based on audio
        let particle_intensity = spectrum_value * exp(-distance / particle_size);
        
        // Add particle to final color
        final_color += particle_color * particle_intensity;
    }
    
    // Create audio-reactive background
    let audio_sum = uniforms.audio_levels[0] + uniforms.audio_levels[1];
    let bg_intensity = audio_sum * 0.1;
    let bg_color = float3(0.05, 0.05, 0.1) * (1.0 + bg_intensity);
    
    // Add audio waves
    let wave_center = float2(0.5, 0.5);
    let wave_distance = length(uv - wave_center);
    let wave_frequency = audio_sum * 20.0 + 10.0;
    let wave = sin(wave_distance * wave_frequency - time * 3.0) * 0.5 + 0.5;
    let wave_color = float3(0.1, 0.3, 0.6) * wave * audio_sum * 0.5;
    
    // Combine all effects
    let final_result = bg_color + final_color + wave_color;
    
    // Add subtle noise
    let noise = fract(sin(dot(screen_pos, float2(12.9898, 78.233))) * 43758.5453);
    let final_with_noise = final_result + (noise - 0.5) * 0.01;
    
    return float4(final_with_noise, 1.0);
}
