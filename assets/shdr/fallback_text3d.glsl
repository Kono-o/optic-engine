//VERTEX

#version 450

layout (location = 0) in vec2 vPos;
layout (location = 1) in vec2 vUVM;

layout (location = 0) uniform mat4 uView;
layout (location = 1) uniform mat4 uProj;
layout (location = 2) uniform mat4 uTfm;

layout (location = 3) in vec2 iPos;
layout (location = 4) in vec3 iScale;
layout (location = 5) in vec4 iCol;
layout (location = 6) in vec4 iUVRect;
layout (location = 7) in uint iStyle;

layout (location = 0) out vec4 fCol;
layout (location = 1) out vec2 fUV;
layout (location = 2) flat out uint fStyle;

void main() {
    fCol = iCol;
    fStyle = iStyle;

    vec2 uv = vUVM;
    fUV = vec2(iUVRect.x + uv.x * (iUVRect.z - iUVRect.x),
               iUVRect.y + uv.y * (iUVRect.w - iUVRect.y));

    vec3 pos = vec3(vPos * iScale.xy + iPos, 0.0);

    vec4 world_pos = uTfm * vec4(pos, 1.0);
    gl_Position = uProj * uView * world_pos;
}

//FRAG

#version 450

layout (location = 0) in vec4 fCol;
layout (location = 1) in vec2 fUV;
layout (location = 2) flat in uint fStyle;

layout (location = 0) out vec4 fragPIXEL;

uniform sampler2D Tex0;
uniform float uSoftness;

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec3 msdf = texture(Tex0, fUV).rgb;
    float dist = median(msdf.r, msdf.g, msdf.b);
    float softness = uSoftness;

    if ((fStyle & 1u) != 0u) {
        dist -= 0.05;
    }

    float alpha = smoothstep(0.5 - softness, 0.5 + softness, dist);
    vec4 color = fCol;
    color.a *= alpha;
    fragPIXEL = color;
}
