using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture.Scene
{
    // What changes the help entries: a collection being edited, and the random entry button's setting (§22).
    public sealed record HelpContext(bool Editing = false, bool RandomGames = false, bool RandomSystems = false, bool Folder = false, string QuickSelect = "leftright");

    // Mistress's own help entries per view, in its words, drawn with LunaP's button set for the pad's family (§3.6, §4.9, §15).
    public static class HelpPrompts
    {
        // An ES-DE help entry, the action it names in this view, its button, and the customButtonIcon key a theme would give it, with {0} for the family's suffix.
        private sealed record Prompt(string Entry, string Label, PadGlyphButton Button, string IconKey);

        private static readonly Prompt[] System =
        [
            new("left/right", "System", PadGlyphButton.DPadLeftRight, "dpad_leftright"),
            new("a", "Select", PadGlyphButton.South, "button_a_{0}"),
            new("start", "Menu", PadGlyphButton.Start, "button_start_{1}"),
        ];

        private static readonly Prompt[] Gamelist =
        [
            new("up/down", "Choose", PadGlyphButton.DPadUpDown, "dpad_updown"),
            new("left/right", "System", PadGlyphButton.DPadLeftRight, "dpad_leftright"),
            new("l", "Jump", PadGlyphButton.LeftShoulder, "button_l"),
            new("r", "Jump", PadGlyphButton.RightShoulder, "button_r"),
            new("lt", "First", PadGlyphButton.LeftTrigger, "button_lt"),
            new("rt", "Last", PadGlyphButton.RightTrigger, "button_rt"),
            new("a", "Launch", PadGlyphButton.South, "button_a_{0}"),
            new("b", "Back", PadGlyphButton.East, "button_b_{0}"),
            new("y", "Favorite", PadGlyphButton.North, "button_y_{0}"),
            new("back", "Options", PadGlyphButton.Select, "button_back_{1}"),
            new("start", "Menu", PadGlyphButton.Start, "button_start_{1}"),
        ];

        // ES-DE's suffixes for a family's face buttons and middle buttons; an unknown pad takes a theme's Xbox icons, ES-DE's own default controller type (§15).
        private static (string? Face, string? Middle) Suffixes(PadFamily family) => family switch
        {
            PadFamily.PlayStation => ("PS", "PS4"),
            PadFamily.Nintendo => ("switch", "switch"),
            _ => ("XBOX", "XBOX"),
        };

        // With the swap on, a function's prompt names the face button it moved to, and that button's icon (settings reference §4.61).
        private static Prompt Swapped(Prompt p) => p.Button switch
        {
            PadGlyphButton.South => p with { Button = PadGlyphButton.East, IconKey = p.IconKey.Replace("button_a_", "button_b_") },
            PadGlyphButton.East => p with { Button = PadGlyphButton.South, IconKey = p.IconKey.Replace("button_b_", "button_a_") },
            _ => p,
        };

        private static string? IconKey(Prompt p, PadFamily family)
        {
            (string? face, string? middle) = Suffixes(family);
            if (p.IconKey.Contains("{0}")) return face is null ? null : p.IconKey.Replace("{0}", face);
            if (p.IconKey.Contains("{1}")) return middle is null ? null : p.IconKey.Replace("{1}", middle);
            return p.IconKey;
        }

        // The pair quick system select takes reads System; left and right name nothing when they do not change it (§29).
        private static Prompt[] QuickSelected(Prompt[] known, string quick) => known
            .Where(p => p.Entry != "left/right" || quick == "leftright")
            .Select(p => (p.Entry, quick) is ("l" or "r", "shoulders") or ("lt" or "rt", "triggers") ? p with { Label = "System" } : p).ToArray();

        private static readonly Prompt Random = new("thumbstickclick", "Random", PadGlyphButton.ThumbstickClick, "thumbstick_click");

        // The entries a theme lists, in its order, that Mistress has an action for in this view; "all" is every one.
        public static IReadOnlyList<HintEntry> For(string view, IReadOnlyList<string> entries, IReadOnlyDictionary<string, ThemePath> icons, PadFamily family = PadFamily.Generic, bool swapped = false, HelpContext? context = null)
        {
            HelpContext c = context ?? new HelpContext();
            Prompt[] known = view == "system" ? System : Gamelist;
            if (view == "gamelist" && c.Editing) known = known.Select(p => p.Entry == "y" ? p with { Label = "Collection" } : p).ToArray();
            if (view == "gamelist" && c.Folder) known = known.Select(p => p.Entry == "a" ? p with { Label = "Select" } : p).ToArray();
            if (view == "gamelist") known = QuickSelected(known, c.QuickSelect);
            if (view == "system" ? c.RandomSystems : c.RandomGames) known = [.. known.Take(known.Length - 1), Random, known[^1]];
            IEnumerable<Prompt> chosen = entries.Contains("all") ? known : entries.Select(n => known.FirstOrDefault(p => p.Entry == n)).OfType<Prompt>();
            if (swapped) chosen = chosen.Select(Swapped);
            return chosen.Select(p => new HintEntry(p.Label, IconKey(p, family) is { } key && icons.GetValueOrDefault(key) is { Exists: true } icon ? icon.Absolute : null)
            {
                Button = p.Button,
            }).ToList();
        }
    }
}
