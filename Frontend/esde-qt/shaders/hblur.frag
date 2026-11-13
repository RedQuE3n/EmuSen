#version 440
// The media viewer's video blur: a horizontal Gaussian of `sigma` pixels over the video's layer, out to three sigmas (EmuSen_Qt.md §4.14).

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float sigma;
    float texel;
};

layout(binding = 1) uniform sampler2D source;

void main() {
    int reach = int(ceil(3.0 * sigma));
    vec4 sum = vec4(0.0);
    float total = 0.0;
    for (int i = -reach; i <= reach; ++i) {
        float w = exp(-0.5 * float(i * i) / (sigma * sigma));
        sum += w * texture(source, qt_TexCoord0 + vec2(float(i) * texel, 0.0));
        total += w;
    }
    fragColor = sum / total * qt_Opacity;
}
