using System;

namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The copy mode: four texels a step written straight to the colour image as bytes, with no combiner, blender or depth - see Mars_RdpCopy.md.
    public sealed partial class Rdp
    {
        private const int CopyCycle = 2;

        // Eight bytes a step, cut short at the span's end, and mirrored when the span runs right to left - see §1.
        private void DrawCopy((int First, int Last) rows, bool majorOnLeft, int tile, int maxLevel)
        {
            // A 32-bit colour image crashes the reference's pipeline, which then draws nothing - see §1.
            if (_colorImageSize == 3) return;

            int direction = majorOnLeft ? 1 : -1;
            int pixelBytes = _colorImageSize == 2 ? 2 : 1;
            int perStep = _colorImageSize == 0 ? 8 : 16 >> _colorImageSize;

            int ds = direction * _textureStep[0], dt = direction * _textureStep[1], dw = direction * _textureStep[2];

            for (int y = rows.First; y <= rows.Last; y++)
            {
                if (!_spanDrawn[y] || _spanRight[y] < _spanLeft[y]) continue;

                int at = y * Attributes;
                int s = _spanAttributes[at + AttributeS], t = _spanAttributes[at + AttributeT], w = _spanAttributes[at + AttributeW];

                int row = y * _colorImageWidth;
                uint pointer = _colorImage + (uint)((row + (majorOnLeft ? _spanLeft[y] : _spanRight[y])) * pixelBytes);
                uint last = _colorImage + (uint)((row + (majorOnLeft ? _spanRight[y] : _spanLeft[y])) * pixelBytes);

                for (int left = _spanRight[y] - _spanLeft[y]; left >= 0; left -= perStep)
                {
                    (int cs, int ct) = TextureCoordinates(s, t, w);
                    int from = LevelOfDetail(s + ds, t + dt, w + dw, s + (ds << 1), t + (dt << 1), w + (dw << 1), tile, maxLevel).Tile;

                    ulong texels = _colorImageSize == 0 ? 0 : CopyTexels(cs, ct, from);
                    int mask = CopyAlphaMask(texels);

                    int bytes = Math.Min(8, (int)(majorOnLeft ? last - pointer : pointer - last) + pixelBytes);
                    uint byteAt = pointer;
                    for (int k = 7; bytes > 0; k--, bytes--, byteAt = (uint)(byteAt + direction))
                    {
                        if ((mask & (1 << k)) != 0) WriteCopyByte(byteAt, (int)(texels >> (k << 3)) & 0xFF);
                    }

                    s += ds;
                    t += dt;
                    w += dw;
                    pointer = (uint)(pointer + 8 * direction);
                }
            }
        }

        // The four texels at s to s + 3, each a sixteen-bit word, and the eight bytes they make - see §2.
        private ulong CopyTexels(int s, int t, int tileIndex)
        {
            ref TextureTile tile = ref _tiles[tileIndex];

            s = (Shifted(s, tile.ShiftS) - (tile.SL << 3)) >> 5;
            t = (Shifted(t, tile.ShiftT) - (tile.TL << 3)) >> 5;
            if (tile.MaskT != 0) t = Masked(t, tile.MaskT, tile.MirrorT, tile.MirrorBitT);

            bool yuv = tile.Format == 1;
            int stride = yuv || tile.Size == 1 ? 2 : tile.Size >= 2 ? 4 : 1;
            int start = ((((tile.Line * t) & 0x1FF) + tile.Memory) << 4) & 0x1FFF;

            Span<int> along = stackalloc int[4];
            for (int k = 0; k < 4; k++)
            {
                int column = tile.MaskS != 0 ? Masked(s + k, tile.MaskS, tile.MirrorS, tile.MirrorBitS) : s + k;
                along[k] = (column * stride) & 0x1FFF;
            }

            Span<int> upper = stackalloc int[4], lower = stackalloc int[4];
            for (int k = 0; k < 4; k++) upper[k] = (start + along[k]) & 0x1FFF;

            // YUV reads its chroma from a further step along, in the lower half's read alone - see §2.2.
            lower[0] = upper[0];
            lower[2] = upper[2];
            lower[1] = yuv ? (upper[0] + ((along[1] - along[0]) << 1)) & 0x1FFF : upper[1];
            lower[3] = yuv ? (lower[1] + along[3] - along[0]) & 0x1FFF : upper[3];

            if ((t & 1) != 0)
            {
                for (int k = 0; k < 4; k++) (upper[k], lower[k]) = (upper[k] ^ 8, lower[k] ^ 8);
            }

            // A palette entry is sixteen bits, so neither the byte rule of §2.3 nor the wide rule below reads the tile's format - see §2.4.
            int size = PaletteEnabled ? 2 : tile.Size;
            bool wide = !PaletteEnabled && (tile.Format == 1 || tile.Format == 0 && tile.Size == 3);

            Span<int> words = stackalloc int[4];
            for (int k = 0; k < 4; k++)
            {
                int word = CopyBankWord(lower, k, 0);
                if (PaletteEnabled) words[k] = TextureWord(0x400 | (CopyPaletteIndex(word, lower[k] & 3, ref tile) << 2) | k);
                else if (wide || (lower[k] & 0x1000) == 0) words[k] = word;
                else words[k] = CopyBankWord(upper, k, 0x400);
            }

            ulong low = ((ulong)words[2] << 16) | (uint)words[3];
            if (size == 2) return ((((ulong)words[0] << 16) | (uint)words[1]) << 32) | low;

            ulong high = 0;
            for (int k = 0; k < 4; k++) high = (high << 8) | (uint)Replicated(words[k], lower[k] & 3, size, tile.Format, tile.Palette);
            return (high << 32) | low;
        }

        // The four addresses reach four banks at once, so a texel whose bank an earlier one claimed reads that one's word - see §2.1.
        private int CopyBankWord(ReadOnlySpan<int> addresses, int texel, int half)
        {
            int bank = (addresses[texel] >> 2) & 3;

            int first = 0;
            while (((addresses[first] >> 2) & 3) != bank) first++;

            return TextureWord(half | ((addresses[first] >> 2) & 0x3FF));
        }

        // A texel narrower than sixteen bits becomes one byte: a nibble doubled, a palette number and a nibble, or two nibbles - see §2.3.
        private static int Replicated(int word, int nibble, int size, int format, int palette)
        {
            if (size == 0)
            {
                int value = (word >> ((nibble ^ 3) << 2)) & 0xF;
                if (format == 2) return (palette << 4) | value;
                if (format != 3) return (value << 4) | value;

                int doubled = (value << 4) | value;
                return (doubled & 0xE0) | ((doubled & 0xE0) >> 3) | ((doubled & 0xC0) >> 6);
            }

            if (size != 1) return (word >> 8) & 0xFF;

            int at = ((nibble ^ 3) | 1) << 2;
            if (format != 3) return (((word >> at) & 0xF) << 4) | ((word >> (at & ~4)) & 0xF);

            int half = (word >> at) & 0xF;
            return (half << 4) | half;
        }

        // A four-bit texel's index carries the tile's palette number; a wider one takes both nibbles of its own byte - see §2.4.
        private static int CopyPaletteIndex(int word, int nibble, ref TextureTile tile)
        {
            if (tile.Size == 0) return (tile.Palette << 4) | ((word >> ((nibble ^ 3) << 2)) & 0xF);

            int at = ((nibble & 2) ^ 2) << 2;
            return ((at != 0 ? (word >> 12) & 0xF : (word >> 4) & 0xF) << 4) | ((word >> at) & 0xF);
        }

        // Alpha compare keeps a sixteen-bit image's pixels by their alpha bit and an eight-bit one's by its byte against the threshold - see §3.
        private int CopyAlphaMask(ulong texels)
        {
            if (!AlphaCompare) return 0xFF;
            if (_colorImageSize != 1 && _colorImageSize != 2) return 0;

            int mask = 0;
            for (int k = 0; k < 4; k++)
            {
                bool keep = _colorImageSize == 2
                    ? ((texels >> (48 - (k << 4))) & 1) != 0
                    : (int)((texels >> (24 - (k << 3))) & 0xFF) >= AlphaThreshold;

                if (keep) mask |= 0xC0 >> (k << 1);
            }

            return mask;
        }

        // One byte, and the hidden bits its pair shares, which only an odd address writes - see §3.
        private void WriteCopyByte(uint address, int value)
        {
            byte[] rdram = _bus.Rdram;
            Touch(address);
            if (address >= rdram.Length) return;

            rdram[address] = (byte)value;
            if ((address & 1) != 0) _bus.RdramHidden[address >> 1] = (byte)((value & 1) * 3);
        }
    }
}
