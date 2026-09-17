using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The two-cycle mode: two combiner cycles and two blender cycles per pixel, pipelined so each pixel's first cycles run before its predecessor's last - see Mars_RdpTwoCycle.md.
    public sealed partial class Rdp
    {
        private const int TwoCycle = 1;

        // The first blender cycle's result, which the second cycle's first input reads.
        private Color _blended;
        private int _pastShiftA;
        private int _pastShiftB;

        // The last depth test's stored slope, which the next pixel's first blend shifts by - see §4.
        private int _pastStoredEncoded;

        private readonly struct TwoCyclePixel
        {
            public readonly int Coverage, CompareAlpha, DitherColor;
            public readonly bool CoverageBit;
            public readonly (byte X, byte Y) Offset;

            public TwoCyclePixel(int coverage, bool coverageBit, (byte X, byte Y) offset, int compareAlpha, int ditherColor) =>
                (Coverage, CoverageBit, Offset, CompareAlpha, DitherColor) = (coverage, coverageBit, offset, compareAlpha, ditherColor);
        }

        // A pixel's first combiner cycle runs before its predecessor's last blender cycle, which then reads this pixel's shade alpha - see §1.
        private void DrawTwoCycle((int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            int deltaZ = PrimitiveDepth ? _primitiveDeltaZ : _depthSlope;
            int deltaZEncoded = DeltaZEncoding(deltaZ);
            if (PrimitiveDepth) _depthCorrectDx = _depthCorrectDy = 0;

            int direction = majorOnLeft ? 1 : -1;
            Span<int> steps = stackalloc int[Attributes];
            for (int c = 0; c < 4; c++) steps[c] = direction * _shadeStep[c];
            steps[AttributeZ] = PrimitiveDepth ? 0 : direction * _depthStep;
            for (int c = 0; c < 3; c++) steps[AttributeS + c] = direction * _textureStep[c];

            int texelLevel = TwoCycleTexels(out bool lod);
            int ditherColor = 7, ditherAlpha = 0;
            Span<int> values = stackalloc int[Attributes];

            for (int y = rows.First; y <= rows.Last; y++)
            {
                if (!_spanDrawn[y] || _spanRight[y] < _spanLeft[y]) continue;

                int left = _spanLeft[y], right = _spanRight[y];
                RowCoverage(y, left, right);

                for (int c = 0; c < Attributes; c++) values[c] = _spanAttributes[y * Attributes + c];
                if (PrimitiveDepth) values[AttributeZ] = _primitiveZ;

                int clipped = (majorOnLeft ? left - _spanMajorX[y] : _spanMajorX[y] - right) & 0xFFF;
                for (int c = 0; c < Attributes; c++) values[c] += steps[c] * clipped;

                int last = right - left, length = last + clipped;
                bool nextRowDrawn = y + 1 <= rows.Last && _spanDrawn[y + 1];

                int x = majorOnLeft ? left : right;
                TwoCyclePixel current = FirstCycles(values, steps, x, y, _coverage[x], beyond: false, texelLevel, lod, nextRowDrawn, length, tile, maxLevel,
                    ref ditherColor, ref ditherAlpha);

                for (int n = 0; n <= last; n++, x += direction)
                {
                    int coverage = current.Coverage;
                    int z = CorrectDepth((values[AttributeZ] >> 10) & 0x3FFFFF, current.Offset, coverage);

                    // The second cycle reads the second texel as texel 0 and the first as texel 1 - see §2.
                    (_texel0, _texel1) = (_texel1, _texel0);
                    CombineSecondCycle(ditherAlpha, ref coverage);

                    int pixel = y * _colorImageWidth + x;
                    int memoryCoverage = ReadMemory(pixel);
                    uint depthIndex = (_depthImage >> 1) + (uint)pixel;

                    bool write = CompareDepth(depthIndex, z, deltaZ, deltaZEncoded, memoryCoverage, ref coverage, out bool blend, out bool overflow)
                        && (Antialias ? coverage != 0 : current.CoverageBit);

                    // The first blend is never divided, and weighs memory alpha by the previous pixel's slope - see §4.
                    if (write)
                    {
                        BlendEquation(FirstBlendCycle, _pixel, divide: false, _pastShiftA, _pastShiftB, out int br, out int bg, out int bb);
                        _blended = new Color { R = br, G = bg, B = bb };
                    }

                    for (int c = 0; c < Attributes; c++) values[c] += steps[c];
                    TwoCyclePixel next = FirstCycles(values, steps, x + direction, y, n < last ? _coverage[x + direction] : (byte)0, beyond: n == last, texelLevel, lod,
                        nextRowDrawn, length, tile, maxLevel, ref ditherColor, ref ditherAlpha);

                    if (write && (!AlphaCompare || current.CompareAlpha >= AlphaThreshold))
                    {
                        Blend(SecondBlendCycle, _blended, current.DitherColor, blend, overflow, _blendShiftA, _blendShiftB, out int r, out int g, out int b);
                        WriteMemory(pixel, r, g, b, blend, coverage, memoryCoverage);
                        if (DepthUpdate) StoreDepth(depthIndex, z, deltaZEncoded);
                    }

                    current = next;
                }
            }
        }

        // Coverage, shade, dither, texels and the first combiner cycle for a pixel, or for the one past a row's end - see §1.
        private TwoCyclePixel FirstCycles(ReadOnlySpan<int> values, ReadOnlySpan<int> steps, int x, int y, byte mask, bool beyond, int texelLevel, bool lod,
            bool nextRowDrawn, int length, int tile, int maxLevel, ref int ditherColor, ref int ditherAlpha)
        {
            int coverage = CoverageCount(mask);
            (byte X, byte Y) offset = CoverageOffsets[mask];

            _shade = new Color
            {
                R = CorrectShade(values[0] >> 14, _shadeCorrectDx[0], _shadeCorrectDy[0], offset, coverage),
                G = CorrectShade(values[1] >> 14, _shadeCorrectDx[1], _shadeCorrectDy[1], offset, coverage),
                B = CorrectShade(values[2] >> 14, _shadeCorrectDx[2], _shadeCorrectDy[2], offset, coverage),
                A = CorrectShade(values[3] >> 14, _shadeCorrectDx[3], _shadeCorrectDy[3], offset, coverage),
            };

            if (((RgbDither << 2) | AlphaDither) != 0xF) Dither(x, y, ref ditherColor, ref ditherAlpha);

            int s = values[AttributeS], t = values[AttributeT], w = values[AttributeW];
            int ds = steps[AttributeS], dt = steps[AttributeT], dw = steps[AttributeW];

            if (texelLevel <= 1)
            {
                // Past a long enough row's end, when both texels are read through the second cycle, the second is the next row's first - see §3.
                if (beyond && texelLevel == 0 && nextRowDrawn && length >= 3)
                {
                    (int first, int second, int fraction) = lod ? NextRowLevelOfDetail(y + 1, ds, dt, dw, tile, maxLevel) : (tile, tile, _lodFraction);
                    _lodFraction = fraction;

                    (int cs, int ct) = TextureCoordinates(s, t, w);
                    _texel0 = Texel(cs, ct, first, BilinearFirstCycle, convert: false, default);

                    (int rs, int rt, int rw) = RowStart(y + 1);
                    (int ns, int nt) = TextureCoordinates(rs, rt, rw);
                    _texel1 = Texel(ns, nt, second, BilinearFirstCycle, convert: false, _texel0);
                }
                else
                {
                    (int first, int second, int fraction) = lod ? TwoCycleLevelOfDetail(s, t, w, ds, dt, dw, tile, maxLevel) : (tile, (tile + 1) & 7, _lodFraction);
                    _lodFraction = fraction;

                    (int cs, int ct) = TextureCoordinates(s, t, w);
                    _texel0 = Texel(cs, ct, first, BilinearFirstCycle, convert: false, default);
                    _texel1 = Texel(cs, ct, second, BilinearSecondCycle, ConvertOne, _texel0);
                }
            }
            else if (texelLevel == 2)
            {
                (int first, _, int fraction) = lod ? TwoCycleLevelOfDetail(s, t, w, ds, dt, dw, tile, maxLevel) : (tile, 0, _lodFraction);
                _lodFraction = fraction;

                (int cs, int ct) = TextureCoordinates(s, t, w);
                _texel0 = Texel(cs, ct, first, BilinearFirstCycle, convert: false, default);
            }

            int compareAlpha = CombineFirstCycle(ditherAlpha, coverage);
            return new TwoCyclePixel(coverage, CoverageBit(mask), offset, compareAlpha, ditherColor);
        }

        // The first combiner cycle: the combined colour the second reads, and the alpha the pixel's alpha compare tests - see §1.
        private int CombineFirstCycle(int ditherAlpha, int coverage)
        {
            (int red, int green, int blue, int alpha) = CombinerEquations(FirstCombineCycle);

            int compareAlpha = 0;
            if (AlphaCompare)
            {
                compareAlpha = Clamp9(alpha);
                if (compareAlpha == 0xFF) compareAlpha = 0x100;

                if (!AlphaFromCoverage)
                {
                    compareAlpha += ditherAlpha;
                    if ((compareAlpha & 0x100) != 0) compareAlpha = 0xFF;
                }
                else
                {
                    compareAlpha = CoverageTimesAlpha ? (compareAlpha * coverage + 4) >> 3 : coverage << 5;
                    if (compareAlpha > 0xFF) compareAlpha = 0xFF;
                }
            }

            _combined = new Color { R = red >> 8, G = green >> 8, B = blue >> 8, A = alpha };

            _blenderShadeAlpha = _shade.A + ditherAlpha;
            if ((_blenderShadeAlpha & 0x100) != 0) _blenderShadeAlpha = 0xFF;

            return compareAlpha;
        }

        // Both texels and the second's successor, the first alone, or neither, by what the cycles read; and whether a level is measured - see §3.
        private int TwoCycleTexels(out bool lod)
        {
            CombinerSelectors first = FirstCombineCycle, second = SecondCombineCycle;

            static bool Alpha(CombinerSelectors c, int texel) => c.AlphaA == texel || c.AlphaB == texel || c.AlphaC == texel || c.AlphaD == texel;
            static bool Any(CombinerSelectors c, int texel) =>
                c.ColorA == texel || c.ColorB == texel || c.ColorC == texel || c.ColorD == texel || c.ColorC == texel + 7 || Alpha(c, texel);

            bool fractionFirst = first.ColorC == 13 || first.AlphaC == 0, fractionSecond = second.ColorC == 13 || second.AlphaC == 0;
            lod = LodEnabled || fractionFirst || fractionSecond;

            if (Any(second, 2) || AlphaCompare && (Alpha(first, 1) || Alpha(first, 2) || first.AlphaC == 0)) return 0;
            if (Any(first, 2) || Any(second, 1)) return 1;
            if (Any(first, 1) || fractionFirst || fractionSecond) return 2;
            return 3;
        }
    }
}
