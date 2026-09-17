namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // The texture image, the eight tiles, and loads from RDRAM into texture memory - see Mars_RdpTextures.md §1 to §3.
    public sealed partial class Rdp
    {
        private struct TextureTile
        {
            public int Format, Size, Line, Memory, Palette;
            public bool ClampS, MirrorS, ClampT, MirrorT;
            public int MaskS, ShiftS, MaskT, ShiftT;

            // Quarter texels, as the tile-size and load commands carry them.
            public int SL, TL, SH, TH;

            // A tile with no mask always clamps, and a mirror wraps at no more than ten bits - see §5.2.
            public bool ClampsS => ClampS || MaskS == 0;
            public bool ClampsT => ClampT || MaskT == 0;
            public int MirrorBitS => MaskS <= 10 ? MaskS : 10;
            public int MirrorBitT => MaskT <= 10 ? MaskT : 10;
            public int ClampLimitS => ((SH >> 2) - (SL >> 2)) & 0x3FF;
            public int ClampLimitT => ((TH >> 2) - (TL >> 2)) & 0x3FF;
        }

        private readonly TextureTile[] _tiles = new TextureTile[8];

        private int _textureImageSize;
        private int _textureImageWidth;
        private uint _textureImage;

        private void TextureImage(ulong word)
        {
            _textureImageSize = (int)(word >> 51) & 3;
            _textureImageWidth = (int)((word >> 32) & 0x3FF) + 1;
            _textureImage = (uint)word & 0x00FF_FFFF;
        }

        private void Tile(ulong word)
        {
            ref TextureTile tile = ref _tiles[(word >> 24) & 7];
            tile.Format = (int)(word >> 53) & 7;
            tile.Size = (int)(word >> 51) & 3;
            tile.Line = (int)(word >> 41) & 0x1FF;
            tile.Memory = (int)(word >> 32) & 0x1FF;
            tile.Palette = (int)(word >> 20) & 0xF;
            tile.ClampT = ((word >> 19) & 1) != 0;
            tile.MirrorT = ((word >> 18) & 1) != 0;
            tile.MaskT = (int)(word >> 14) & 0xF;
            tile.ShiftT = (int)(word >> 10) & 0xF;
            tile.ClampS = ((word >> 9) & 1) != 0;
            tile.MirrorS = ((word >> 8) & 1) != 0;
            tile.MaskS = (int)(word >> 4) & 0xF;
            tile.ShiftS = (int)word & 0xF;
        }

        private ref TextureTile TileSize(ulong word)
        {
            ref TextureTile tile = ref _tiles[(word >> 24) & 7];
            tile.SL = Quarters(word >> 44);
            tile.TL = Quarters(word >> 32);
            tile.SH = Quarters(word >> 12);
            tile.TH = Quarters(word);
            return ref tile;
        }

        // A tile load walks rows of image texels four quarters apart; a block load walks one row whose t steps by a slope - see §3.1.
        private void Load(ulong word, bool block)
        {
            ref TextureTile tile = ref TileSize(word);

            int top, bottom, first, last, right;
            if (block)
            {
                int row = tile.TL & 0x3FF;
                (top, bottom, first, last, right) = (row << 2, (row << 2) | 3, row, row, tile.SH);
            }
            else
            {
                (top, bottom, first, last, right) = (tile.TL, tile.TH | 3, tile.TL >> 2, tile.TH >> 2, tile.SH >> 2);
            }

            // A block's left column is a signed twelve-bit whole number, which moves the image pointer but not the texel coordinate - see §3.1.
            int left = block ? (tile.SL << 20) >> 20 : tile.SL >> 2;
            int sStep = (0x200 >> (block ? _textureImageSize + 2 : _textureImageSize)) << 16;
            int tStep = block ? (tile.TH << 8) & ~0x1F : 0;

            for (int row = first; row <= last; row++)
            {
                bool valid = System.Math.Max(row << 2, top) < System.Math.Min((row << 2) + 4, bottom);
                int t = ((tile.TL << 3) << 16) + (block ? 0 : (row - first) * (0x20 << 16));
                LoadRow(ref tile, row, valid ? right : 0, left, (tile.SL << 3) << 16, t, sStep, tStep, block);
            }
        }

        // An eight-byte window of the image goes to four words of texture memory per step; a four-bit image loads nothing - see §3.2.
        private void LoadRow(ref TextureTile tile, int row, int right, int left, int s, int t, int sStep, int tStep, bool block)
        {
            (int imageAdvance, int texelAdvance) = _textureImageSize switch { 1 => (8, 8), 2 => (8, 4), 3 => (8, 2), _ => (0, 0) };
            if (texelAdvance == 0) return;

            int layout = tile.Format == 1 ? 0 : tile.Format == 0 && tile.Size == 3 ? 1 : 2;
            int pointer = (int)_textureImage + (((_textureImageWidth * row + left) << _textureImageSize) >> 1);
            int length = (right - left + 1) & 0xFFF;

            for (int n = 0; n < length; n += texelAdvance)
            {
                int ts = (short)((s >> 16) & 0xFFFF) - (tile.SL << 3);
                int tt = (short)((t >> 16) & 0xFFFF) - (tile.TL << 3);
                ts >>= block ? 3 : 5;
                tt >>= block ? 3 : 5;

                ulong window = ImageWindow(pointer);
                StoreTexels(ref tile, ts, tt, window, layout);

                s = (s + sStep) & ~0x1F;
                t = (t + tStep) & ~0x1F;
                pointer += imageAdvance;
            }
        }

        // The eight bytes from the image pointer, read through the two 32-bit words that hold them and a following pair - see §3.2.
        private ulong ImageWindow(int pointer)
        {
            int word = (pointer >> 2) & ~1;
            ulong first = ((ulong)ImageWord(word) << 32) | ImageWord(word + 1);
            ulong second = ((ulong)ImageWord(word + 2) << 32) | ImageWord(word + 3);

            int offset = (pointer & 7) * 8;
            return offset == 0 ? first : (first << offset) | (second >> (64 - offset));
        }

        private uint ImageWord(int index)
        {
            long at = (index & 0x3F_FFFF) * 4L;
            byte[] rdram = _bus.Rdram;
            return at + 3 < rdram.Length ? (uint)((rdram[at] << 24) | (rdram[at + 1] << 16) | (rdram[at + 2] << 8) | rdram[at + 3]) : 0;
        }

        // Four consecutive words sorted into four banks by their low two bits, with odd texel rows swapping pairs of banks - see §3.2.
        private void StoreTexels(ref TextureTile tile, int s, int t, ulong window, int layout)
        {
            int words = tile.Size == 1 || tile.Format == 1 ? s >> 1 : tile.Size >= 2 ? s : s >> 2;
            words &= 0x7FF;

            int row = tile.Line * t + tile.Memory;
            int first = ((row << 2) + words) & 0x7FD;
            bool high = (first & 0x400) != 0;
            bool oddRow = (t & 1) != 0;

            System.Span<int> bank = stackalloc int[4];
            for (int i = 0; i < 4; i++)
            {
                int index = ((first + i) & 0x7FF) ^ (oddRow ? 2 : 0);
                bank[index & 3] = index & 0x3FF;
            }

            if (layout == 2)
            {
                int half = high ? 0x400 : 0;
                bool swapped = oddRow;
                WriteTextureWord(bank[0] | half, (int)(window >> (swapped ? 16 : 48)));
                WriteTextureWord(bank[1] | half, (int)(window >> (swapped ? 0 : 32)));
                WriteTextureWord(bank[2] | half, (int)(window >> (swapped ? 48 : 16)));
                WriteTextureWord(bank[3] | half, (int)(window >> (swapped ? 32 : 0)));
                return;
            }

            // YUV takes alternate bytes into each half, 32-bit RGBA alternate words - see §3.2.
            uint low, upper;
            if (layout == 0)
            {
                low = (uint)(((window >> 56) & 0xFF) << 24 | ((window >> 40) & 0xFF) << 16 | ((window >> 24) & 0xFF) << 8 | ((window >> 8) & 0xFF));
                upper = (uint)(((window >> 48) & 0xFF) << 24 | ((window >> 32) & 0xFF) << 16 | ((window >> 16) & 0xFF) << 8 | (window & 0xFF));
            }
            else
            {
                low = (uint)(((window >> 48) << 16) | ((window >> 16) & 0xFFFF));
                upper = (uint)(((window >> 32) & 0xFFFF) << 16 | (window & 0xFFFF));
            }

            bool alternate = ((words & 2) != 0) ^ oddRow;
            int one = alternate ? bank[2] : bank[0], two = alternate ? bank[3] : bank[1];
            WriteTextureWord(one, (int)(low >> 16));
            WriteTextureWord(two, (int)low);
            WriteTextureWord(one | 0x400, (int)(upper >> 16));
            WriteTextureWord(two | 0x400, (int)upper);
        }

        private void WriteTextureWord(int index, int value)
        {
            TextureMemory[index * 2] = (byte)(value >> 8);
            TextureMemory[index * 2 + 1] = (byte)value;
        }

        private int TextureWord(int index) => (TextureMemory[index * 2] << 8) | TextureMemory[index * 2 + 1];
    }
}
