using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // Texture coordinates, perspective division, and a point-sampled texel from texture memory - see Mars_RdpTextures.md §4 and §5.
    public sealed partial class Rdp
    {
        private static readonly int[] DivideTable = BuildDivideTable();

        // A five-bit channel widened to eight by repeating its top three bits.
        private static readonly byte[] FiveToEight = BuildFiveToEight();

        // A texture rectangle is a rectangle carrying s and t and their steps, the steps swapped between x and y when flipped - see §4.1.
        private void TexturedRectangle(bool flipped)
        {
            ulong word = _command[0], coordinates = _command[1];
            ClearAttributes();

            int dsdx = ((short)(coordinates >> 16) << 11) / _scale, dtdy = ((short)coordinates << 11) / _scale;
            _attributeValue[AttributeS] = (int)(coordinates >> 48) << 16;
            _attributeValue[AttributeT] = (int)((coordinates >> 32) & 0xFFFF) << 16;

            if (flipped)
            {
                _attributeDx[AttributeT] = dtdy;
                _attributeDe[AttributeS] = _attributeDy[AttributeS] = dsdx;
            }
            else
            {
                _attributeDx[AttributeS] = dsdx;
                _attributeDe[AttributeT] = _attributeDy[AttributeT] = dtdy;
            }

            Draw(WalkRectangle(word), majorOnLeft: true, tile: (int)(word >> 24) & 7, maxLevel: 0);
        }

        private (int S, int T) TextureCoordinates(int s, int t, int w)
        {
            (s, t) = DividedCoordinates(s, t, w);
            return (ClampCoordinate(s), ClampCoordinate(t));
        }

        // s and t over w, or s and t alone, to seventeen bits and two overflow flags, before the clamp to sixteen - see §4.2.
        private (int S, int T) DividedCoordinates(int s, int t, int w)
        {
            s >>= 16;
            t >>= 16;
            w >>= 16;

            if (!Perspective) return ((short)s & 0x1FFFF, (short)t & 0x1FFFF);

            int flags = (short)w <= 0 ? 2 << 17 : 0;
            int entry = DivideTable[w & 0x7FFF];
            int reciprocal = entry >> 4, shift = entry & 0xF;

            return (Divided((short)s * reciprocal, shift) | flags, Divided((short)t * reciprocal, shift) | flags);
        }

        // The product scaled by the reciprocal's shift, and flagged over or under when bits beyond the range differ - see §4.2.
        private static int Divided(int product, int shift)
        {
            int range = ((1 << 30) - 1) & -((1 << 29) >> shift);
            int outside = product & range;

            int scaled;
            if (shift != 0xE)
            {
                product >>= 13 - shift;
                scaled = product;
            }
            else
            {
                scaled = product << 1;
            }

            int flags = outside == range || outside == 0 ? 0 : (product & (1 << 29)) == 0 ? 2 << 17 : 1 << 17;
            return (scaled & 0x1FFFF) | flags;
        }

        private static int ClampCoordinate(int value)
        {
            if ((value & 0x40000) != 0) return 0x7FFF;
            if ((value & 0x20000) != 0) return 0x8000;

            return (value & 0x18000) switch
            {
                0x8000 => 0x7FFF,
                0x10000 => 0x8000,
                _ => value & 0xFFFF,
            };
        }

        // Shifted and made relative to the tile's corner, then sampled at one texel, or at four when filtering or a palette asks - see §5 and Mars_RdpFiltering.md §1.
        private Color Texel(int s, int t, int tileIndex) => Texel(s, t, tileIndex, BilinearFirstCycle, convert: false, default);

        // The second cycle's texel may filter or convert differently, and convert the first cycle's texel instead of its own - see Mars_RdpTwoCycle.md §2.
        private Color Texel(int s, int t, int tileIndex, bool bilinear, bool convert, Color previous)
        {
            ref TextureTile tile = ref _tiles[tileIndex];

            s = Shifted(s, tile.ShiftS);
            t = Shifted(t, tile.ShiftT);
            bool beyondS = (s >> 3) >= tile.SH, beyondT = (t >> 3) >= tile.TH;

            s -= tile.SL << 3;
            t -= tile.TL << 3;

            return SampleFour || PaletteEnabled
                ? FourTexels(s, t, beyondS, beyondT, ref tile, bilinear, convert, previous)
                : PointTexel(s, t, beyondS, beyondT, ref tile, bilinear, convert, previous);
        }

        // Clamped, masked and mirrored, fetched, and converted from YUV unless filtered - see §5.
        private Color PointTexel(int s, int t, bool beyondS, bool beyondT, ref TextureTile tile, bool bilinear, bool convert, Color previous)
        {
            // Converting passes the first cycle's texel through, or its blue as all four channels when filtering - see Mars_RdpTwoCycle.md §2.
            if (convert) return bilinear ? new Color { R = previous.B, G = previous.B, B = previous.B, A = previous.B } : Converted(SignedNine(previous), SignedNine(previous).B);

            s = Clamped(s, tile.ClampsS, beyondS, tile.ClampLimitS);
            t = Clamped(t, tile.ClampsT, beyondT, tile.ClampLimitT);

            if (tile.MaskS != 0) s = Masked(s, tile.MaskS, tile.MirrorS, tile.MirrorBitS);
            if (tile.MaskT != 0) t = Masked(t, tile.MaskT, tile.MirrorT, tile.MirrorBitT);

            Color texel = FetchTexel(s, t & 0xFF, ref tile);

            // The reference also cuts red and green to nine bits here, which the combiner's sign extension makes invisible - see §5.3.
            return bilinear ? texel : Converted(texel, texel.B);
        }

        private static Color SignedNine(Color color) => new() { R = (color.R << 23) >> 23, G = (color.G << 23) >> 23, B = (color.B << 23) >> 23, A = color.A };

        // The luma plus the chroma through the four conversion constants, the chroma and luma possibly from different texels - see §5.3.
        private Color Converted(Color chroma, int luma) => new()
        {
            R = (luma + ((_k0 * chroma.G + 0x80) >> 8)) & 0x1FF,
            G = (luma + ((_k1 * chroma.R + _k2 * chroma.G + 0x80) >> 8)) & 0x1FF,
            B = (luma + ((_k3 * chroma.R + 0x80) >> 8)) & 0x1FF,
            A = luma & 0x1FF,
        };

        private static int Shifted(int coordinate, int shift) =>
            shift < 11 ? (short)coordinate >> shift : (short)(coordinate << (16 - shift));

        private static int Clamped(int coordinate, bool clamps, bool beyond, int limit)
        {
            if (!clamps) return coordinate >> 5;
            if (beyond) return limit;
            return (coordinate & 0x10000) == 0 ? coordinate >> 5 : 0;
        }

        private static int Masked(int coordinate, int mask, bool mirror, int mirrorBit)
        {
            if (mirror && ((coordinate >> mirrorBit) & 1) != 0) coordinate = ~coordinate;
            return coordinate & (0xFFFF >> (16 - mask)) & 0x3FF;
        }

        // Each format and size read from its place in texture memory, the row already cut to eight bits or not; formats past four read as intensity - see §5.1.
        private Color FetchTexel(int s, int row, ref TextureTile tile)
        {
            int line = tile.Line * row + tile.Memory;
            bool odd = (row & 1) != 0;
            int byteSwap = odd ? 4 : 0, wordSwap = odd ? 2 : 0;

            int format = tile.Format < 5 ? tile.Format : 4;
            switch ((format << 2) | tile.Size)
            {
                case 0x0 or 0x10:
                {
                    int value = Nibble(((line << 4) + s) >> 1, s, byteSwap);
                    return Gray(value | (value << 4));
                }

                case 0x1 or 0x9 or 0x11:
                    return Gray(TextureMemory[((line << 3) + s ^ byteSwap) & 0xFFF]);

                case 0x2:
                {
                    int value = TextureWord(((line << 2) + s ^ wordSwap) & 0x7FF);
                    return new Color
                    {
                        R = FiveToEight[value >> 11],
                        G = FiveToEight[(value >> 6) & 0x1F],
                        B = FiveToEight[(value >> 1) & 0x1F],
                        A = (value & 1) != 0 ? 0xFF : 0,
                    };
                }

                case 0x3:
                {
                    int index = ((line << 2) + s ^ wordSwap) & 0x3FF;
                    int first = TextureWord(index), second = TextureWord(index | 0x400);
                    return new Color { R = first >> 8, G = first & 0xFF, B = second >> 8, A = second & 0xFF };
                }

                case 0x4 or 0x5:
                {
                    int value = TextureMemory[((line << 3) + s ^ byteSwap) & 0x7FF];
                    if (tile.Size == 0) value = (value & 0xF0) | ((value & 0xF0) >> 4);
                    return new Color { R = value - 0x80, G = value - 0x80, B = value, A = value };
                }

                case 0x6 or 0x7:
                {
                    int bytes = (line << 3) + s;
                    int chroma = TextureWord(((bytes >> 1) ^ wordSwap) & 0x3FF);
                    var texel = new Color { R = (chroma >> 8) - 0x80, G = (chroma & 0xFF) - 0x80 };

                    if (tile.Size == 2 || (s & 1) != 0)
                    {
                        texel.B = texel.A = TextureMemory[((bytes ^ byteSwap) & 0x7FF) | 0x800];
                    }
                    else
                    {
                        int luma = TextureWord((((bytes >> 1) ^ wordSwap) & 0x3FF) | 0x400);
                        texel.B = luma >> 8;
                        texel.A = ((luma >> 8) & 0xF) | (luma & 0xF0);
                    }

                    return texel;
                }

                case 0x8:
                {
                    int value = Nibble(((line << 4) + s) >> 1, s, byteSwap);
                    return Gray((tile.Palette << 4) | value);
                }

                case 0xC:
                {
                    int value = Nibble(((line << 4) + s) >> 1, s, byteSwap);
                    int intensity = value & 0xE;
                    return new Color
                    {
                        R = ((intensity << 4) | (intensity << 1) | (intensity >> 2)) & 0xFF,
                        G = ((intensity << 4) | (intensity << 1) | (intensity >> 2)) & 0xFF,
                        B = ((intensity << 4) | (intensity << 1) | (intensity >> 2)) & 0xFF,
                        A = (value & 1) != 0 ? 0xFF : 0,
                    };
                }

                case 0xD:
                {
                    int value = TextureMemory[((line << 3) + s ^ byteSwap) & 0xFFF];
                    int intensity = (value & 0xF0) | ((value & 0xF0) >> 4);
                    return new Color { R = intensity, G = intensity, B = intensity, A = ((value & 0xF) << 4) | (value & 0xF) };
                }

                case 0xE:
                {
                    int value = TextureWord(((line << 2) + s ^ wordSwap) & 0x7FF);
                    return new Color { R = value >> 8, G = value >> 8, B = value >> 8, A = value & 0xFF };
                }

                default:
                {
                    int value = TextureWord(((line << 2) + s ^ wordSwap) & 0x7FF);
                    return new Color { R = value >> 8, G = value & 0xFF, B = value >> 8, A = value & 0xFF };
                }
            }
        }

        // The high nibble for an even s and the low for an odd one.
        private int Nibble(int index, int s, int byteSwap)
        {
            int value = TextureMemory[(index ^ byteSwap) & 0xFFF];
            return (s & 1) != 0 ? value & 0xF : value >> 4;
        }

        private static Color Gray(int value) => new() { R = value & 0xFF, G = value & 0xFF, B = value & 0xFF, A = value & 0xFF };

        // A normalised w's top six bits pick a reciprocal and a slope, which the next eight bits interpolate - see §4.2.
        private static int[] BuildDivideTable()
        {
            var table = new int[0x8000];

            for (int w = 0; w < table.Length; w++)
            {
                int k = 1;
                while (k <= 14 && ((w << k) & 0x8000) == 0) k++;
                int shift = k - 1;

                int normalised = (w << shift) & 0x3FFF;
                int fraction = (normalised & 0xFF) << 2;
                int segment = normalised >> 8;

                int point = ReciprocalPoint(segment);
                int slope = ReciprocalPoint(segment + 1) - point;
                int reciprocal = (((slope * fraction) >> 10) + point) & 0x7FFF;

                table[w] = shift | (reciprocal << 4);
            }

            return table;
        }

        // 2^20 over 64 to 128, rounded, except that the seventh is one lower - see §4.2.
        private static int ReciprocalPoint(int segment) => segment == 6 ? 0x3A83 : (int)Math.Round(0x10_0000 / (64.0 + segment));

        private static byte[] BuildFiveToEight()
        {
            var table = new byte[32];
            for (int i = 0; i < table.Length; i++) table[i] = (byte)((i << 3) | ((i >> 2) & 7));
            return table;
        }
    }
}
