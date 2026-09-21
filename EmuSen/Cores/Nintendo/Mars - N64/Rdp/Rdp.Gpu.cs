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
            if (CycleType != FillCycle && (CycleType != OneCycle || !TheDeviceShadesThisPrimitive())) { gpu.NotShaded(); return; }

            gpu.Image(_colorImage & ~(uint)Math.Max(_colorImageBytes - 1, 0), _colorImageWidth, _colorImageBytes == 1 ? 0 : _colorImageBytes);
            gpu.DepthImage(_depthImage);
            if (!gpu.Shades) { gpu.NotShaded(); return; }

            if (CycleType == FillCycle)
            {
                Span<uint> fill = gpu.Primitive(out int filled);
                fill[0] = 3;
                fill[1] = _fillColor;
                for (int y = rows.First; y <= rows.Last; y++)
                    if (_spanDrawn[y]) gpu.Row(filled, y, _spanLeft[y], _spanRight[y], Math.Max(_spanRight[y] - Math.Max(_spanLeft[y], _colorImageWidth) + 1, 0));
                return;
            }

            RecordOneCycle(gpu, rows, majorOnLeft, tile, maxLevel);
        }

        // The previous pixel's result is a carry; four texels and the level of detail are stages not ported yet - see §6.2 and §7.
        private bool TheDeviceShadesThisPrimitive()
        {
            bool lodFraction = CombineColorC == 13 || CombineAlphaC == 0;
            bool combined = CombineColorA == 0 || CombineColorB == 0 || CombineColorD == 0 || CombineColorC == 0 || CombineColorC == 7
                || CombineAlphaA == 0 || CombineAlphaB == 0 || CombineAlphaD == 0;
            return !combined && !lodFraction && !LodEnabled && !SampleFour && !PaletteEnabled;
        }

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
        private void RecordOneCycle(GpuRasteriser gpu, (int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            int deltaZ = PrimitiveDepth ? _primitiveDeltaZ : _depthSlope;
            if (PrimitiveDepth) _depthCorrectDx = _depthCorrectDy = 0;

            int direction = majorOnLeft ? 1 : -1;
            Span<int> steps = stackalloc int[Attributes];
            for (int c = 0; c < 4; c++) steps[c] = direction * _shadeStep[c];
            steps[AttributeZ] = PrimitiveDepth ? 0 : direction * _depthStep;
            for (int c = 0; c < 3; c++) steps[AttributeS + c] = direction * _textureStep[c];

            (bool texel0, bool texel1) = CombinerTexels();
            uint memory = gpu.TextureMemory(TextureMemory, _textureMemoryChanged);
            Span<uint> packed = gpu.Tiles(_tilesChanged, out uint tileSet);
            if (!packed.IsEmpty) RecordTiles(packed);
            _textureMemoryChanged = _tilesChanged = false;

            Span<uint> p = gpu.Primitive(out int primitive);
            p[0] = 0;
            p[2] = Bit(0, KeyEnabled) | Bit(1, CoverageTimesAlpha) | Bit(2, AlphaFromCoverage) | Bit(3, AlphaCompare) | Bit(4, DitherAlpha)
                | Bit(5, Antialias) | Bit(6, ColorOnCoverage) | Bit(7, ForceBlend) | Bit(8, ImageRead) | Bit(9, DepthUpdate) | Bit(10, DepthCompare)
                | Bit(11, PrimitiveDepth) | Bit(12, _scissorField) | Bit(13, majorOnLeft) | Bit(14, ((RgbDither << 2) | AlphaDither) != 0xF);
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
                | Bit(9, SharpenEnabled) | Bit(10, LodEnabled) | Bit(11, ConvertOne);
            p[58] = memory;
            p[59] = tileSet;
            p[60] = (uint)tile;
            p[61] = (uint)maxLevel;
            p[62] = (uint)_minLevel;
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
                row[24] = Bit(0, nextRowDrawn) | Bit(1, length > 7) | Bit(2, length == 7);
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
