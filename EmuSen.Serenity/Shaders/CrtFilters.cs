using System.Collections.Generic;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // CRT filters; crt-lottes is Timothy Lottes' public-domain shader at its default settings, in Vulkan GLSL - see EmuSen_Serenity.md §3.4.
    public static class CrtFilters
    {
        public const string LottesGlsl = @"
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
";

        // crt-lottes.slang's own #pragma parameter lines for the eleven the port kept, labelled for a person; above Lottes, which reads it as it is built - see EmuSen_Serenity.md §3.7.
        public static IReadOnlyList<SlangParameter> LottesParameters { get; } = new SlangParameter[]
        {
            new("hardScan", "Scanline hardness", -8f, -20f, 0f, 1f),
            new("hardPix", "Pixel hardness", -3f, -20f, 0f, 1f),
            new("warpX", "Curvature, horizontal", 0.031f, 0f, 0.125f, 0.01f),
            new("warpY", "Curvature, vertical", 0.041f, 0f, 0.125f, 0.01f),
            new("maskDark", "Mask dark", 0.5f, 0f, 2f, 0.1f),
            new("maskLight", "Mask light", 1.5f, 0f, 2f, 0.1f),
            new("brightBoost", "Brightness boost", 1f, 0f, 2f, 0.05f),
            new("hardBloomPix", "Bloom softness, horizontal", -1.5f, -2f, -0.5f, 0.1f),
            new("hardBloomScan", "Bloom softness, vertical", -2f, -4f, -1f, 0.1f),
            new("bloomAmount", "Bloom amount", 0.15f, 0f, 1f, 0.05f),
            new("shape", "Filter kernel shape", 2f, 0f, 10f, 0.05f),
        };

        public static ScreenFilter Lottes { get; } = new("CRT (Lottes)",
            new[] { new FilterPass(LottesGlsl, PassScale.Viewport) },
            CrtFilter.TelevisionConsoles,
            "crt-lottes by Timothy Lottes, public domain, from libretro's slang-shaders",
            LottesParameters);
    }
}
