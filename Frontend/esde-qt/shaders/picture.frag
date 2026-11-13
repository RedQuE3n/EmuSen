#version 440
// Esde.Picture's one pass over a premultiplied texture: crop or tile, brightness, Rec. 601 saturation, tint, dim, rounded mask, ramp (EmuSen_Qt.md §4.8).

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec4 tint;
    vec4 tintEnd;
    float vertical;
    float saturation;
    float dim;
    float radius;
    vec2 size;
    vec4 uvRect;
    vec2 tiles;
    vec2 tileOrigin;
    float tiled;
    float fadeFrom;
    float fadeFalloff;
    float brightness;
};

layout(binding = 1) uniform sampler2D source;

void main() {
    vec2 cell = qt_TexCoord0 * tiles - tileOrigin;
    vec2 uv = uvRect.xy + (tiled > 0.5 ? fract(cell) : cell) * uvRect.zw;
    vec4 c = texture(source, uv);
    // THEMES.md brightness: 0.3 of a step added to each channel, before saturation and tint, on premultiplied colour.
    c.rgb += 0.3 * brightness * c.a;
    float luma = dot(c.rgb, vec3(0.299, 0.587, 0.114));
    c.rgb = mix(vec3(luma), c.rgb, saturation);
    vec4 t = mix(tint, tintEnd, vertical > 0.5 ? uv.y : uv.x);
    // Qt hands a color uniform over premultiplied (measured 2026-11-08: 80FF8000 read 64 levels when multiplied by its alpha again).
    c *= t;
    c.rgb *= dim;
    vec2 p = qt_TexCoord0 * size;
    vec2 half_size = size * 0.5;
    float r = min(radius, min(half_size.x, half_size.y));
    float d = length(max(abs(p - half_size) - (half_size - vec2(r)), 0.0)) - r;
    float mask = r > 0.0 ? clamp(0.5 - d, 0.0, 1.0) : 1.0;
    float ramp = fadeFrom * clamp(1.0 - fadeFalloff * qt_TexCoord0.y, 0.0, 1.0);
    fragColor = c * mask * ramp * qt_Opacity;
}
