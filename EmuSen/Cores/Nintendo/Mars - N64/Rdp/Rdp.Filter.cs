using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // Four texels around a coordinate, from texture memory or through a palette, filtered or converted - see Mars_RdpFiltering.md.
    public sealed partial class Rdp
    {
        // The fraction picks one of the square's two triangles; YUV's chroma is half as wide, so it may pick the other - see §2.
        private Color FourTexels(int s, int t, bool beyondS, bool beyondT, ref TextureTile tile, bool bilinear, bool convert, Color previous)
        {
            int sFraction = tile.ClampsS && (beyondS || (s & 0x10000) != 0) ? 0 : s & 0x1F;
            int tFraction = tile.ClampsT && (beyondT || (t & 0x10000) != 0) ? 0 : t & 0x1F;

            s = Clamped(s, tile.ClampsS, beyondS, tile.ClampLimitS);
            t = Clamped(t, tile.ClampsT, beyondT, tile.ClampLimitT);
            int sStep = Wrapped(ref s, tile.MaskS, tile.MirrorS, tile.MirrorBitS, -1);
            int tStep = Wrapped(ref t, tile.MaskT, tile.MirrorT, tile.MirrorBitT, 0xFF);

            bool upper = ((sFraction + tFraction) & 0x20) != 0;
            int sFractionRg = tile.Format == 1 ? (sFraction >> 1) | ((s & 1) << 4) : sFraction;
            bool upperRg = ((sFractionRg + tFraction) & 0x20) != 0;

            // Converting without filtering reads no texel at all.
            if (convert && !bilinear) return Converted(SignedNine(previous), SignedNine(previous).B);

            Span<Color> texels = stackalloc Color[4];
            if (!SampleFour) NearestPaletteTexels(texels, s, t, ref tile, upperRg);
            else if (PaletteEnabled) PaletteTexels(texels, s, sStep, t, tStep, ref tile, upperRg);
            else Texels(texels, s, sStep, t, tStep, ref tile);

            // Luma crosses to the other corner when the two triangles differ.
            if (upper != upperRg && (tile.Format == 1 && tile.Size < 2 || PaletteEnabled))
            {
                (texels[0].B, texels[3].B, texels[1].B, texels[2].B) = (texels[3].B, texels[0].B, texels[2].B, texels[1].B);
                (texels[0].A, texels[3].A, texels[1].A, texels[2].A) = (texels[3].A, texels[0].A, texels[2].A, texels[1].A);
            }

            if (!bilinear) return Converted(upperRg ? texels[3] : texels[0], (upper ? texels[3] : texels[0]).B);

            bool center = MidTexel && sFraction == 0x10 && tFraction == 0x10;
            bool centerRg = MidTexel && sFractionRg == 0x10 && tFraction == 0x10;

            // Filtering while converting weights the corners' differences by the first cycle's texel instead of the fraction - see Mars_RdpTwoCycle.md §2.
            if (convert)
            {
                Color p = SignedNine(previous);
                return new Color
                {
                    R = ConvertedCorners(texels[0].R, texels[1].R, texels[2].R, texels[3].R, p, upperRg, centerRg),
                    G = ConvertedCorners(texels[0].G, texels[1].G, texels[2].G, texels[3].G, p, upperRg, centerRg),
                    B = ConvertedCorners(texels[0].B, texels[1].B, texels[2].B, texels[3].B, p, upper, center),
                    A = ConvertedCorners(texels[0].A, texels[1].A, texels[2].A, texels[3].A, p, upper, center),
                };
            }

            return new Color
            {
                R = Interpolated(texels[0].R, texels[1].R, texels[2].R, texels[3].R, sFractionRg, tFraction, upperRg, centerRg),
                G = Interpolated(texels[0].G, texels[1].G, texels[2].G, texels[3].G, sFractionRg, tFraction, upperRg, centerRg),
                B = Interpolated(texels[0].B, texels[1].B, texels[2].B, texels[3].B, sFraction, tFraction, upper, center),
                A = Interpolated(texels[0].A, texels[1].A, texels[2].A, texels[3].A, sFraction, tFraction, upper, center),
            };
        }

        private static int ConvertedCorners(int t0, int t1, int t2, int t3, Color p, bool upper, bool center)
        {
            if (center) return p.B + ((p.R * (t2 - t3) + p.G * (t1 - t3) + ((~t3 + t0) << 6) + 0xC0) >> 8);
            if (upper) return p.B + ((p.R * (t2 - t3) + p.G * (t1 - t3) + 0x80) >> 8);
            return p.B + ((p.R * (t1 - t0) + p.G * (t2 - t0) + 0x80) >> 8);
        }

        // Masked and mirrored as for one texel, and the step to the next: back to zero past the mask, or reversed and held at a mirror's turn - see §2.1.
        private static int Wrapped(ref int coordinate, int mask, bool mirror, int mirrorBit, int wrapBits)
        {
            if (mask == 0) return 1;

            int bits = (0xFFFF >> (16 - mask)) & 0x3FF;
            if (mirror)
            {
                int wrap = (coordinate >> mirrorBit) & 1;
                coordinate = (coordinate ^ -wrap) & bits;
                return ((coordinate - wrap) & bits) == bits ? 0 : 1 - (wrap << 1);
            }

            coordinate &= bits;
            return coordinate == bits ? -(coordinate & wrapBits) : 1;
        }

        // Three texels weighted by the triangle the fraction lies in, or the rounded average of all four at the mid-texel - see §3.
        private static int Interpolated(int t0, int t1, int t2, int t3, int sFraction, int tFraction, bool upper, bool center)
        {
            if (center) return (t0 + t1 + t2 + t3 + 2) >> 2;
            if (upper) return t3 + (((0x20 - sFraction) * (t2 - t3) + (0x20 - tFraction) * (t1 - t3) + 0x10) >> 5);
            return t0 + ((sFraction * (t1 - t0) + tFraction * (t2 - t0) + 0x10) >> 5);
        }

        // The texel, the one to its right, the one below and the one diagonal; YUV reads chroma a further step along - see §2.2.
        private void Texels(Span<Color> texels, int s0, int sStep, int t0, int tStep, ref TextureTile tile)
        {
            int row0 = t0 & 0xFF, row1 = row0 + tStep, s1 = s0 + sStep;

            texels[0] = FetchTexel(s0, row0, ref tile);
            texels[2] = FetchTexel(s0, row1, ref tile);

            if (tile.Format != 1)
            {
                texels[1] = FetchTexel(s1, row0, ref tile);
                texels[3] = FetchTexel(s1, row1, ref tile);
                return;
            }

            texels[1] = FetchTexel(s1 + sStep, row0, ref tile);
            texels[3] = FetchTexel(s1 + sStep, row1, ref tile);
            if (tile.Size < 2) return;

            // Sixteen- and thirty-two-bit YUV take luma from the texel itself.
            Color luma1 = FetchTexel(s1, row0, ref tile), luma3 = FetchTexel(s1, row1, ref tile);
            (texels[1].B, texels[1].A, texels[3].B, texels[3].A) = (luma1.B, luma1.A, luma3.B, luma3.A);
        }

        // Each of the four texels' palette entries, taken from its own bank of four - see §4.1.
        private void PaletteTexels(Span<Color> texels, int s0, int sStep, int t0, int tStep, ref TextureTile tile, bool upperRg)
        {
            int row0 = t0 & 0xFF, row1 = row0 + tStep;
            int s1 = s0 + (tile.Format == 1 ? sStep << 1 : sStep);

            texels[0] = PaletteColor(PaletteIndex(s0, row0, ref tile) << 2, upperRg);
            texels[1] = PaletteColor((PaletteIndex(s1, row0, ref tile) << 2) | 1, upperRg);
            texels[2] = PaletteColor((PaletteIndex(s0, row1, ref tile) << 2) | 2, upperRg);
            texels[3] = PaletteColor((PaletteIndex(s1, row1, ref tile) << 2) | 3, upperRg);
        }

        // One texel's palette entry from all four banks - see §4.1.
        private void NearestPaletteTexels(Span<Color> texels, int s, int t, ref TextureTile tile, bool upperRg)
        {
            int entry = PaletteIndex(s, t, ref tile) << 2;
            for (int bank = 0; bank < 4; bank++) texels[bank] = PaletteColor(entry | bank, upperRg);
        }

        // A nibble with the tile's palette number, a byte, or a word's high byte; YUV always reads bytes - see §4.1.
        private int PaletteIndex(int s, int row, ref TextureTile tile)
        {
            int line = tile.Line * row + tile.Memory;
            bool odd = (row & 1) != 0;
            int byteSwap = odd ? 4 : 0, wordSwap = odd ? 2 : 0;
            bool yuv = tile.Format == 1;

            if (tile.Size == 0 && !yuv)
            {
                int value = TextureMemory[(((line << 4) + s) >> 1 ^ byteSwap) & 0x7FF];
                return (tile.Palette << 4) | ((s & 1) != 0 ? value & 0xF : value >> 4);
            }

            if (tile.Size >= 2 && !yuv) return TextureWord(((line << 2) + s ^ wordSwap) & 0x3FF) >> 8;

            int bytes = TextureMemory[((line << 3) + s ^ byteSwap) & 0x7FF];
            return tile.Size == 0 ? (tile.Palette << 4) | (bytes >> 4) : bytes;
        }

        // The palette's upper half of texture memory, its bank reversed in the upper chroma triangle, as RGBA16 or IA16 - see §4.1.
        private Color PaletteColor(int entry, bool upperRg)
        {
            int value = TextureWord(0x400 | (upperRg ? entry ^ 3 : entry));

            return PaletteIntensityAlpha
                ? new Color { R = value >> 8, G = value >> 8, B = value >> 8, A = value & 0xFF }
                : new Color
                {
                    R = FiveToEight[value >> 11],
                    G = FiveToEight[(value >> 6) & 0x1F],
                    B = FiveToEight[(value >> 1) & 0x1F],
                    A = (value & 1) != 0 ? 0xFF : 0,
                };
        }
    }
}
