//VERTEX

#version 450

layout (location = 0) in vec2 vPos;
layout (location = 1) in vec2 vUVM;

layout (location = 0) uniform mat4 uProj;
layout (location = 1) uniform uint uLayer;

layout (location = 2) in vec2 iPos;
layout (location = 3) in vec2 iScale;
layout (location = 4) in vec4 iCol;
layout (location = 5) in vec4 iUVRect;
layout (location = 6) in uint iStyle;

layout (location = 0) out vec4 fCol;
layout (location = 1) out vec2 fUV;
layout (location = 2) flat out uint fStyle;

const float FAUX_ITALIC_SKEW = 0.25;

void main() {
    fCol = iCol;
    fStyle = iStyle;

    vec2 uv = vUVM;
    fUV = vec2(iUVRect.x + uv.x * (iUVRect.z - iUVRect.x),
               iUVRect.y + uv.y * (iUVRect.w - iUVRect.y));

    vec2 pos = vPos * iScale + iPos;

    if ((iStyle & 2u) != 0u) {
        pos.x += pos.y * FAUX_ITALIC_SKEW;
    }

    gl_Position = uProj * vec4(pos, uLayer * 0.001, 1.0);
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

vec4 apply_border(vec4 color, float dist, float softness) {
    float border_width = 0.1;
    float border_dist = smoothstep(0.5 - border_width - softness, 0.5 - border_width + softness, dist);
    float border_alpha = border_dist * (1.0 - smoothstep(0.5 - softness, 0.5 + softness, dist));
    vec4 border_col = vec4(0.0, 0.0, 0.0, 1.0);
    return mix(color, border_col, border_alpha);
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
    if ((fStyle & 4u) != 0u) {
        color = apply_border(color, dist, softness);
    }

    color.a *= alpha;
    fragPIXEL = color;
}
