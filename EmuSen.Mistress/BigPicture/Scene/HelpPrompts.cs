using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // Mistress's own help entries per view, in its words and glyphs, since ES-DE's are its own resources (§3.6, §4.9, Q9).
    public static class HelpPrompts
    {
        private sealed record Prompt(string Entry, string Glyph, string Label, string Button);

        private static readonly Prompt[] System =
        [
            new("left/right", "<>", "System", "dpad_leftright"),
            new("a", "A", "Select", "button_a_XBOX"),
            new("start", "+", "Menu", "button_start_XBOX"),
        ];

        private static readonly Prompt[] Gamelist =
        [
            new("up/down", "^v", "Choose", "dpad_updown"),
            new("left/right", "<>", "System", "dpad_leftright"),
            new("l", "L", "Page", "button_l"),
            new("r", "R", "Page", "button_r"),
            new("lt", "LT", "First", "button_lt"),
            new("rt", "RT", "Last", "button_rt"),
            new("a", "A", "Launch", "button_a_XBOX"),
            new("b", "B", "Back", "button_b_XBOX"),
            new("x", "X", "Search", "button_x_XBOX"),
            new("back", "-", "Favorite", "button_back_XBOX"),
            new("start", "+", "Menu", "button_start_XBOX"),
        ];

        // The entries a theme lists, in its order, that Mistress has an action for in this view; "all" is every one.
        public static IReadOnlyList<HintEntry> For(string view, IReadOnlyList<string> entries, IReadOnlyDictionary<string, ThemePath> icons)
        {
            Prompt[] known = view == "system" ? System : Gamelist;
            IEnumerable<Prompt> chosen = entries.Contains("all") ? known : entries.Select(n => known.FirstOrDefault(p => p.Entry == n)).OfType<Prompt>();
            return chosen.Select(p => new HintEntry(p.Label, icons.GetValueOrDefault(p.Button) is { Exists: true } icon ? icon.Absolute : null, p.Glyph)).ToList();
        }
    }
}
