namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // How amidog's tests mark a column on their menu screen, by its colour key at the foot of the screen.
    public enum AmidogMark { None, NotApplicable, Ok, Info, Warning, Error }

    // A menu row of amidog's tests: its name and the line of 8-pixel text it sits on.
    public sealed record AmidogRow(string Name, int Line);

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

        public static AmidogMark Classify((byte R, byte G, byte B) c) => c switch
        {
            ( > 200, < 64, < 64) => AmidogMark.Error,
            ( > 200, > 200, < 64) => AmidogMark.Warning,
            ( < 64, > 200, > 200) => AmidogMark.Info,
            ( < 64, > 200, < 64) => AmidogMark.Ok,
            ( > 100 and < 160, > 40 and < 100, < 32) => AmidogMark.NotApplicable,
            _ => AmidogMark.None,
        };

        // The mark of the 8x8 cell at (cell, line): the colour most of its lit pixels have.
        public static AmidogMark Cell(RomPicture p, int cell, int line)
        {
            var counts = new Dictionary<AmidogMark, int>();
            for (int y = line * 8; y < line * 8 + 8; y++)
                for (int x = cell * 8; x < cell * 8 + 8; x++)
                {
                    var m = Classify(p.At(x, y));
                    if (m != AmidogMark.None) counts[m] = counts.GetValueOrDefault(m) + 1;
                }
            return counts.Count == 0 ? AmidogMark.None : counts.MaxBy(kv => kv.Value).Key;
        }

        public static IReadOnlyDictionary<string, AmidogMark[]> Verdicts(RomPicture p, IEnumerable<AmidogRow> rows)
        {
            if ((p.Width, p.Height) != (Width, Height)) throw new ArgumentException($"amidog's menu is {Width}x{Height}, the picture {p.Width}x{p.Height}");
            return rows.ToDictionary(r => r.Name, r => Enumerable.Range(FirstCell, 4).Select(c => Cell(p, c, r.Line)).ToArray());
        }

        // The line whose name is drawn in the highlight's yellow, where the menu's cursor stands.
        public static int Cursor(RomPicture p, int lines) =>
            Enumerable.Range(0, lines).FirstOrDefault(l => Enumerable.Range(0, 7).Any(c => Cell(p, c, l) == AmidogMark.Warning), -1);
    }
}
