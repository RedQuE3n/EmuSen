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
