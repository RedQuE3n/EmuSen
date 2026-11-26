#version 440
// Single-pass analytical SDF Gaussian box-shadow matching CSS outer box-shadow.
layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;
layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec4 color;
    vec2 boxSize;
    vec2 offset;
    float radius;
    float spread;
    float blur;
    float reach;
    float pixelScale;
};

float erfApprox(float x) {
    float s = sign(x);
    float a = abs(x);
    float t = 1.0 / (1.0 + 0.3275911 * a);
    float y = 1.0 - t * (0.254829592 + t * (-0.284496736 + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429)))) * exp(-a * a);
    return s * y;
}

void main() {
    vec2 fullSize = boxSize + vec2(2.0 * reach);
    vec2 p = qt_TexCoord0 * fullSize - (boxSize * 0.5 + vec2(reach));
    float pxStep = max(length(dFdx(p)), length(dFdy(p)));
    vec2 halfIn = boxSize * 0.5;
    float rIn = clamp(radius, 0.0, min(halfIn.x, halfIn.y));
    vec2 qIn = abs(p) - (halfIn - vec2(rIn));
    float dIn = length(max(qIn, 0.0)) + min(max(qIn.x, qIn.y), 0.0) - rIn;
    if (dIn <= 0.0) {
        fragColor = vec4(0.0);
        return;
    }
    vec2 halfOut = halfIn + vec2(spread);
    if (halfOut.x <= 0.0 || halfOut.y <= 0.0) {
        fragColor = vec4(0.0);
        return;
    }
    float t = rIn < spread ? (rIn / spread - 1.0) : 0.0;
    float rOut = spread > 0.0 ? rIn + spread * (1.0 + t * t * t) : max(0.0, rIn + spread);
    rOut = clamp(rOut, 0.0, min(halfOut.x, halfOut.y));
    vec2 sp = p - offset;
    float alpha = 0.0;
    if (blur <= 0.0) {
        vec2 qOut = abs(sp) - (halfOut - vec2(rOut));
        float dOut = length(max(qOut, 0.0)) + min(max(qOut.x, qOut.y), 0.0) - rOut;
        float aa = pxStep > 0.0 ? pxStep : 1.0 / max(pixelScale, 1e-3);
        alpha = clamp(0.5 - dOut / aa, 0.0, 1.0);
    } else {
        float px = sp.x;
        float py = sp.y;
        float hw = halfOut.x;
        float hh = halfOut.y;
        if (abs(py) - hh > abs(px) - hw) {
            px = sp.y;
            py = sp.x;
            hw = halfOut.y;
            hh = halfOut.x;
        }
        float sigma = blur * 0.5;
        float low = max(-hh, py - 4.0 * sigma);
        float high = min(hh, py + 4.0 * sigma);
        if (low < high) {
            const int STEPS = 8;
            float step = (high - low) / float(STEPS);
            float invSqrt2S = 1.0 / (1.41421356237 * sigma);
            float straight = hh - rOut;
            for (int i = 0; i < STEPS; ++i) {
                float ya = low + float(i) * step;
                float yb = ya + step;
                float ymid = 0.5 * (ya + yb);
                float massY = 0.5 * (erfApprox((yb - py) * invSqrt2S) - erfApprox((ya - py) * invSqrt2S));
                float ay = abs(ymid);
                float xspan = hw;
                if (ay > straight) {
                    float dy = ay - straight;
                    xspan = (hw - rOut) + sqrt(max(0.0, rOut * rOut - dy * dy));
                }
                float massX = 0.5 * (erfApprox((xspan - px) * invSqrt2S) + erfApprox((xspan + px) * invSqrt2S));
                alpha += massY * massX;
            }
        }
    }
    fragColor = color * (alpha * qt_Opacity);
}
