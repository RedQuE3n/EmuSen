#version 440
// EdgeFade on the GPU: the layer multiplied by each faded edge's ramp, none of it at the edge and all of it `reach` in (EmuSen_Kit.md §7.35).
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    // The ramps' lengths at the top, bottom, left and right, as fractions of the item's height or width; 0 where an edge does not fade.
    vec4 reach;
};
layout(binding = 1) uniform sampler2D source;

float ramp(float distance, float length) {
    return length > 0.0 ? clamp(distance / length, 0.0, 1.0) : 1.0;
}

void main() {
    vec2 t = qt_TexCoord0;
    float kept = ramp(t.y, reach.x) * ramp(1.0 - t.y, reach.y) * ramp(t.x, reach.z) * ramp(1.0 - t.x, reach.w);
    fragColor = texture(source, t) * (kept * qt_Opacity);
}
