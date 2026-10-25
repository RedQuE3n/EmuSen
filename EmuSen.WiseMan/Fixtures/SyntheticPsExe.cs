using System.Text;

namespace EmuSen.WiseMan.Fixtures
{
    // Builds a synthetic PS-EXE with the header psx-spx's "CDROM File Formats" gives - never real program data - see Diamond_Native.md §5.1.
    public static class SyntheticPsExe
    {
        public const int Header = 0x800;
        public const uint Origin = 0x8001_0000, Stack = 0x801F_FFF0;

        // The program, by default a jump to itself and its delay slot, loaded and entered at the origin; market is the marker's area, or null for none.
        public static byte[] Build(byte[]? code = null, string? market = "North America", uint pc = Origin, uint load = Origin)
        {
            code ??= Words(0x0800_0000u | ((pc >> 2) & 0x03FF_FFFF), 0);
            int size = (code.Length + Header - 1) / Header * Header;
            var exe = new byte[Header + size];
            Encoding.ASCII.GetBytes("PS-X EXE").CopyTo(exe, 0);
            Put(exe, 0x10, pc);
            Put(exe, 0x18, load);
            Put(exe, 0x1C, (uint)size);
            Put(exe, 0x30, Stack);
            if (market is not null) Encoding.ASCII.GetBytes($"Sony Computer Entertainment Inc. for {market} area").CopyTo(exe, 0x4C);
            code.CopyTo(exe, Header);
            return exe;
        }

        // A program that prints through the DEV8 DUART, 1F802023h, a byte at a time, as the open kernel's TTY will, and stops.
        public static byte[] Printing(string text)
        {
            var words = new List<uint> { 0x3C08_1F80, 0x3508_2023 };
            foreach (char c in text) words.AddRange(new[] { 0x2409_0000u | c, 0xA109_0000u });
            uint loop = Origin + 4 * (uint)words.Count;
            words.AddRange(new[] { 0x0800_0000u | ((loop >> 2) & 0x03FF_FFFF), 0u });
            return Build(Words(words.ToArray()));
        }

        public static byte[] Words(params uint[] words) => words.SelectMany(BitConverter.GetBytes).ToArray();

        private static void Put(byte[] b, int at, uint v) => BitConverter.TryWriteBytes(b.AsSpan(at, 4), v);
    }
}
