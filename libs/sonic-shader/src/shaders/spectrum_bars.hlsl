//! @title Spectrum Bars Visualizer
//! @description Real-time audio spectrum visualization with animated bars (HLSL with future Slang compatibility)
//! @author Sonic Flow Team
//! @version 1.0.0

// Vertex shader input structure
struct VertexInput {
    float2 position : POSITION;
    float2 texCoord : TEXCOORD0;
};

// Vertex shader output structure
struct VertexOutput {
    float4 position : SV_Position;
    float2 texCoord : TEXCOORD0;
};

// Constant buffer for uniforms
cbuffer AudioVisualizationUniforms : register(b0) {
    float4x4 modelViewProjection;
    float time;
    float sensitivity;
    float4 colorScheme;
    float spectrumData[128];
    float4 audioLevels;
    float effectParams[8];
};

// Vertex shader
VertexOutput vertexMain(VertexInput input) {
    VertexOutput output;
    output.position = mul(modelViewProjection, float4(input.position, 0.0, 1.0));
    output.texCoord = input.texCoord;
    return output;
}

// Fragment shader
float4 fragmentMain(VertexOutput input) : SV_Target {
    // Calculate bar index based on texture coordinate
    float barIndex = input.texCoord.x * 128.0;
    int index = int(barIndex);
    
    // Get spectrum data for this bar
    float spectrumValue = spectrumData[index];
    
    // Apply sensitivity and time-based animation
    float animatedValue = spectrumValue * sensitivity * (1.0 + 0.1 * sin(time * 2.0 + barIndex * 0.1));
    
    // Calculate bar height
    float barHeight = animatedValue * input.texCoord.y;
    
    // Create color based on spectrum value and position
    float3 color = colorScheme.rgb * (0.5 + 0.5 * animatedValue);
    color += float3(0.1, 0.2, 0.3) * sin(time + barIndex * 0.2);
    
    // Apply fade effect
    float alpha = smoothstep(0.0, 0.1, barHeight) * smoothstep(1.0, 0.9, barHeight);
    
    return float4(color, alpha * colorScheme.a);
}
