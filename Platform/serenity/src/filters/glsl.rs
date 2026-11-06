//! The built-in filters' fixed GLSL, copied from the passes they were first written as. See EmuSen_Serenity.md §3.

/// crt-lottes by Timothy Lottes, public domain, from libretro's slang-shaders, at its default settings.
pub const LOTTES: &str = r#"
uniform sampler2D source;
uniform vec2 inputSize;
uniform vec2 outputSize;

uniform float hardScan;
uniform float hardPix;
uniform float warpX;
uniform float warpY;
uniform float maskDark;
uniform float maskLight;
uniform float brightBoost;
uniform float hardBloomPix;
uniform float hardBloomScan;
uniform float bloomAmount;
uniform float shape;

float ToLinear1(float c) { return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4); }
vec3 ToLinear(vec3 c) { return vec3(ToLinear1(c.r), ToLinear1(c.g), ToLinear1(c.b)); }
float ToSrgb1(float c) { return c < 0.0031308 ? c * 12.92 : 1.055 * pow(c, 0.41666) - 0.055; }
vec3 ToSrgb(vec3 c) { return vec3(ToSrgb1(c.r), ToSrgb1(c.g), ToSrgb1(c.b)); }

vec3 Fetch(vec2 pos, vec2 off) {
    vec2 texel = floor(pos * inputSize + off) + vec2(0.5, 0.5);
    return ToLinear(brightBoost * source_eval(texel).rgb);
}

vec2 Dist(vec2 pos) {
    pos = pos * inputSize;
    return -((pos - floor(pos)) - vec2(0.5, 0.5));
}

float Gaus(float pos, float scale) { return exp2(scale * pow(abs(pos), shape)); }

vec3 Horz3(vec2 pos, float off) {
    vec3 b = Fetch(pos, vec2(-1.0, off));
    vec3 c = Fetch(pos, vec2(0.0, off));
    vec3 d = Fetch(pos, vec2(1.0, off));
    float dst = Dist(pos).x;
    float wb = Gaus(dst - 1.0, hardPix);
    float wc = Gaus(dst + 0.0, hardPix);
    float wd = Gaus(dst + 1.0, hardPix);
    return (b * wb + c * wc + d * wd) / (wb + wc + wd);
}

vec3 Horz5(vec2 pos, float off) {
    vec3 a = Fetch(pos, vec2(-2.0, off));
    vec3 b = Fetch(pos, vec2(-1.0, off));
    vec3 c = Fetch(pos, vec2(0.0, off));
    vec3 d = Fetch(pos, vec2(1.0, off));
    vec3 e = Fetch(pos, vec2(2.0, off));
    float dst = Dist(pos).x;
    float wa = Gaus(dst - 2.0, hardPix);
    float wb = Gaus(dst - 1.0, hardPix);
    float wc = Gaus(dst + 0.0, hardPix);
    float wd = Gaus(dst + 1.0, hardPix);
    float we = Gaus(dst + 2.0, hardPix);
    return (a * wa + b * wb + c * wc + d * wd + e * we) / (wa + wb + wc + wd + we);
}

vec3 Horz7(vec2 pos, float off) {
    vec3 a = Fetch(pos, vec2(-3.0, off));
    vec3 b = Fetch(pos, vec2(-2.0, off));
    vec3 c = Fetch(pos, vec2(-1.0, off));
    vec3 d = Fetch(pos, vec2(0.0, off));
    vec3 e = Fetch(pos, vec2(1.0, off));
    vec3 f = Fetch(pos, vec2(2.0, off));
    vec3 g = Fetch(pos, vec2(3.0, off));
    float dst = Dist(pos).x;
    float wa = Gaus(dst - 3.0, hardBloomPix);
    float wb = Gaus(dst - 2.0, hardBloomPix);
    float wc = Gaus(dst - 1.0, hardBloomPix);
    float wd = Gaus(dst + 0.0, hardBloomPix);
    float we = Gaus(dst + 1.0, hardBloomPix);
    float wf = Gaus(dst + 2.0, hardBloomPix);
    float wg = Gaus(dst + 3.0, hardBloomPix);
    return (a * wa + b * wb + c * wc + d * wd + e * we + f * wf + g * wg) / (wa + wb + wc + wd + we + wf + wg);
}

float Scan(vec2 pos, float off) { return Gaus(Dist(pos).y + off, hardScan); }
float BloomScan(vec2 pos, float off) { return Gaus(Dist(pos).y + off, hardBloomScan); }

vec3 Tri(vec2 pos) {
    return Horz3(pos, -1.0) * Scan(pos, -1.0) + Horz5(pos, 0.0) * Scan(pos, 0.0) + Horz3(pos, 1.0) * Scan(pos, 1.0);
}

vec3 Bloom(vec2 pos) {
    return Horz5(pos, -2.0) * BloomScan(pos, -2.0) + Horz7(pos, -1.0) * BloomScan(pos, -1.0) + Horz7(pos, 0.0) * BloomScan(pos, 0.0)
         + Horz7(pos, 1.0) * BloomScan(pos, 1.0) + Horz5(pos, 2.0) * BloomScan(pos, 2.0);
}

vec2 Warp(vec2 pos) {
    pos = pos * 2.0 - 1.0;
    pos *= vec2(1.0 + (pos.y * pos.y) * warpX, 1.0 + (pos.x * pos.x) * warpY);
    return pos * 0.5 + 0.5;
}

vec3 Mask(vec2 pos) {
    vec3 mask = vec3(maskDark, maskDark, maskDark);
    pos.x += pos.y * 3.0;
    pos.x = fract(pos.x * 0.166666666);
    if (pos.x < 0.333) mask.r = maskLight;
    else if (pos.x < 0.666) mask.g = maskLight;
    else mask.b = maskLight;
    return mask;
}

vec4 filter_main(vec2 coord) {
    vec2 pos = Warp(coord / outputSize);
    vec3 color = Tri(pos) + Bloom(pos) * bloomAmount;
    color *= Mask(coord);
    if (pos.x <= 0.0001 || pos.x >= 0.9999 || pos.y <= 0.0001 || pos.y >= 0.9999) color = vec3(0.0, 0.0, 0.0);
    return vec4(ToSrgb(clamp(color, 0.0, 1.0)), 1.0);
}
"#;

/// A monochrome panel's four shades and slow response; P0 to P3 are its shades.
pub const MONOCHROME_PANEL: &str = r#"
uniform sampler2D original;
uniform sampler2D history1;
uniform sampler2D history2;
uniform sampler2D history3;

uniform float response;
const vec3 p0 = P0;
const vec3 p1 = P1;
const vec3 p2 = P2;
const vec3 p3 = P3;

float Shade(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

vec3 Palette(float v) {
    float i = clamp((1.0 - v) * 3.0, 0.0, 3.0);
    if (i < 1.0) return mix(p0, p1, i);
    if (i < 2.0) return mix(p1, p2, i - 1.0);
    return mix(p2, p3, i - 2.0);
}

vec4 filter_main(vec2 coord) {
    float v = Shade(original_eval(coord).rgb);
    v += (Shade(history1_eval(coord).rgb) - v) * response;
    v += (Shade(history2_eval(coord).rgb) - v) * response * response;
    v += (Shade(history3_eval(coord).rgb) - v) * response * response * response;
    return vec4(Palette(v), 1.0);
}
"#;

/// Dots with gaps and their shadows; BG and COVER are the reflector and the dots' share.
pub const DOT_MATRIX: &str = r#"
uniform sampler2D source;
uniform vec2 inputSize;
uniform vec2 outputSize;

const vec3 background = BG;
const float coverage = COVER;
uniform float shadowOpacity;
const vec2 shadowOffset = vec2(0.6, 0.8);

float Cover(vec2 t, float aa) {
    vec2 f = t - floor(t);
    vec2 d = abs(f - 0.5) * 2.0;
    vec2 c = 1.0 - smoothstep(coverage - aa, coverage + aa, d);
    return c.x * c.y;
}

float Luma(vec3 c) { return dot(c, vec3(0.299, 0.587, 0.114)); }

vec4 filter_main(vec2 coord) {
    vec2 t = coord / outputSize * inputSize;
    float perTexel = outputSize.x / inputSize.x;
    float grid = smoothstep(1.5, 3.0, perTexel);
    float aa = 2.0 / perTexel;
    vec3 lit = source_eval(floor(t) + 0.5).rgb;
    float cover = mix(1.0, Cover(t, aa), grid);
    vec3 color = mix(background, lit, cover);
    vec2 s = t - shadowOffset;
    vec3 caster = source_eval(floor(s) + 0.5).rgb;
    float darkness = clamp(1.0 - Luma(caster) / Luma(background), 0.0, 1.0);
    float shadow = darkness * Cover(s, aa) * shadowOpacity * grid;
    return vec4(color * (1.0 - shadow), 1.0);
}
"#;

/// A colour panel's response and matrix; LUM and M00 to M22.
pub const COLOUR_PANEL: &str = r#"
uniform sampler2D original;
uniform sampler2D history1;

uniform float response;
const float lum = LUM;

vec4 filter_main(vec2 coord) {
    vec3 c = mix(original_eval(coord).rgb, history1_eval(coord).rgb, response);
    vec3 l = pow(c, vec3(2.2, 2.2, 2.2));
    vec3 o = vec3(M00 * l.r + M01 * l.g + M02 * l.b,
                      M10 * l.r + M11 * l.g + M12 * l.b,
                      M20 * l.r + M21 * l.g + M22 * l.b) * lum;
    return vec4(pow(clamp(o, 0.0, 1.0), vec3(1.0 / 2.2, 1.0 / 2.2, 1.0 / 2.2)), 1.0);
}
"#;

/// Stripes inside each pixel; FLOOR, GAIN, GAP and ROWDIM.
pub const STRIPE_GRID: &str = r#"
uniform sampler2D source;
uniform vec2 inputSize;
uniform vec2 outputSize;

const float lowest = FLOOR;
const float gain = GAIN;
const float gap = GAP;
const float rowDim = ROWDIM;

float Box(float x, float a, float b, float aa) { return smoothstep(a - aa, a + aa, x) - smoothstep(b - aa, b + aa, x); }

vec4 filter_main(vec2 coord) {
    vec2 t = coord / outputSize * inputSize;
    float perTexel = outputSize.x / inputSize.x;
    float grid = smoothstep(2.0, 4.0, perTexel);
    float aa = 1.0 / perTexel;
    vec2 f = t - floor(t);
    float w = (1.0 - gap) / 3.0;
    vec3 stripe = vec3(Box(f.x, 0.0, w, aa), Box(f.x, w, 2.0 * w, aa), Box(f.x, 2.0 * w, 3.0 * w, aa));
    vec3 mask = lowest + (1.0 - lowest) * stripe;
    mask *= mix(1.0, rowDim, smoothstep(0.88 - aa, 0.88 + aa, f.y));
    vec3 lit = source_eval(floor(t) + 0.5).rgb;
    return vec4(clamp(lit * mix(vec3(1.0, 1.0, 1.0), mask * gain, grid), 0.0, 1.0), 1.0);
}
"#;

/// Every other output row darkened.
pub const SCANLINES: &str = r#"
uniform sampler2D image;
uniform vec2 outputSize;

vec4 filter_main(vec2 coord) {
    vec4 texColor = image_eval(coord);

    // Darkens every other output-resolution row, so it stays crisp at any window size.
    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.78;

    return vec4(texColor.rgb * scanline, texColor.a);
}
"#;

/// Scanlines with a vignette.
pub const SIMPLE_CRT: &str = r#"
uniform sampler2D image;
uniform vec2 outputSize;

vec4 filter_main(vec2 coord) {
    vec4 texColor = image_eval(coord);

    float row = floor(coord.y);
    float scanline = mod(row, 2.0) < 1.0 ? 1.0 : 0.72;

    // Soft vignette - darkens toward the corners like a curved CRT tube.
    vec2 centered = (coord / outputSize) - 0.5;
    float vignette = 1.0 - dot(centered, centered) * 0.55;

    return vec4(texColor.rgb * scanline * vignette, texColor.a);
}
"#;

/// The picture drawn into its rectangle with nothing over it.
pub const PLAIN: &str = r#"
uniform sampler2D image;

vec4 filter_main(vec2 coord) {
    return image_eval(coord);
}
"#;

/// A frame drawn at a whole multiple of the picture, brought back to it.
pub const REDUCE: &str = r#"
uniform sampler2D source;
uniform float by;
vec4 filter_main(vec2 coord) {
    vec2 corner = floor(coord) * by;
    vec4 sum = vec4(0.0);
    for (int j = 0; j < 8; j++) {
        for (int i = 0; i < 8; i++) {
            if (float(i) < by && float(j) < by) sum += source_eval(corner + vec2(float(i) + 0.5, float(j) + 0.5));
        }
    }
    return vec4(sum / (by * by));
}"#;

/// Abramowitz and Stegun 7.1.26, and the normal distribution's integral from it.
pub const FUNCTIONS: &str = r#"
float erf1(float x) {
    float t = 1.0 / (1.0 + 0.3275911 * abs(x));
    float y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * exp(-x * x);
    return x < 0.0 ? -y : y;
}
float cdf(float x) { return 0.5 + 0.5 * erf1(x * 0.70710678); }
vec3 cdf3(vec3 x) { return vec3(cdf(x.r), cdf(x.g), cdf(x.b)); }
"#;

/// The subcarrier's phase at a sample, and PAL's switch.
pub const PHASE: &str = r#"
uniform float frameCount;
float phaseAt(float index, float row, float samples) {
    return fract((index + 0.5) / samples * cyclesAcross + fract(row * phaseLine) + mod(frameCount, 2.0) * phaseFrame + fract(mod(frameCount, 1024.0) * phaseCrawl));
}
float palSwitch(float row) {
    return mod(row + mod(frameCount, 2.0) * switchFrame, 2.0) < 0.5 ? 1.0 : -1.0;
}
"#;

/// The share of a footprint a periodic lit run covers.
pub const COVERAGE: &str = r#"
float runIntegral(float x, float lit, float period) {
    float n = floor(x / period);
    float f = x - n * period;
    float whole = lit * period * n * (n - 1.0) * 0.5 + n * (lit * period - 0.5 * lit * lit);
    float part = n * lit * f + (f < lit ? 0.5 * f * f : lit * (f - 0.5 * lit));
    return whole + part;
}
float runLength(float x, float lit, float period) {
    float n = floor(x / period);
    return n * lit + min(x - n * period, lit);
}
float boxed(float x, float reach, float start, float lit, float period) {
    float base = floor((x - reach - start) / period) * period + start;
    float u = x - base;
    float h = max(reach, 0.0001);
    return (runLength(u + h, lit, period) - runLength(u - h, lit, period)) / (2.0 * h);
}
float covered(float x, float reach, float start, float lit, float period) {
    float base = floor((x - reach - start) / period) * period + start;
    float u = x - base;
    float h = max(reach, 0.0001);
    return (runIntegral(u + h, lit, period) - 2.0 * runIntegral(u, lit, period) + runIntegral(u - h, lit, period)) / (h * h);
}
"#;

/// Every pass's vertex stage, and the fragment stage's head.
pub const STAGES: &str = r#"
#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;
void main()
{
    gl_Position = MVP * Position;
    vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
"#;

