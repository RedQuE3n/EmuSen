using EmuSen.Cores.Nintendo.Mars.Rdp.Gpu;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // A processor at the multiple that walks and hands its rows to a device instead of shading them - see Mars_Gpu.md §5 and §6.
    public sealed partial class Rdp
    {
        [EmuSen.Common.SkipInState] private GpuRasteriser? _gpu;

        // Only a processor at a multiple, drawing alone: the machine's own picture is never the device's - see Mars_GpuPlan.md §0.
        public void ShadeOn(GpuRasteriser? gpu)
        {
            if (gpu is not null && !_scaled) throw new InvalidOperationException("the device shades the multiple, never the machine's picture");
            _gpu = gpu;
            if (gpu is not null) { _alone = true; _workers = 1; }
        }

        // Set when a load or a tile command has changed what the device's copy would have to hold - see Mars_Gpu.md §7.1.
        [EmuSen.Common.SkipInState] private bool _textureMemoryChanged = true;
        [EmuSen.Common.SkipInState] private bool _tilesChanged = true;

        private void RecordForTheDevice((int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            GpuRasteriser gpu = _gpu!;
            if (CycleType == CopyCycle) { gpu.NotShaded(GpuRasteriser.Declined.Copy); return; }
            if (CycleType != FillCycle && !TheDeviceShadesThisPrimitive()) { gpu.NotShaded(GpuRasteriser.Declined.Carry); return; }

            gpu.Image(_colorImage & ~(uint)Math.Max(_colorImageBytes - 1, 0), _colorImageWidth, _colorImageBytes == 1 ? 0 : _colorImageBytes);
            gpu.DepthImage(_depthImage);
            if (!gpu.Shades) { gpu.NotShaded(GpuRasteriser.Declined.Image); return; }

            if (CycleType == FillCycle)
            {
                Span<uint> fill = gpu.Primitive(out int filled);
                fill[0] = 3;
                fill[1] = _fillColor;
                for (int y = rows.First; y <= rows.Last; y++)
                    if (_spanDrawn[y]) gpu.Row(filled, y, _spanLeft[y], _spanRight[y], Math.Max(_spanRight[y] - Math.Max(_spanLeft[y], _colorImageWidth) + 1, 0));
                return;
            }

            RecordShaded(gpu, rows, majorOnLeft, tile, maxLevel);
        }

        // A cycle reading the combiner's own previous result is the carry an invocation per pixel cannot see - see §6.2.
        private static bool ReadsCombined(CombinerSelectors c) =>
            c.ColorA == 0 || c.ColorB == 0 || c.ColorD == 0 || c.ColorC == 0 || c.ColorC == 7
            || c.AlphaA == 0 || c.AlphaB == 0 || c.AlphaD == 0;

        // In the two-cycle mode the second cycle's COMBINED is this pixel's first cycle and is fine; the first cycle's
        // is the pixel before. The first blend weighing memory alpha carries the previous pixel's depth slope - see §10.1.
        private bool TheDeviceShadesThisPrimitive() =>
            CycleType == OneCycle
                ? !ReadsCombined(SecondCombineCycle)
                : !ReadsCombined(FirstCombineCycle) && BlendSecondAlpha != 1 && !ConvertOne;

        // The eight tiles as the shader reads them, four words each - see §7.1.
        private void RecordTiles(Span<uint> into)
        {
            for (int i = 0; i < _tiles.Length; i++)
            {
                ref TextureTile tile = ref _tiles[i];
                into[i * 4] = (uint)(tile.Format | (tile.Size << 3) | (tile.Line << 5) | (tile.Memory << 14) | (tile.Palette << 23));
                into[i * 4 + 1] = (uint)(Bit(0, tile.ClampS) | Bit(1, tile.MirrorS) | Bit(2, tile.ClampT) | Bit(3, tile.MirrorT)
                    | ((uint)tile.MaskS << 4) | ((uint)tile.ShiftS << 8) | ((uint)tile.MaskT << 12) | ((uint)tile.ShiftT << 16));
                into[i * 4 + 2] = (uint)(tile.SL | (tile.TL << 12));
                into[i * 4 + 3] = (uint)(tile.SH | (tile.TH << 12));
            }
        }

        // DrawOneCycle's setup, written down instead of run: the layout is shade.comp's - see §6.
        private void RecordShaded(GpuRasteriser gpu, (int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            int deltaZ = PrimitiveDepth ? _primitiveDeltaZ : _depthSlope;
            if (PrimitiveDepth) _depthCorrectDx = _depthCorrectDy = 0;

            int direction = majorOnLeft ? 1 : -1;
            Span<int> steps = stackalloc int[Attributes];
            for (int c = 0; c < 4; c++) steps[c] = direction * _shadeStep[c];
            steps[AttributeZ] = PrimitiveDepth ? 0 : direction * _depthStep;
            for (int c = 0; c < 3; c++) steps[AttributeS + c] = direction * _textureStep[c];

            (bool texel0, bool texel1) = CombinerTexels();
            bool readsLodFraction = CombineColorC == 13 || CombineAlphaC == 0;
            uint memory = gpu.TextureMemory(TextureMemory, _textureMemoryChanged);
            Span<uint> packed = gpu.Tiles(_tilesChanged, out uint tileSet);
            if (!packed.IsEmpty) RecordTiles(packed);
            _textureMemoryChanged = _tilesChanged = false;

            bool twoCycle = CycleType == TwoCycle;
            int texelLevel = twoCycle ? TwoCycleTexels(out _) : 3;

            Span<uint> p = gpu.Primitive(out int primitive);
            p[0] = twoCycle ? 1u : 0u;
            p[2] = Bit(0, KeyEnabled) | Bit(1, CoverageTimesAlpha) | Bit(2, AlphaFromCoverage) | Bit(3, AlphaCompare) | Bit(4, DitherAlpha)
                | Bit(5, Antialias) | Bit(6, ColorOnCoverage) | Bit(7, ForceBlend) | Bit(8, ImageRead) | Bit(9, DepthUpdate) | Bit(10, DepthCompare)
                | Bit(11, PrimitiveDepth) | Bit(12, _scissorField) | Bit(13, majorOnLeft) | Bit(14, ((RgbDither << 2) | AlphaDither) != 0xF)
                | Bit(15, (CycleType == TwoCycle ? SecondBlendCycle.SecondAlpha : BlendSecondAlpha) == 1);
            p[3] = (uint)(RgbDither | (AlphaDither << 2) | (DepthMode << 4) | (CoverageDestination << 6) | (_colorImageSize << 8) | (_colorImageFormat << 10));

            CombinerSelectors c2 = SecondCombineCycle;
            p[4] = (uint)(c2.ColorA | (c2.ColorB << 4) | (c2.ColorC << 8) | ((c2.ColorD & 7) << 13) | (c2.AlphaA << 16) | (c2.AlphaB << 19) | (c2.AlphaC << 22) | (c2.AlphaD << 25));

            BlendSelectors blend = FirstBlendCycle;
            p[5] = (uint)(blend.FirstColor | (blend.FirstAlpha << 2) | (blend.SecondColor << 4) | (blend.SecondAlpha << 6));

            Put(p, 6, _primitiveColor);
            Put(p, 10, _environmentColor);
            Put(p, 14, _blendColor);
            Put(p, 18, _fogColor);
            Put(p, 22, _keyCenter);
            Put(p, 26, _keyScale);
            Put(p, 30, _keyWidth);
            p[33] = (uint)_k4;
            p[34] = (uint)_k5;
            p[35] = (uint)_primitiveLodFraction;
            p[36] = (uint)_primitiveZ;
            p[37] = (uint)deltaZ;
            p[38] = (uint)DeltaZEncoding(deltaZ);
            for (int c = 0; c < Attributes; c++) p[40 + c] = (uint)steps[c];
            for (int c = 0; c < 4; c++) { p[48 + c] = (uint)_shadeCorrectDx[c]; p[52 + c] = (uint)_shadeCorrectDy[c]; }
            p[56] = (uint)_depthCorrectDx;
            p[57] = (uint)_depthCorrectDy;

            p[39] = Bit(0, texel0) | Bit(1, texel1) | Bit(2, Perspective) | Bit(3, SampleFour) | Bit(4, PaletteEnabled)
                | Bit(5, PaletteIntensityAlpha) | Bit(6, MidTexel) | Bit(7, BilinearFirstCycle) | Bit(8, DetailEnabled)
                | Bit(9, SharpenEnabled) | Bit(10, LodEnabled) | Bit(11, ConvertOne) | Bit(12, readsLodFraction) | Bit(13, BilinearSecondCycle);
            p[58] = memory;
            p[59] = tileSet;
            p[60] = (uint)tile;
            p[61] = (uint)maxLevel;
            p[62] = (uint)_minLevel;
            CombinerSelectors c1 = FirstCombineCycle;
            p[68] = (uint)(c1.ColorA | (c1.ColorB << 4) | (c1.ColorC << 8) | ((c1.ColorD & 7) << 13) | (c1.AlphaA << 16) | (c1.AlphaB << 19) | (c1.AlphaC << 22) | (c1.AlphaD << 25));

            BlendSelectors blend2 = SecondBlendCycle;
            p[69] = (uint)(blend2.FirstColor | (blend2.FirstAlpha << 2) | (blend2.SecondColor << 4) | (blend2.SecondAlpha << 6));
            p[70] = (uint)texelLevel;
            for (int c = 0; c < 3; c++) p[72 + c] = (uint)(_attributeDy[AttributeS + c] & ~0x7FFF);

            p[64] = (uint)_k0;
            p[65] = (uint)_k1;
            p[66] = (uint)_k2;
            p[67] = (uint)_k3;

            for (int y = rows.First; y <= rows.Last; y++)
            {
                if (!_spanDrawn[y] || _spanRight[y] < _spanLeft[y]) continue;

                int left = _spanLeft[y], right = _spanRight[y];
                Span<uint> row = gpu.Row(primitive, y, left, right, CoveredPastTheWidth(y, left, right));
                if (row.IsEmpty) continue;

                // The row's values at its first pixel, as DrawOneCycle steps them there from the major edge.
                int clipped = majorOnLeft ? left - _spanMajorX[y] : _spanMajorX[y] - right;
                if (!_scaled) clipped &= 0xFFF;
                for (int c = 0; c < Attributes; c++) row[4 + c] = (uint)(_spanAttributes[y * Attributes + c] + steps[c] * clipped);

                int length = (right - left) + clipped;
                bool nextRowDrawn = y + 1 <= rows.Last && _spanDrawn[y + 1];
                row[24] = Bit(0, nextRowDrawn) | Bit(1, length > 7) | Bit(2, length == 7) | Bit(3, length == 6) | Bit(4, length >= 3);
                if (nextRowDrawn)
                    for (int c = 0; c < 3; c++) row[21 + c] = (uint)_spanAttributes[(y + 1) * Attributes + AttributeS + c];

                uint invalid = 0;
                for (int sub = 0; sub < 4; sub++)
                {
                    int k = y * 4 + sub;
                    row[12 + sub] = (uint)_edgeLeft[k];
                    row[16 + sub] = (uint)_edgeRight[k];
                    if (_edgeInvalid[k]) invalid |= 1u << sub;
                }
                row[20] = invalid;
            }
        }

        // A shaded column with no coverage is read and never written, and the scissor's own column is always such a one - see §5.4.
        private int CoveredPastTheWidth(int y, int left, int right)
        {
            if (right < _colorImageWidth) return 0;

            RowCoverage(y, left, right);
            int covered = 0;
            for (int x = Math.Max(left, _colorImageWidth); x <= right; x++) if (_coverage[x] != 0) covered++;
            return covered;
        }

        private static uint Bit(int at, bool set) => set ? 1u << at : 0;

        private static void Put(Span<uint> record, int at, Color color)
        {
            record[at] = (uint)color.R;
            record[at + 1] = (uint)color.G;
            record[at + 2] = (uint)color.B;
            record[at + 3] = (uint)color.A;
        }
    }
}
