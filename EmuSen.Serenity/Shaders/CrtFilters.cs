namespace EmuSen.Serenity.Shaders
{
    // CRT filters ported to SkSL; crt-lottes is Timothy Lottes' public-domain shader at its default settings - see EmuSen_Serenity.md §3.4.
    public static class CrtFilters
    {
        public const string LottesSksl = @"
uniform shader source;
uniform float2 inputSize;
uniform float2 outputSize;

const float hardScan = -8.0;
const float hardPix = -3.0;
const float warpX = 0.031;
const float warpY = 0.041;
const float maskDark = 0.5;
const float maskLight = 1.5;
const float brightBoost = 1.0;
const float hardBloomPix = -1.5;
const float hardBloomScan = -2.0;
const float bloomAmount = 0.15;
const float shape = 2.0;

float ToLinear1(float c) { return c <= 0.04045 ? c / 12.92 : pow((c + 0.055) / 1.055, 2.4); }
float3 ToLinear(float3 c) { return float3(ToLinear1(c.r), ToLinear1(c.g), ToLinear1(c.b)); }
float ToSrgb1(float c) { return c < 0.0031308 ? c * 12.92 : 1.055 * pow(c, 0.41666) - 0.055; }
float3 ToSrgb(float3 c) { return float3(ToSrgb1(c.r), ToSrgb1(c.g), ToSrgb1(c.b)); }

float3 Fetch(float2 pos, float2 off) {
    float2 texel = floor(pos * inputSize + off) + float2(0.5, 0.5);
    return ToLinear(brightBoost * source.eval(texel).rgb);
}

float2 Dist(float2 pos) {
    pos = pos * inputSize;
    return -((pos - floor(pos)) - float2(0.5, 0.5));
}

float Gaus(float pos, float scale) { return exp2(scale * pow(abs(pos), shape)); }

float3 Horz3(float2 pos, float off) {
    float3 b = Fetch(pos, float2(-1.0, off));
    float3 c = Fetch(pos, float2(0.0, off));
    float3 d = Fetch(pos, float2(1.0, off));
    float dst = Dist(pos).x;
    float wb = Gaus(dst - 1.0, hardPix);
    float wc = Gaus(dst + 0.0, hardPix);
    float wd = Gaus(dst + 1.0, hardPix);
    return (b * wb + c * wc + d * wd) / (wb + wc + wd);
}

float3 Horz5(float2 pos, float off) {
    float3 a = Fetch(pos, float2(-2.0, off));
    float3 b = Fetch(pos, float2(-1.0, off));
    float3 c = Fetch(pos, float2(0.0, off));
    float3 d = Fetch(pos, float2(1.0, off));
    float3 e = Fetch(pos, float2(2.0, off));
    float dst = Dist(pos).x;
    float wa = Gaus(dst - 2.0, hardPix);
    float wb = Gaus(dst - 1.0, hardPix);
    float wc = Gaus(dst + 0.0, hardPix);
    float wd = Gaus(dst + 1.0, hardPix);
    float we = Gaus(dst + 2.0, hardPix);
    return (a * wa + b * wb + c * wc + d * wd + e * we) / (wa + wb + wc + wd + we);
}

float3 Horz7(float2 pos, float off) {
    float3 a = Fetch(pos, float2(-3.0, off));
    float3 b = Fetch(pos, float2(-2.0, off));
    float3 c = Fetch(pos, float2(-1.0, off));
    float3 d = Fetch(pos, float2(0.0, off));
    float3 e = Fetch(pos, float2(1.0, off));
    float3 f = Fetch(pos, float2(2.0, off));
    float3 g = Fetch(pos, float2(3.0, off));
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

float Scan(float2 pos, float off) { return Gaus(Dist(pos).y + off, hardScan); }
float BloomScan(float2 pos, float off) { return Gaus(Dist(pos).y + off, hardBloomScan); }

float3 Tri(float2 pos) {
    return Horz3(pos, -1.0) * Scan(pos, -1.0) + Horz5(pos, 0.0) * Scan(pos, 0.0) + Horz3(pos, 1.0) * Scan(pos, 1.0);
}

float3 Bloom(float2 pos) {
    return Horz5(pos, -2.0) * BloomScan(pos, -2.0) + Horz7(pos, -1.0) * BloomScan(pos, -1.0) + Horz7(pos, 0.0) * BloomScan(pos, 0.0)
         + Horz7(pos, 1.0) * BloomScan(pos, 1.0) + Horz5(pos, 2.0) * BloomScan(pos, 2.0);
}

float2 Warp(float2 pos) {
    pos = pos * 2.0 - 1.0;
    pos *= float2(1.0 + (pos.y * pos.y) * warpX, 1.0 + (pos.x * pos.x) * warpY);
    return pos * 0.5 + 0.5;
}

float3 Mask(float2 pos) {
    float3 mask = float3(maskDark, maskDark, maskDark);
    pos.x += pos.y * 3.0;
    pos.x = fract(pos.x * 0.166666666);
    if (pos.x < 0.333) mask.r = maskLight;
    else if (pos.x < 0.666) mask.g = maskLight;
    else mask.b = maskLight;
    return mask;
}

half4 main(float2 coord) {
    float2 pos = Warp(coord / outputSize);
    float3 color = Tri(pos) + Bloom(pos) * bloomAmount;
    color *= Mask(coord);
    if (pos.x <= 0.0001 || pos.x >= 0.9999 || pos.y <= 0.0001 || pos.y >= 0.9999) color = float3(0.0, 0.0, 0.0);
    return half4(ToSrgb(clamp(color, 0.0, 1.0)), 1.0);
}
";

        public static ScreenFilter Lottes { get; } = new("CRT (Lottes)",
            new[] { new FilterPass(LottesSksl, PassScale.Viewport) },
            new[] { "NES", "SNES", "N64" },
            "crt-lottes by Timothy Lottes, public domain, from libretro's slang-shaders");
    }
}
