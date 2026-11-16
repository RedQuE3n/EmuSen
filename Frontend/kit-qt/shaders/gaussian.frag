#version 440
// One pass of a separable Gaussian, along `step`, of deviation `sigma` texels, out to three deviations (EmuSen_Kit.md §7.118).
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    // One texel along the pass's axis, in texture coordinates.
    vec2 step;
    float sigma;
};
layout(binding = 1) uniform sampler2D source;

// The most taps either side: three deviations of the largest deviation the blur asks a pass for (GaussianBlur.qml, `texels`).
const int MOST = 24;

void main() {
    int reach = min(MOST, int(ceil(3.0 * sigma)));
    float twice = 2.0 * sigma * sigma;
    vec4 sum = texture(source, qt_TexCoord0);
    float total = 1.0;
    for (int i = 1; i <= MOST; ++i) {
        if (i > reach)
            break;
        float w = exp(-float(i * i) / twice);
        sum += w * (texture(source, qt_TexCoord0 + float(i) * step) + texture(source, qt_TexCoord0 - float(i) * step));
        total += 2.0 * w;
    }
    fragColor = sum / total * qt_Opacity;
}
