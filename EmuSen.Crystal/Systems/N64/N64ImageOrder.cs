namespace EmuSen.Crystal.Systems.N64
{
    // A Nintendo 64 image's three container orders, told by its first word and not its extension - see Mars_Rom.md §1.1.
    public static class N64ImageOrder
    {
        public const uint Magic = 0x80371240;

        // The image in the halfword-swapped .v64 order whatever order it is in, or the file itself when it is no N64 image or a truncated one.
        public static byte[] ToByteSwapped(byte[] file)
        {
            if (file.Length < 4) return file;
            uint word = (uint)((file[0] << 24) | (file[1] << 16) | (file[2] << 8) | file[3]);
            switch (word)
            {
                case Magic when file.Length % 2 == 0:
                    return Swapped(file, 1);
                case 0x40123780 when file.Length % 4 == 0:
                    return Swapped(file, 2);
                default:
                    return file;
            }
        }

        // Each byte from the one whose index differs by the given bit: 1 swaps halfwords' bytes, 2 a word's halves.
        private static byte[] Swapped(byte[] file, int bit)
        {
            var into = new byte[file.Length];
            for (int i = 0; i < into.Length; i++) into[i] = file[i ^ bit];
            return into;
        }
    }
}
