//! @title Simple Test Shader
//! @description Basic test shader for HLSL compilation (HLSL with future Slang compatibility)
//! @author Sonic Flow Team
//! @version 1.0.0

// Vertex shader input structure
struct VertexInput {
    float2 position : POSITION;
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
    output.texCoord = input.position * 0.5 + 0.5; // Convert to 0-1 range
    return output;
}

// Fragment shader
float4 fragmentMain(VertexOutput input) : SV_Target {
    // Simple animated color based on time and position
    float3 color = float3(
        0.5 + 0.5 * sin(time + input.texCoord.x * 3.14159),
        0.5 + 0.5 * sin(time * 1.5 + input.texCoord.y * 3.14159),
        0.5 + 0.5 * sin(time * 0.7 + (input.texCoord.x + input.texCoord.y) * 3.14159)
    );
    
    return float4(color, 1.0);
}
