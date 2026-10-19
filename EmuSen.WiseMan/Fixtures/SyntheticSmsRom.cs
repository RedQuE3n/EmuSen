using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic Master System or SG-1000 cartridge: Sega's header with its checksum and region, a boot stub and a pad reader - never real game data - see Endou_Native.md §4.
    public static class SyntheticSmsRom
    {
        // DI; IM 1; LD SP,nn; then forever IN A,($DC); LD ($C000),A; IN A,($DD); LD ($C001),A - the pads' two ports copied to the first bytes of RAM.
        public static byte[] Stub(ushort stack) => new byte[] { 0xF3, 0xED, 0x56, 0x31, (byte)stack, (byte)(stack >> 8), 0xDB, 0xDC, 0x32, 0x00, 0xC0, 0xDB, 0xDD, 0x32, 0x01, 0xC0, 0x18, 0xF4 };

        public const int HeaderAt = 0x7FF0;

        // A Master System cartridge of at least 32 KiB with Sega's header at $7FF0; region $3 Japan, $4 export (SMS Power!'s "ROM Header").
        public static byte[] Cartridge(int region = 0x4, int size = 0x8000, ushort product = 7026, int version = 0, bool rightChecksum = true)
        {
            if (size < 0x8000) throw new ArgumentOutOfRangeException(nameof(size), "the header at $7FF0 needs 32 KiB");
            var rom = new byte[size];
            Stub(0xDFF0).CopyTo(rom, 0);
            Encoding.ASCII.GetBytes("TMR SEGA").CopyTo(rom, HeaderAt);
            rom[HeaderAt + 12] = Bcd(product % 100);
            rom[HeaderAt + 13] = Bcd(product / 100 % 100);
            rom[HeaderAt + 14] = (byte)((product / 10000) << 4 | (version & 0xF));
            rom[HeaderAt + 15] = (byte)(region << 4 | SizeCode(size));
            ushort sum = Checksum(rom, HeaderAt, SizeCode(size));
            if (!rightChecksum) sum ^= 0x5A5A;
            rom[HeaderAt + 10] = (byte)sum;
            rom[HeaderAt + 11] = (byte)(sum >> 8);
            return rom;
        }

        // An SG-1000 cartridge: no header, the stub with its stack in the SG-1000's 1 KiB of RAM.
        public static byte[] Sg1000(int size = 0x4000)
        {
            var rom = new byte[size];
            Stub(0xC3F0).CopyTo(rom, 0);
            return rom;
        }

        // The image as a copier wrote it: 512 bytes of its own before the cartridge.
        public static byte[] WithCopierHeader(byte[] rom) => new byte[512].Concat(rom).ToArray();

        // The size code for the image's size, as SMS Power!'s "ROM Header" tabulates it; 256 KiB and above as $0-$2.
        public static int SizeCode(int size) => size switch
        {
            0x8000 => 0xC,
            0x10000 => 0xE,
            0x20000 => 0xF,
            0x40000 => 0x0,
            0x80000 => 0x1,
            _ => 0x2,
        };

        // The 16-bit sum over the size code's range, the checksum's two bytes left out, as Sega's Mark III manual states it - see Endou_Native.md §2.2.
        public static ushort Checksum(byte[] rom, int headerAt, int sizeCode)
        {
            int[] kib = { 256, 512, 1024, 0, 0, 0, 0, 0, 0, 0, 8, 16, 32, 48, 64, 128 };
            int end = Math.Min(kib[sizeCode] * 1024, rom.Length);
            ushort sum = 0;
            for (int i = 0; i < end; i++)
                if (i != headerAt + 10 && i != headerAt + 11) sum += rom[i];
            return sum;
        }

        private static byte Bcd(int n) => (byte)(n / 10 << 4 | n % 10);
    }
}
