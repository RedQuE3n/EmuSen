using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic Genesis, 32X or Sega CD image with the header Nephrite reads - never real game data - see Nephrite_Native.md §2.2.
    public static class SyntheticMdRom
    {
        public static byte[] Cartridge(string system = "SEGA GENESIS", string region = "JUE", int saveBytes = 0, int size = 0x20000)
        {
            var rom = new byte[size];
            Put(rom, 0x100, system, 16);
            Put(rom, 0x120, "EMUSEN SYNTHETIC", 48);
            Put(rom, 0x150, "EMUSEN SYNTHETIC", 48);
            Put(rom, 0x180, "GM 00000000-00", 14);
            Put(rom, 0x190, "J6", 16);
            Put(rom, 0x1F0, region, 3);
            if (saveBytes > 0)
            {
                // Battery RAM on the odd lane at $200001, as the header's "RA" $F8 declares it.
                rom[0x1B0] = (byte)'R';
                rom[0x1B1] = (byte)'A';
                rom[0x1B2] = 0xF8;
                rom[0x1B3] = 0x20;
                BigEndian(rom, 0x1B4, 0x200001);
                BigEndian(rom, 0x1B8, (uint)(0x200001 + 2 * (saveBytes - 1)));
            }
            return rom;
        }

        // A data track of 2,048-byte sectors whose system area carries the signature and the region.
        public static byte[] Disc(string region = "U", int size = 0x10000)
        {
            var iso = new byte[size];
            Put(iso, 0, "SEGADISCSYSTEM", 16);
            Put(iso, 0x100, "SEGA GENESIS", 16);
            Put(iso, 0x1F0, region, 3);
            return iso;
        }

        private static void Put(byte[] b, int at, string text, int width) => Encoding.ASCII.GetBytes(text.PadRight(width)[..width]).CopyTo(b, at);

        private static void BigEndian(byte[] b, int at, uint v)
        {
            b[at] = (byte)(v >> 24);
            b[at + 1] = (byte)(v >> 16);
            b[at + 2] = (byte)(v >> 8);
            b[at + 3] = (byte)v;
        }
    }
}
