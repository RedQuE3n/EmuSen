using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic Master System or SG-1000 cartridge: Sega's header with its checksum and region, a boot stub and a pad reader, and the mappers' banking programs - never real game data - see Endou_Native.md §4, §9.8.
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

        // A step of a banking program: a byte written to an address, or a byte read from one and kept in RAM.
        public readonly record struct Step(ushort Address, byte Value, bool Write)
        {
            public static Step Put(ushort address, byte value) => new(address, value, true);
            public static Step Keep(ushort address) => new(address, 0, false);
        }

        // DI; LD SP,$DFF0; each step as LD A,n; LD (nn),A or LD A,(nn); LD ($C000+k),A; then JR $ - the k-th kept byte at $C000+k.
        public static byte[] Program(IEnumerable<Step> steps)
        {
            var code = new List<byte> { 0xF3, 0x31, 0xF0, 0xDF };
            int kept = 0;
            foreach (var s in steps)
            {
                if (s.Write) code.AddRange(new byte[] { 0x3E, s.Value, 0x32, (byte)s.Address, (byte)(s.Address >> 8) });
                else
                {
                    code.AddRange(new byte[] { 0x3A, (byte)s.Address, (byte)(s.Address >> 8), 0x32, (byte)kept, 0xC0 });
                    kept++;
                }
            }
            code.AddRange(new byte[] { 0x18, 0xFE });
            return code.ToArray();
        }

        // A Master System image of `banks` 16 KiB banks, each holding its own number at $3F00, the program at $0000 and Sega's export header.
        public static byte[] Banked(int banks, byte[] program)
        {
            var rom = new byte[banks * 0x4000];
            for (int b = 0; b < banks; b++) rom[b * 0x4000 + 0x3F00] = (byte)b;
            program.CopyTo(rom, 0);
            Encoding.ASCII.GetBytes("TMR SEGA").CopyTo(rom, HeaderAt);
            rom[HeaderAt + 15] = 0x4C;
            return rom;
        }

        // Sega's mapper: each slot banked, the first kilobyte kept, a bank past the ROM wrapped, RAM's copy of $FFFF, the cartridge's RAM in slot 2 in both banks - see Endou_Native.md §9.3.
        public static byte[] SegaMapper(int banks = 32) => Banked(banks, Program(new[]
        {
            Step.Put(0xFFFF, 5), Step.Keep(0xBF00), Step.Put(0xFFFE, 6), Step.Keep(0x7F00), Step.Put(0xFFFD, 7), Step.Keep(0x3F00), Step.Keep(0x0000),
            Step.Put(0xFFFF, (byte)(banks + 1)), Step.Keep(0xBF00), Step.Keep(0xFFFF),
            Step.Put(0xFFFC, 0x08), Step.Put(0x8000, 0x77), Step.Keep(0x8000), Step.Put(0xFFFC, 0x0C), Step.Keep(0x8000), Step.Put(0xFFFC, 0x00), Step.Keep(0xBF00),
        }));

        // Codemasters' mapper: slots 2 and 1 banked by their own addresses, the first kilobyte not kept, the RAM over $A000-$BFFF - see Endou_Native.md §9.3.
        public static byte[] CodemastersMapper(int banks = 8) => WithCodemastersHeader(Banked(banks, Program(new[]
        {
            Step.Put(0x8000, 5), Step.Keep(0xBF00), Step.Put(0x4000, 3), Step.Keep(0x7F00), Step.Put(0xFFFF, 1), Step.Keep(0xBF00),
            Step.Put(0x4000, 0x84), Step.Put(0xA000, 0x5A), Step.Keep(0xA000), Step.Keep(0x8000), Step.Keep(0x7F00),
        })));

        // A board without a mapper: 48 KiB read straight, and Sega's registers meaning nothing - see Endou_Native.md §9.3.
        public static byte[] Plain(int banks = 3) => Banked(banks, Program(new[] { Step.Put(0xFFFF, 0), Step.Keep(0xBF00), Step.Keep(0x7F00), Step.Keep(0x3F00) }));

        // Codemasters' header at $7FE0 over every bank: the little-endian words summed with Sega's header left out, and the word that cancels it (SMS Power!'s "Codemasters Header").
        public static byte[] WithCodemastersHeader(byte[] rom)
        {
            Array.Clear(rom, 0x7FE0, 16);
            rom[0x7FE0] = (byte)(rom.Length / 0x4000);
            new byte[] { 0x31, 0x08, 0x93, 0x10, 0x59 }.CopyTo(rom, 0x7FE1);
            ushort sum = 0;
            for (int i = 0; i < rom.Length; i += 2)
                if (i < 0x7FF0 || i >= 0x8000) sum += (ushort)(rom[i] | rom[i + 1] << 8);
            ushort neg = (ushort)-sum;
            rom[0x7FE6] = (byte)sum;
            rom[0x7FE7] = (byte)(sum >> 8);
            rom[0x7FE8] = (byte)neg;
            rom[0x7FE9] = (byte)(neg >> 8);
            return rom;
        }

        // DI; LD SP,$DFF0; each write as LD A,n; OUT (p),A; then JR $.
        public static byte[] Ports(IEnumerable<(byte Port, byte Value)> writes)
        {
            var code = new List<byte> { 0xF3, 0x31, 0xF0, 0xDF };
            foreach (var (port, value) in writes) code.AddRange(new byte[] { 0x3E, value, 0xD3, port });
            code.AddRange(new byte[] { 0x18, 0xFE });
            return code.ToArray();
        }

        // Mode 4 with the left column masked: CRAM 0, blue, under every pixel, and the backdrop, CRAM 17, green, over the first eight columns - see Endou_Native.md §10.8.
        public static byte[] Picture() => Banked(2, Ports(new (byte, byte)[]
        {
            (0xBF, 0x26), (0xBF, 0x80), (0xBF, 0x40), (0xBF, 0x81), (0xBF, 0x01), (0xBF, 0x87),
            (0xBF, 0x00), (0xBF, 0xC0), (0xBE, 0x30), (0xBF, 0x11), (0xBF, 0xC0), (0xBE, 0x0C),
        }));

        // Tone 0 at half-period $0FE and full volume through port $7F, 440.4 Hz - see Endou_Native.md §12.
        public static byte[] Tone(bool sg1000 = false)
        {
            var program = Ports(new (byte, byte)[] { (0x7F, 0x9F), (0x7F, 0xBF), (0x7F, 0xDF), (0x7F, 0xFF), (0x7F, 0x8E), (0x7F, 0x0F), (0x7F, 0x90) });
            if (sg1000)
            {
                var rom = new byte[0x4000];
                program.CopyTo(rom, 0);
                return rom;
            }
            return Banked(2, program);
        }

        // An SG-1000 cartridge in Graphics I: name 0 everywhere, its rows four pixels of colour 6 then four of colour 12, no sprites - see Endou_Native.md §11.
        public static byte[] TmsPicture()
        {
            var writes = new List<(byte, byte)>();
            foreach (var (register, value) in new (int, byte)[] { (0, 0x00), (1, 0x80), (2, 0x0E), (3, 0x80), (4, 0x00), (5, 0x76), (6, 0x00), (7, 0xF4) })
                writes.AddRange(new (byte, byte)[] { (0xBF, value), (0xBF, (byte)(0x80 | register)) });
            writes.AddRange(new (byte, byte)[] { (0xBF, 0x00), (0xBF, 0x7B), (0xBE, 0xD0), (0xBF, 0x00), (0xBF, 0x60), (0xBE, 0x6C), (0xBF, 0x00), (0xBF, 0x40) });
            for (int row = 0; row < 8; row++) writes.Add((0xBE, 0xF0));
            writes.AddRange(new (byte, byte)[] { (0xBF, 0xC0), (0xBF, 0x81) });
            var rom = new byte[0x4000];
            Ports(writes).CopyTo(rom, 0);
            return rom;
        }

        private static byte Bcd(int n) => (byte)(n / 10 << 4 | n % 10);
    }
}
