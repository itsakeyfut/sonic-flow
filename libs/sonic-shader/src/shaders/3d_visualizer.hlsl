//! @title 3D Audio Visualizer
//! @version 1.0.0
//! @author Sonic Flow
//! @description Real-time 3D audio visualization with depth and perspective

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
    
    // Create 3D coordinate system
    let center = float2(0.5, 0.5);
    let offset = uv - center;
    
    // Calculate 3D position
    let x = offset.x * 2.0;
    let y = offset.y * 2.0;
    let z = 0.0; // We'll calculate this based on audio
    
    // Create 3D grid
    let grid_size = 20.0;
    let grid_x = fract(x * grid_size);
    let grid_y = fract(y * grid_size);
    let grid_z = fract(z * grid_size);
    
    // Audio-reactive 3D displacement
    let spectrum_index = int((x + 1.0) * 0.5 * 128.0);
    let spectrum_value = uniforms.spectrum_data[spectrum_index] * uniforms.sensitivity;
    
    // Create 3D height field
    let height = spectrum_value * 0.5;
    let depth = sin(x * 10.0 + time) * cos(y * 10.0 + time) * spectrum_value * 0.3;
    
    // 3D lighting calculation
    let light_pos = float3(sin(time) * 2.0, cos(time) * 2.0, 3.0);
    let surface_pos = float3(x, y, height + depth);
    let light_dir = normalize(light_pos - surface_pos);
    let normal = normalize(float3(-height, -depth, 1.0));
    
    // Diffuse lighting
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Specular lighting
    let view_dir = normalize(float3(0.0, 0.0, 1.0) - surface_pos);
    let reflect_dir = reflect(-light_dir, normal);
    let specular = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    
    // Ambient lighting
    let ambient = 0.1;
    
    // Color based on frequency and intensity
    let frequency_factor = (x + 1.0) * 0.5;
    let base_color = mix(float3(0.2, 0.5, 1.0), float3(1.0, 0.3, 0.7), frequency_factor);
    let intensity = spectrum_value;
    let color = base_color * intensity;
    
    // Apply lighting
    let lit_color = color * (ambient + diffuse) + float3(1.0, 1.0, 1.0) * specular * 0.5;
    
    // Add grid lines
    let grid_threshold = 0.1;
    let grid_line = step(grid_threshold, grid_x) * step(grid_threshold, grid_y);
    let grid_color = float3(0.3, 0.3, 0.3) * grid_line * 0.3;
    
    // Add audio-reactive background
    let audio_sum = uniforms.audio_levels[0] + uniforms.audio_levels[1];
    let bg_color = float3(0.05, 0.05, 0.1) * (1.0 + audio_sum * 0.5);
    
    // Combine all effects
    let final_color = bg_color + lit_color + grid_color;
    
    // Add depth fog
    let fog_factor = exp(-depth * 2.0);
    let final_with_fog = mix(float3(0.1, 0.1, 0.2), final_color, fog_factor);
    
    // Add subtle noise
    let noise = fract(sin(dot(screen_pos, float2(12.9898, 78.233))) * 43758.5453);
    let final_with_noise = final_with_fog + (noise - 0.5) * 0.02;
    
    return float4(final_with_noise, 1.0);
}
