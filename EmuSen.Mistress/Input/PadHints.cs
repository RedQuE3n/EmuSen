using System.Text.RegularExpressions;

namespace EmuSen.Mistress.Input
{
    // The interface's text hints name face buttons by an Xbox pad's letters; with the swap on, A and B trade places and so do X and Y - see EmuSen_Settings_Reference.md §4.61.
    public static partial class PadHints
    {
        // Set by the main window from its settings; the keyboard's hint reads it too.
        public static bool Swapped { get; set; }

        public static string Face(string hint) => Swapped ? Letter().Replace(hint, m => m.Value switch { "A" => "B", "B" => "A", "X" => "Y", _ => "X" }) : hint;

        // A lone letter at the start or after a wide gap, followed by the two spaces before its words.
        [GeneratedRegex(@"(?<=^|\s{2})[ABXY](?=\s{2})")]
        private static partial Regex Letter();
    }
}
