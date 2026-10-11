namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // How amidog's tests mark a column on their menu screen, by its colour key at the foot of the screen.
    public enum AmidogMark { None, NotApplicable, Ok, Info, Warning, Error }

    // A menu row of amidog's tests: its name, the line of text it sits on and the column its letters start at, in cells.
    public sealed record AmidogRow(string Name, int Line, int Column = AmidogScreen.FirstCell);

    // amidog's verdicts read from the menu screen: each row's X, F, V and T letters by their colour - see Diamond_Pad.md §5.
    public static class AmidogScreen
    {
        public const int Width = 512, Height = 224;
        // The four letters' first column, in 8-pixel cells.
        public const int FirstCell = 8;
        public static readonly string[] Columns = { "exception", "flag", "value", "timing" };

        public static readonly AmidogRow[] Cpx =
        {
            new("EXCP", 2), new("INTR", 3), new("SLOT", 4), new("SLOT BR", 7), new("BREAK", 8), new("ILLEGAL", 9),
        };

        // psxtest_gte's results in its 6-pixel font: OFFICIAL TIMING's 22 rows and TOTAL, and UNOFFICIAL TIMING's 42 in two columns.
        public const int GteCell = 6;
        public static readonly AmidogRow[] GteOfficialTiming = new[] { "SQR", "NCLIP", "AVSZ3", "AVSZ4", "OP", "GPF", "GPL", "RTPS", "RTPT", "NCS", "NCT", "NCCS", "NCCT", "CC", "DPCS", "DPCT", "INTPL", "CDP", "NCDS", "NCDT", "DPCL", "MVMVA" }
            .Select((n, i) => new AmidogRow(n, 1 + i, 9)).Append(new AmidogRow("TOTAL", 25, 9)).ToArray();
        public static readonly int[] GteUnlisted = Enumerable.Range(0, 64).Where(n => !new[] { 0x01, 0x06, 0x0C, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x1B, 0x1C, 0x1E, 0x20, 0x28, 0x29, 0x2A, 0x2D, 0x2E, 0x30, 0x3D, 0x3E, 0x3F }.Contains(n)).ToArray();
        public static readonly AmidogRow[] GteUnofficialTiming = GteUnlisted
            .Select((n, i) => i < 32 ? new AmidogRow($"{n:X2}h", 1 + i, 9) : new AmidogRow($"{n:X2}h", i - 32, 23)).Append(new AmidogRow("TOTAL", 12, 23)).ToArray();

        public static AmidogMark Classify((byte R, byte G, byte B) c) => c switch
        {
            ( > 200, < 64, < 64) => AmidogMark.Error,
            ( > 200, > 200, < 64) => AmidogMark.Warning,
            ( < 64, > 200, > 200) => AmidogMark.Info,
            ( < 64, > 200, < 64) => AmidogMark.Ok,
            ( > 100 and < 160, > 40 and < 100, < 32) => AmidogMark.NotApplicable,
            _ => AmidogMark.None,
        };

        // The mark of the cell at (cell, line), size pixels square: the colour most of its lit pixels have.
        public static AmidogMark Cell(RomPicture p, int cell, int line, int size = 8)
        {
            var counts = new Dictionary<AmidogMark, int>();
            for (int y = line * size; y < line * size + size; y++)
                for (int x = cell * size; x < cell * size + size; x++)
                {
                    var m = Classify(p.At(x, y));
                    if (m != AmidogMark.None) counts[m] = counts.GetValueOrDefault(m) + 1;
                }
            return counts.Count == 0 ? AmidogMark.None : counts.MaxBy(kv => kv.Value).Key;
        }

        public static IReadOnlyDictionary<string, AmidogMark[]> Verdicts(RomPicture p, IEnumerable<AmidogRow> rows, int size = 8)
        {
            if ((p.Width, p.Height) != (Width, Height)) throw new ArgumentException($"amidog's menu is {Width}x{Height}, the picture {p.Width}x{p.Height}");
            return rows.ToDictionary(r => r.Name, r => Enumerable.Range(r.Column, 4).Select(c => Cell(p, c, r.Line, size)).ToArray());
        }

        // The line whose name is drawn in the highlight's yellow, where the menu's cursor stands.
        public static int Cursor(RomPicture p, int lines) =>
            Enumerable.Range(0, lines).FirstOrDefault(l => Enumerable.Range(0, 7).Any(c => Cell(p, c, l) == AmidogMark.Warning), -1);
    }
}
