namespace EmuSen.Crystal.Cheats
{
    // Galoob's Game Genie cipher of the Game Boy and the Game Gear, over six or nine upper-case hex digits - see Mercury_Cheats.md §2, EmuSen_Cheats.md §11.
    public static class GenieCipher
    {
        public static (int Address, byte Value) Decode(string hex) =>
            (((Digit(hex[5]) ^ 0x0F) << 12) | (Digit(hex[2]) << 8) | (Digit(hex[3]) << 4) | Digit(hex[4]), (byte)((Digit(hex[0]) << 4) | Digit(hex[1])));

        // Nine digits carry a compare: the seventh and ninth, rotated right by two and XORed with $BA; six carry none.
        public static byte? Compare(string hex)
        {
            if (hex.Length != 9) return null;
            byte packed = (byte)((Digit(hex[6]) << 4) | Digit(hex[8]));
            return (byte)((byte)((packed >> 2) | (packed << 6)) ^ 0xBA);
        }

        private static int Digit(char c) => System.Convert.ToInt32(c.ToString(), 16);
    }
}
