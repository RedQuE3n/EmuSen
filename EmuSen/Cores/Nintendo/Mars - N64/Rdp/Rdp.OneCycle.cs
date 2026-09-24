using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The one-cycle mode: combiner, blender, dither and the framebuffer they read and write - see Mars_RdpCoverage.md.
    public sealed partial class Rdp
    {
        private const int OneCycle = 0;

        // The two ordered-dither patterns, as the reference rasterizer carries them - see §4.4.
        private static readonly byte[] MagicSquare = { 0, 6, 1, 7, 4, 2, 5, 3, 3, 5, 2, 4, 7, 1, 6, 0 };
        private static readonly byte[] Bayer = { 0, 4, 1, 5, 4, 0, 5, 1, 3, 7, 2, 6, 7, 3, 6, 2 };

        private static readonly byte[] BlendQuotients = BuildBlendQuotients();

        // The previous pixel's combiner result, which is what the one-cycle mode's combined input reads - see §6.1.
        private Color _combined;
        private Color _pixel;
        private Color _memory;
        private Color _shade;
        private Color _texel0;
        private Color _texel1;
        private int _blenderShadeAlpha;
        private int _blendShiftA;
        private int _blendShiftB;

        // Each row starts from its major edge's values, stepped to the first pixel drawn, and steps once per pixel - see Mars_RdpDepth.md §2.
        private void DrawOneCycle((int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            int deltaZ = PrimitiveDepth ? _primitiveDeltaZ : _depthSlope;
            int deltaZEncoded = DeltaZEncoding(deltaZ);
            if (PrimitiveDepth) _depthCorrectDx = _depthCorrectDy = 0;

            int direction = majorOnLeft ? 1 : -1;
            Span<int> steps = stackalloc int[Attributes];
            for (int c = 0; c < 4; c++) steps[c] = direction * _shadeStep[c];
            steps[AttributeZ] = PrimitiveDepth ? 0 : direction * _depthStep;
            for (int c = 0; c < 3; c++) steps[AttributeS + c] = direction * _textureStep[c];

            (bool texel0, bool texel1) = CombinerTexels();
            bool lodFraction = CombineColorC == 13 || CombineAlphaC == 0;
            bool lod = LodEnabled || lodFraction;
            bool measures = lod && (texel0 || texel1 || lodFraction);

            int ditherColor = 7, ditherAlpha = 0;
            bool dither = ((RgbDither << 2) | AlphaDither) != 0xF;
            Span<int> values = stackalloc int[Attributes];

            for (int y = rows.First; y <= rows.Last; y++)
            {
                if (!_spanDrawn[y] || _spanRight[y] < _spanLeft[y] || !Owns(y)) continue;

                _rowStamp = Stamp(y);
                _lastShadedStamp = _memoryStamp = _pastStoredStamp = _rowStamp;

                // A row that measures no level keeps the fraction it found, which is not this row's to stamp - see Mars_Rdp.md §2.8.
                if (measures) _lodStamp = _rowStamp;
                if (texel0 || texel1) _texel0Stamp = _rowStamp;
                if (texel1) _texel1Stamp = _rowStamp;

                int left = _spanLeft[y], right = _spanRight[y];
                RowCoverage(y, left, right);

                for (int c = 0; c < Attributes; c++) values[c] = _spanAttributes[y * Attributes + c];
                if (PrimitiveDepth) values[AttributeZ] = _primitiveZ;

                int clipped = majorOnLeft ? left - _spanMajorX[y] : _spanMajorX[y] - right;
                if (!_scaled) clipped &= 0xFFF;
                for (int c = 0; c < Attributes; c++) values[c] += steps[c] * clipped;

                // The next pixel's texel of a long span's last pixel is the next row's first, when that row is drawn - see Mars_RdpTextures.md §6.
                int last = right - left, length = last + clipped;
                bool longSpan = length > 7, midSpan = length == 7;
                bool nextRowDrawn = y + 1 <= rows.Last && _spanDrawn[y + 1];
                (int Tile, int Fraction) level = (tile, _lodFraction);
                bool levelReady = false;

                int x = majorOnLeft ? left : right;
                for (int n = 0; n <= last; n++, x += direction)
                {
                    int ds = steps[AttributeS], dt = steps[AttributeT], dw = steps[AttributeW];

                    // Each pixel's level is the one its predecessor measured for its texel 1, when there was one - see Mars_RdpLod.md §2.
                    if (measures && !levelReady)
                    {
                        level = PixelLevelOfDetail(values[AttributeS], values[AttributeT], values[AttributeW], ds, dt, dw, y + 1, nextRowDrawn,
                            n == last, n == last - 1, longSpan, midSpan, tile, maxLevel);
                    }

                    _lodFraction = level.Fraction;
                    levelReady = false;

                    if (texel0 || texel1)
                    {
                        (int s, int t) = TextureCoordinates(values[AttributeS], values[AttributeT], values[AttributeW]);
                        _texel0 = Texel(s, t, level.Tile);
                    }

                    if (texel1)
                    {
                        int next = (y + 1) * Attributes;
                        (int s, int t) = n == last && longSpan && nextRowDrawn
                            ? TextureCoordinates(_spanAttributes[next + AttributeS], _spanAttributes[next + AttributeT], _spanAttributes[next + AttributeW])
                            : TextureCoordinates(values[AttributeS] + ds, values[AttributeT] + dt, values[AttributeW] + dw);

                        int nextTile = tile;
                        if (lod && n < last)
                        {
                            level = PixelLevelOfDetail(values[AttributeS] + ds, values[AttributeT] + dt, values[AttributeW] + dw, ds, dt, dw, y + 1, nextRowDrawn,
                                n + 1 == last, n + 1 == last - 1, longSpan, midSpan, tile, maxLevel);
                            (nextTile, levelReady) = (level.Tile, true);
                        }
                        else if (lod)
                        {
                            nextTile = AfterSpanTile(values[AttributeS] + ds, values[AttributeT] + dt, values[AttributeW] + dw, ds, dt, dw, y + 1, nextRowDrawn,
                                longSpan, midSpan, length == 6, tile, maxLevel);
                        }

                        _texel1 = Texel(s, t, nextTile);
                    }

                    byte mask = _coverage[x];
                    int coverage = CoverageCount(mask);
                    bool coverageBit = CoverageBit(mask);
                    (byte X, byte Y) offset = CoverageOffsets[mask];

                    _shade = new Color
                    {
                        R = CorrectShade(values[0] >> 14, _shadeCorrectDx[0], _shadeCorrectDy[0], offset, coverage),
                        G = CorrectShade(values[1] >> 14, _shadeCorrectDx[1], _shadeCorrectDy[1], offset, coverage),
                        B = CorrectShade(values[2] >> 14, _shadeCorrectDx[2], _shadeCorrectDy[2], offset, coverage),
                        A = CorrectShade(values[3] >> 14, _shadeCorrectDx[3], _shadeCorrectDy[3], offset, coverage),
                    };
                    int z = CorrectDepth((values[AttributeZ] >> 10) & 0x3FFFFF, offset, coverage);

                    if (dither) Dither(x, y, ref ditherColor, ref ditherAlpha);
                    CombineSecondCycle(ditherAlpha, ref coverage);

                    int pixel = y * _colorImageWidth + x;
                    int memoryCoverage = ReadMemory(pixel);
                    uint depthIndex = (_depthImage >> 1) + (uint)pixel;

                    if (CompareDepth(depthIndex, z, deltaZ, deltaZEncoded, memoryCoverage, ref coverage, out bool blend, out bool overflow)
                        && BlendOneCycle(ditherColor, blend, overflow, coverage, coverageBit, out int r, out int g, out int b))
                    {
                        WriteMemory(pixel, r, g, b, blend, coverage, memoryCoverage);
                        if (DepthUpdate) StoreDepth(depthIndex, z, deltaZEncoded);
                    }

                    for (int c = 0; c < Attributes; c++) values[c] += steps[c];
                }
            }
        }

        // Rows are counted in fields under an interlaced scissor; noise selections are not built - see §4.4.
        private void Dither(int x, int y, ref int color, ref int alpha)
        {
            int index = (((y >> (_scissorField ? 1 : 0)) & 3) << 2) | (x & 3);
            (color, alpha) = _ditherTable[index];
        }

        // The sixteen colour and alpha dithers of one pair of modes, one of sixteen tables built once - see Mars_Performance.md §23.
        [EmuSen.Common.SkipInState] private (byte Color, byte Alpha)[] _ditherTable = DitherTables[0];

        private static readonly (byte Color, byte Alpha)[][] DitherTables = BuildDitherTables();

        private static (byte Color, byte Alpha)[][] BuildDitherTables()
        {
            var tables = new (byte Color, byte Alpha)[16][];

            for (int rgb = 0; rgb < 4; rgb++)
            {
                for (int alphaMode = 0; alphaMode < 4; alphaMode++)
                {
                    var table = new (byte Color, byte Alpha)[16];
                    for (int index = 0; index < 16; index++)
                    {
                        (int color, int pattern) = rgb switch
                        {
                            0 => (MagicSquare[index], (int)MagicSquare[index]),
                            1 => (Bayer[index], (int)Bayer[index]),
                            2 => (0, (int)MagicSquare[index]),
                            _ => (7, (int)Bayer[index]),
                        };

                        int alpha = alphaMode switch { 0 => pattern, 1 => ~pattern & 7, _ => 0 };
                        table[index] = ((byte)color, (byte)alpha);
                    }

                    tables[rgb * 4 + alphaMode] = table;
                }
            }

            return tables;
        }

        // (A - B) × C + D in nine-bit signed arithmetic for one cycle's selectors: colour before its shift to nine bits, alpha after - see §4.1.
        private (int R, int G, int B, int A) CombinerEquations(CombinerSelectors c)
        {
            // Each input's source is chosen once, not once a channel - see Mars_Performance.md §23.
            Color a = ColorA(c.ColorA), b = ColorB(c.ColorB), m = ColorC(c.ColorC), d = ColorD(c.ColorD);

            return (
                ColorEquation(a.R, b.R, m.R, d.R),
                ColorEquation(a.G, b.G, m.G, d.G),
                ColorEquation(a.B, b.B, m.B, d.B),
                AlphaEquation(AlphaABD(c.AlphaA), AlphaABD(c.AlphaB), AlphaC(c.AlphaC), AlphaABD(c.AlphaD)));
        }

        // The one-cycle mode's cycle and the two-cycle mode's last: the pixel's colour and alpha, then clamped to eight bits - see §4.1.
        private void CombineSecondCycle(int ditherAlpha, ref int coverage)
        {
            CombinerSelectors last = SecondCombineCycle;
            (int red, int green, int blue, int alpha) = CombinerEquations(last);

            _combined = new Color { R = red >> 8, G = green >> 8, B = blue >> 8, A = alpha };

            // Keying passes this cycle's first colour input through in place of its result - see Mars_RdpChromaKey.md §1.
            int keyAlpha = 0;
            if (KeyEnabled)
            {
                keyAlpha = ChromaKey(red, green, blue);
                Color through = ColorA(last.ColorA);
                _pixel = new Color { R = Clamp9(through.R), G = Clamp9(through.G), B = Clamp9(through.B) };
            }
            else
            {
                _pixel = new Color { R = Clamp9(_combined.R), G = Clamp9(_combined.G), B = Clamp9(_combined.B) };
            }

            int pixelAlpha = Clamp9(alpha);
            if (pixelAlpha == 0xFF) pixelAlpha = 0x100;

            int scaled = 0;
            if (CoverageTimesAlpha)
            {
                scaled = (pixelAlpha * coverage + 4) >> 3;
                coverage = (scaled >> 5) & 0xF;
            }

            if (!AlphaFromCoverage)
            {
                if (KeyEnabled) pixelAlpha = keyAlpha;
                else
                {
                    pixelAlpha += ditherAlpha;
                    if ((pixelAlpha & 0x100) != 0) pixelAlpha = 0xFF;
                }
            }
            else
            {
                pixelAlpha = CoverageTimesAlpha ? scaled : coverage << 5;
                if (pixelAlpha > 0xFF) pixelAlpha = 0xFF;
            }

            _pixel.A = pixelAlpha;

            _blenderShadeAlpha = _shade.A + ditherAlpha;
            if ((_blenderShadeAlpha & 0x100) != 0) _blenderShadeAlpha = 0xFF;
        }

        // Which texels the second cycle's selectors read, so a primitive that reads neither fetches none - see Mars_RdpTextures.md §6.
        private (bool Texel0, bool Texel1) CombinerTexels()
        {
            bool Reads(int texel) =>
                CombineColorA == texel || CombineColorB == texel || CombineColorD == texel || CombineColorC == texel || CombineColorC == texel + 7
                || CombineAlphaA == texel || CombineAlphaB == texel || CombineAlphaC == texel || CombineAlphaD == texel;

            return (Reads(1), Reads(2));
        }

        private Color ColorA(int selector) => selector switch
        {
            0 => _combined,
            1 => _texel0,
            2 => _texel1,
            3 => _primitiveColor,
            4 => _shade,
            5 => _environmentColor,
            6 => Broadcast(0x100),
            _ => default,
        };

        private Color ColorB(int selector) => selector switch
        {
            0 => _combined,
            1 => _texel0,
            2 => _texel1,
            3 => _primitiveColor,
            4 => _shade,
            5 => _environmentColor,
            6 => _keyCenter,
            7 => Broadcast(_k4),
            _ => default,
        };

        private Color ColorC(int selector) => selector switch
        {
            0 => _combined,
            1 => _texel0,
            2 => _texel1,
            3 => _primitiveColor,
            4 => _shade,
            5 => _environmentColor,
            6 => _keyScale,
            7 => Broadcast(_combined.A),
            8 => Broadcast(_texel0.A),
            9 => Broadcast(_texel1.A),
            10 => Broadcast(_primitiveColor.A),
            11 => Broadcast(_shade.A),
            12 => Broadcast(_environmentColor.A),
            13 => Broadcast(_lodFraction),
            14 => Broadcast(_primitiveLodFraction),
            15 => Broadcast(_k5),
            _ => default,
        };

        private Color ColorD(int selector) => selector switch
        {
            0 => _combined,
            1 => _texel0,
            2 => _texel1,
            3 => _primitiveColor,
            4 => _shade,
            5 => _environmentColor,
            6 => Broadcast(0x100),
            _ => default,
        };

        // A scalar input is the same value on every channel.
        private static Color Broadcast(int value) => new() { R = value, G = value, B = value };

        private int AlphaABD(int selector) => selector switch
        {
            0 => _combined.A,
            1 => _texel0.A,
            2 => _texel1.A,
            3 => _primitiveColor.A,
            4 => _shade.A,
            5 => _environmentColor.A,
            6 => 0x100,
            _ => 0,
        };

        private int AlphaC(int selector) => selector switch
        {
            0 => _lodFraction,
            1 => _texel0.A,
            2 => _texel1.A,
            3 => _primitiveColor.A,
            4 => _shade.A,
            5 => _environmentColor.A,
            6 => _primitiveLodFraction,
            _ => 0,
        };

        private static int ColorEquation(int a, int b, int c, int d) =>
            ((Extend9(a) - Extend9(b)) * SignedMultiplier(c) + (Extend9(d) << 8) + 0x80) & 0x1FFFF;

        private static int AlphaEquation(int a, int b, int c, int d) =>
            (((Extend9(a) - Extend9(b)) * SignedMultiplier(c) + (Extend9(d) << 8) + 0x80) >> 8) & 0x1FF;

        // Nine bits whose top two are both set are negative; the multiplier is negative on its ninth bit alone.
        private static int Extend9(int value) => (value & 0x180) == 0x180 ? value | ~0x1FF : value & 0x1FF;

        private static int SignedMultiplier(int value) => value | -(value & 0x100);

        private static int Clamp9(int value) => ((value >> 7) & 3) switch { 2 => 0xFF, 3 => 0, _ => value & 0xFF };

        // Alpha compare, the coverage test, and then blend, pass through or the second input; noise thresholds are not built - see §4.2.
        private bool BlendOneCycle(int ditherColor, bool blend, bool overflow, int coverage, bool coverageBit, out int r, out int g, out int b)
        {
            r = g = b = 0;

            if (AlphaCompare && _pixel.A < AlphaThreshold) return false;
            if (Antialias ? coverage == 0 : !coverageBit) return false;

            Blend(FirstBlendCycle, _pixel, ditherColor, blend, overflow, _blendShiftA, _blendShiftB, out r, out g, out b);
            return true;
        }

        private int AlphaThreshold => DitherAlpha ? 0 : _blendColor.A;

        // Blend, pass the first input through, or take the second, then dither; source is what input 0 reads - see §4.2.
        private void Blend(BlendSelectors selectors, Color source, int ditherColor, bool blend, bool overflow, int shiftA, int shiftB, out int r, out int g, out int b)
        {
            if (!ColorOnCoverage || overflow)
            {
                bool opaque = selectors.FirstAlpha == 0 && selectors.SecondAlpha == 0 && _pixel.A >= 0xFF;

                if (!blend || opaque) (r, g, b) = BlendInput(selectors.FirstColor, source);
                else BlendEquation(selectors, source, !ForceBlend, shiftA, shiftB, out r, out g, out b);
            }
            else
            {
                (r, g, b) = BlendInput(selectors.SecondColor, source);
            }

            if (RgbDither != 3)
            {
                r = Dithered(r, RgbDither == 2 ? ditherColor & 7 : ditherColor);
                g = Dithered(g, RgbDither == 2 ? (ditherColor >> 3) & 7 : ditherColor);
                b = Dithered(b, RgbDither == 2 ? (ditherColor >> 6) & 7 : ditherColor);
            }
        }

        private (int R, int G, int B) BlendInput(int selector, Color source) => selector switch
        {
            0 => (source.R, source.G, source.B),
            1 => (_memory.R, _memory.G, _memory.B),
            2 => (_blendColor.R, _blendColor.G, _blendColor.B),
            _ => (_fogColor.R, _fogColor.G, _fogColor.B),
        };

        // Weighted by the two alphas in eighths, then divided by their sum unless told not to - see §4.2.
        private void BlendEquation(BlendSelectors selectors, Color source, bool divide, int shiftA, int shiftB, out int r, out int g, out int b)
        {
            int firstAlpha = selectors.FirstAlpha switch { 0 => _pixel.A, 1 => _fogColor.A, 2 => _blenderShadeAlpha, _ => 0 };
            int secondAlpha = selectors.SecondAlpha switch { 0 => ~firstAlpha & 0xFF, 1 => _memory.A, 2 => 0xFF, _ => 0 };

            int first = firstAlpha >> 3, second = secondAlpha >> 3;
            if (selectors.SecondAlpha == 1)
            {
                first = (first >> shiftA) & 0x3C;
                second = (second >> shiftB) | 3;
            }

            (int R, int G, int B) one = BlendInput(selectors.FirstColor, source), two = BlendInput(selectors.SecondColor, source);
            int weight = second + 1;

            int sumR = one.R * first + two.R * weight;
            int sumG = one.G * first + two.G * weight;
            int sumB = one.B * first + two.B * weight;

            if (!divide)
            {
                (r, g, b) = ((sumR >> 5) & 0xFF, (sumG >> 5) & 0xFF, (sumB >> 5) & 0xFF);
                return;
            }

            int divisor = ((first & ~3) + (second & ~3) + 4) << 9;
            r = BlendQuotients[divisor | ((sumR >> 2) & 0x7FF)];
            g = BlendQuotients[divisor | ((sumG >> 2) & 0x7FF)];
            b = BlendQuotients[divisor | ((sumB >> 2) & 0x7FF)];
        }

        // Rounds up to the next multiple of eight when the dither value is below the colour's low three bits.
        private static int Dithered(int value, int dither)
        {
            int raised = value > 247 ? 255 : (value & 0xF8) + 8;
            return dither < (value & 7) ? raised : value;
        }

        // Four divisor bits and eleven dividend bits in; eight quotient bits out of a bit-serial divider - see §4.2.
        private static byte[] BuildBlendQuotients()
        {
            var table = new byte[0x8000];

            for (int index = 0; index < table.Length; index++)
            {
                int divisor = (index >> 11) & 0xF, dividend = index & 0x7FF, complement = ~divisor & 0xF;
                int remainder = (complement + (dividend >> 8) + 1) & 7;
                int quotient = 0;

                for (int bit = 7; bit >= 0; bit--)
                {
                    int incoming = (dividend >> bit) & 1;
                    int sum = ((quotient >> (bit + 1)) & 1) != 0
                        ? complement + (remainder << 1) + incoming + 1
                        : divisor + (remainder << 1) + incoming;

                    remainder = sum & 7;
                    if ((sum & 0x10) != 0) quotient |= 1 << bit;
                }

                table[index] = (byte)quotient;
            }

            return table;
        }

        // Reads memory colour and, with image reads on, the coverage stored beside it - see §3.
        private int ReadMemory(int pixel)
        {
            byte[] rdram = _frame;
            _memory = new Color { A = 0xE0 };

            switch (_colorImageSize)
            {
                case 1:
                {
                    uint at = _colorImage + (uint)pixel;
                    Touch(at);
                    int value = at < rdram.Length ? rdram[at] : 0;
                    _memory = new Color { R = value, G = value, B = value, A = 0xE0 };
                    return 7;
                }

                case 2:
                {
                    uint word = (_colorImage >> 1) + (uint)pixel;
                    Touch(word * 2);
                    bool valid = word * 2 + 1 < rdram.Length;
                    int value = valid ? (rdram[word * 2] << 8) | rdram[word * 2 + 1] : 0;
                    int hidden = valid ? _frameHidden[word] : 0;

                    if (_colorImageFormat == 0) _memory = new Color { R = (value >> 8) & 0xF8, G = (value & 0x7C0) >> 3, B = (value & 0x3E) << 2 };
                    else _memory = new Color { R = value >> 8, G = value >> 8, B = value >> 8 };

                    if (!ImageRead)
                    {
                        _memory.A = 0xE0;
                        return 7;
                    }

                    int stored = _colorImageFormat == 0 ? ((value & 1) << 2) | hidden : (value >> 5) & 7;
                    _memory.A = stored << 5;
                    return stored;
                }

                case 3:
                {
                    uint at = ((_colorImage >> 2) + (uint)pixel) * 4;
                    Touch(at);
                    uint value = at + 3 < rdram.Length ? (uint)((rdram[at] << 24) | (rdram[at + 1] << 16) | (rdram[at + 2] << 8) | rdram[at + 3]) : 0;
                    _memory = new Color { R = (int)(value >> 24), G = (int)(value >> 16) & 0xFF, B = (int)(value >> 8) & 0xFF };

                    _memory.A = ImageRead ? (int)value & 0xE0 : 0xE0;
                    return ImageRead ? (int)(value >> 5) & 7 : 7;
                }

                default:
                    return 7;
            }
        }

        // A four-bit image takes a zero byte, an eight-bit one alternates red and green by address - see §3.
        private void WriteMemory(int pixel, int r, int g, int b, bool blend, int coverage, int memoryCoverage)
        {
            byte[] rdram = _frame;

            switch (_colorImageSize)
            {
                case 0:
                {
                    uint at = _colorImage + (uint)pixel;
                    Wrote(at);
                    if (at < rdram.Length) rdram[at] = 0;
                    return;
                }

                case 1:
                {
                    uint at = _colorImage + (uint)pixel;
                    Wrote(at);
                    if (at >= rdram.Length) return;

                    int value = (at & 1) != 0 ? g : r;
                    rdram[at] = (byte)value;
                    if ((at & 1) != 0) _frameHidden[at >> 1] = (byte)((value & 1) * 3);
                    return;
                }

                case 2:
                {
                    uint word = (_colorImage >> 1) + (uint)pixel;
                    Wrote(word * 2);
                    if (word * 2 + 1 >= rdram.Length) return;

                    int stored = FinalCoverage(blend, coverage, memoryCoverage);
                    int color;

                    if (_colorImageFormat == 0)
                    {
                        color = ((r & ~7) << 8) | ((g & ~7) << 3) | ((b & ~7) >> 2);
                    }
                    else
                    {
                        color = (r << 8) | (stored << 5);
                        stored = 0;
                    }

                    int value = (color | (stored >> 2)) & 0xFFFF;
                    rdram[word * 2] = (byte)(value >> 8);
                    rdram[word * 2 + 1] = (byte)value;
                    _frameHidden[word] = (byte)(stored & 3);
                    return;
                }

                default:
                {
                    uint index = (_colorImage >> 2) + (uint)pixel;
                    uint at = index * 4;
                    Wrote(at);
                    if (at + 3 >= rdram.Length) return;

                    int stored = FinalCoverage(blend, coverage, memoryCoverage);
                    rdram[at] = (byte)r;
                    rdram[at + 1] = (byte)g;
                    rdram[at + 2] = (byte)b;
                    rdram[at + 3] = (byte)(stored << 5);
                    _frameHidden[index * 2] = (byte)((g & 1) * 3);
                    _frameHidden[index * 2 + 1] = 0;
                    return;
                }
            }
        }
    }
}
