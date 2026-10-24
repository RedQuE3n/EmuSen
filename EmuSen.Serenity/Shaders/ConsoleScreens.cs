using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Serenity.Shaders
{
    // Each console's screen and the accurate filter's profile made for it, one row for each console of the systems table - see EmuSen_CRT.md §31.
    public static class ConsoleScreens
    {
        public enum Kind { Television, Lcd }

        // A console's screen: for a television, Profile is its row of CrtFilter.ConsoleTiming; for an LCD, the name of the filter that models the panel.
        public sealed record Screen(string Console, Kind Kind, string Profile)
        {
            // An attachment with no video output of its own, whose picture is its host's signal and so its host's profile.
            public bool HostsPicture { get; init; }

            // A dated note of what this console's own profile lacks, so that a gap is decided and visible, never silent.
            public string? Gap { get; init; }
        }

        public static IReadOnlyList<Screen> All { get; } = new Screen[]
        {
            new("NES", Kind.Television, "NES"),
            new("SNES", Kind.Television, "SNES"),
            new("Genesis", Kind.Television, "Genesis"),
            new("Sega CD", Kind.Television, "Genesis") { HostsPicture = true },
            new("32X", Kind.Television, "Genesis") { Gap = "2026-10-20: drawn on the Genesis's row; the 32X puts the mixed picture out through an output stage of its own, whose encoder no document read names (EmuSen_CRT.md §31.4)" },
            new("N64", Kind.Television, "N64"),
            new("PlayStation", Kind.Television, "PlayStation"),
            new("Master System", Kind.Television, "Master System"),
            new("SG-1000", Kind.Television, "SG-1000"),
            new("GB", Kind.Lcd, "Game Boy LCD") { Gap = "2026-10-20: a fit to measurements of the panel, not a model of it; to be recreated more accurately (EmuSen_CRT.md §31.7)" },
            new("GBC", Kind.Lcd, "Game Boy Color LCD") { Gap = "2026-10-20: a fit to measurements of the panel, not a model of it; to be recreated more accurately (EmuSen_CRT.md §31.7)" },
        };

        // A console of the systems table still in development, offered to no player, whose screen row is owed before it reaches one, with a dated note of why - see EmuSen_CRT.md §31.8.
        public sealed record Owed(string Console, string Note);

        public static IReadOnlyList<Owed> OwedRows { get; } = new Owed[]
        {
            new("PlayStation 2", "2026-10-24: a television; its profile needs the PCRTC's clocks, which wait on D-0 and on stage S20, and it is made when the console reaches players at S29 (Sapphire_Plan.md §4.6)"),
        };

        // The consoles whose television is drawn on a row of their own, which every filter of a tube is offered for; an attachment is offered its host's.
        public static IReadOnlyList<string> Televisions { get; } = All.Where(s => s.Kind == Kind.Television && s.Profile == s.Console).Select(s => s.Console).ToArray();
    }
}
