#version 440
// Single-pass rounded-corner texture sampler matching RoundPicture's fitted or cropped image.
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 boxSize;
    vec2 texSize;
    float radius;
};
layout(binding = 1) uniform sampler2D source;

void main() {
    vec2 p = qt_TexCoord0 * boxSize;
    vec2 uv = (p - (boxSize - texSize) * 0.5) / max(texSize, vec2(1.0));
    vec4 c = texture(source, uv);
    vec2 halfSize = boxSize * 0.5;
    float r = clamp(radius, 0.0, min(halfSize.x, halfSize.y));
    vec2 q = abs(p - halfSize) - (halfSize - vec2(r));
    float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
    float px = max(length(dFdx(p)), length(dFdy(p)));
    float aa = max(px, 0.5);
    float mask = r > 0.0 ? clamp(0.5 - d / aa, 0.0, 1.0) : 1.0;
    fragColor = c * (mask * qt_Opacity);
}
