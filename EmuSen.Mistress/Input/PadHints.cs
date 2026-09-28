using System.Text.RegularExpressions;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Input
{
    // The interface's text hints name face buttons by an Xbox pad's letters; with the swap on, A and B trade places and so do X and Y - see EmuSen_Settings_Reference.md §4.61 and §4.79.5.
    public static partial class PadHints
    {
        // Set by the main window from its settings; the keyboard's hint reads it too.
        public static bool Swapped { get; set; }

        public static string Face(string hint) => Swapped ? LoneLetter().Replace(hint, m => Traded(m.Value)) : hint;

        // An Xbox letter, A, B, X or Y, for the button that now does its job.
        public static string Letter(string letter) => Swapped ? Traded(letter) : letter;

        private static string Traded(string letter) => letter switch { "A" => "B", "B" => "A", "X" => "Y", "Y" => "X", _ => letter };

        // The button a function unswapped on this face button is on now: South and East trade, and so do North and West.
        public static PadGlyphButton Glyph(PadGlyphButton unswapped) => Swapped ? Of(unswapped, true) : unswapped;

        public static PadGlyphButton Of(PadGlyphButton unswapped, bool swapped) => !swapped ? unswapped : unswapped switch
        {
            PadGlyphButton.South => PadGlyphButton.East,
            PadGlyphButton.East => PadGlyphButton.South,
            PadGlyphButton.North => PadGlyphButton.West,
            PadGlyphButton.West => PadGlyphButton.North,
            _ => unswapped,
        };

        // A lone letter at the start or after a wide gap, followed by the two spaces before its words.
        [GeneratedRegex(@"(?<=^|\s{2})[ABXY](?=\s{2})")]
        private static partial Regex LoneLetter();
    }
}
