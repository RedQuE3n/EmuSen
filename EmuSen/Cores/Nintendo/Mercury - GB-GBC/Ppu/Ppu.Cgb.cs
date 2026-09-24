namespace EmuSen.Cores.Nintendo.Mercury.Video
{
    // The colour half: eight palettes a side, reached through an auto-incrementing index - see Mercury_Cgb.md §2.
    public sealed partial class Ppu
    {
        // 8 palettes x 4 colours x 2 bytes of little-endian RGB555.
        public const int PaletteRamSize = 64;

        public byte[] BgPaletteRam = new byte[PaletteRamSize];
        public byte[] ObjPaletteRam = new byte[PaletteRamSize];

        public byte BgPaletteIndex;
        public byte ObjPaletteIndex;

        public bool Cgb => _bus.Cgb;

        // A Game Boy cartridge on a Game Boy Color: DMG rendering, its shades looked up in colour palettes - see Mercury_Model.md §4.1.
        public bool Compat => _bus.DmgCompat;

        // What the boot ROM writes for a Game Boy cartridge: object palettes 0 and 1, then background palette 0 - see Mercury_Model.md §3.
        public void LoadCompatibilityPalettes(int number)
        {
            var (background, object0, object1) = CompatibilityPalettes.ForNumber(number);
            for (int i = 0; i < 4; i++)
            {
                ObjPaletteRam[i * 2] = (byte)object0[i];
                ObjPaletteRam[(i * 2) + 1] = (byte)(object0[i] >> 8);
                ObjPaletteRam[8 + (i * 2)] = (byte)object1[i];
                ObjPaletteRam[8 + (i * 2) + 1] = (byte)(object1[i] >> 8);
                BgPaletteRam[i * 2] = (byte)background[i];
                BgPaletteRam[(i * 2) + 1] = (byte)(background[i] >> 8);
            }

            // Both indices auto-incremented from 0 through what was written: sixteen object bytes, eight background ones.
            ObjPaletteIndex = 0x80 | 16;
            BgPaletteIndex = 0x80 | 8;
        }

        public byte ReadBgPaletteIndex() => (byte)(BgPaletteIndex | 0x40);

        public byte ReadObjPaletteIndex() => (byte)(ObjPaletteIndex | 0x40);

        public void WriteBgPaletteIndex(byte data) => BgPaletteIndex = (byte)(data & 0xBF);

        public void WriteObjPaletteIndex(byte data) => ObjPaletteIndex = (byte)(data & 0xBF);

        public byte ReadBgPaletteData() => BgPaletteRam[BgPaletteIndex & 0x3F];

        public byte ReadObjPaletteData() => ObjPaletteRam[ObjPaletteIndex & 0x3F];

        public void WriteBgPaletteData(byte data)
        {
            BgPaletteRam[BgPaletteIndex & 0x3F] = data;
            BgPaletteIndex = Advance(BgPaletteIndex);
        }

        public void WriteObjPaletteData(byte data)
        {
            ObjPaletteRam[ObjPaletteIndex & 0x3F] = data;
            ObjPaletteIndex = Advance(ObjPaletteIndex);
        }

        // Bit 7 makes a write step the index, which is how a game pushes a whole palette through one port.
        private static byte Advance(byte index) =>
            (index & 0x80) != 0 ? (byte)(0x80 | ((index + 1) & 0x3F)) : index;

        private void ResetCgbPalettes()
        {
            BgPaletteIndex = 0;
            ObjPaletteIndex = 0;

            // White, not black: a colour game that draws before uploading a palette should not flash a dark frame.
            for (int i = 0; i < PaletteRamSize; i++)
            {
                BgPaletteRam[i] = 0xFF;
                ObjPaletteRam[i] = 0xFF;
            }
        }

        private void WriteColorPixel(int line, int x, byte[] paletteRam, int palette, int color)
        {
            int entry = (palette * 8) + (color * 2);
            int rgb555 = paletteRam[entry] | (paletteRam[entry + 1] << 8);

            int offset = ((line * ScreenWidth) + x) * 4;

            FrameRgba[offset] = Expand(rgb555 & 0x1F);
            FrameRgba[offset + 1] = Expand((rgb555 >> 5) & 0x1F);
            FrameRgba[offset + 2] = Expand((rgb555 >> 10) & 0x1F);
            FrameRgba[offset + 3] = 0xFF;
        }

        // Five bits to eight, replicating the top three so full scale stays full - see Mercury_Cgb.md §2.1.
        private static byte Expand(int channel) => (byte)((channel << 3) | (channel >> 2));
    }
}
