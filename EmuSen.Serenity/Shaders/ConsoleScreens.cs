using System.Collections.Generic;

namespace EmuSen.Serenity.Shaders
{
    // Each console's screen and the accurate filter's profile made for it, one row for each console of the systems table, as the platform library's catalog gives them - see EmuSen_CRT.md §31.
    public static class ConsoleScreens
    {
        public enum Kind { Television, Lcd }

        // A console's screen: for a television, Profile is its row of the CRT's console timing; for an LCD, the name of the filter that models the panel.
        public sealed record Screen(string Console, Kind Kind, string Profile)
        {
            // An attachment with no video output of its own, whose picture is its host's signal and so its host's profile.
            public bool HostsPicture { get; init; }

            // A dated note of what this console's own profile lacks, so that a gap is decided and visible, never silent.
            public string? Gap { get; init; }
        }

        public static IReadOnlyList<Screen> All => FilterCatalog.Screens;

        // A console of the systems table still in development, offered to no player, whose screen row is owed, with a dated note of why - see EmuSen_CRT.md §31.8.
        public sealed record Owed(string Console, string Note);

        public static IReadOnlyList<Owed> OwedRows => FilterCatalog.Owed;

        // The consoles whose television is drawn on a row of their own, which every filter of a tube is offered for; an attachment is offered its host's.
        public static IReadOnlyList<string> Televisions => FilterCatalog.Televisions;
    }
}
