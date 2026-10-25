using EmuSen.WiseMan.Fixtures.RomRunner;

namespace EmuSen.WiseMan.Fixtures
{
    // A synthetic EE program: an ELF32 header for a little-endian MIPS executable, one loadable segment and code of the test's own, never real program data - see Sapphire_Native.md §5 and §14.3.
    public static class SyntheticPs2Elf
    {
        public const uint Entry = 0x0010_0000;
        public const ushort Executable = 2, IopModule = 0xFF80, Mips = 8;
        public const int Header = 52, ProgramHeader = 32;

        // By default a branch to itself and its delay slot at the entry; the type, machine and class can be spoiled for the refusals, and bss bytes of zeroes follow the code in memory only.
        public static byte[] Build(uint[]? code = null, ushort type = Executable, ushort machine = Mips, byte elfClass = 1, uint origin = Entry, uint bss = 0)
        {
            code ??= new uint[] { 0x1000_FFFF, 0 };
            var image = new byte[Header + ProgramHeader + code.Length * 4];
            new byte[] { 0x7F, (byte)'E', (byte)'L', (byte)'F', elfClass, 1, 1 }.CopyTo(image, 0);
            void Put16(int at, ushort v) => BitConverter.TryWriteBytes(image.AsSpan(at, 2), v);
            void Put32(int at, uint v) => BitConverter.TryWriteBytes(image.AsSpan(at, 4), v);
            Put16(16, type);
            Put16(18, machine);
            Put32(20, 1);
            Put32(24, origin);
            Put32(28, Header);
            Put16(40, Header);
            Put16(42, ProgramHeader);
            Put16(44, 1);
            uint bytes = (uint)(code.Length * 4);
            foreach (var (at, v) in new (int, uint)[] { (0, 1), (4, Header + ProgramHeader), (8, origin), (12, origin), (16, bytes), (20, bytes + bss), (24, 5), (28, 16) }) Put32(Header + at, v);
            for (int i = 0; i < code.Length; i++) Put32(Header + ProgramHeader + i * 4, code[i]);
            return image;
        }

        // A bare program that leaves the words in Sapphire's RAM record and stops, the form every referee reads (Sapphire_Referees.md §4.1).
        public static byte[] Recording(params uint[] words) => Build(RamRecordForm.Sapphire.Program(words));
    }
}
