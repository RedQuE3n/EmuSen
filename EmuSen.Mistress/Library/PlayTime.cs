using System;
using System.Linq;

namespace EmuSen.Mistress.Library
{
    // ES-DE's "Max play time tracking": what one launch adds to a game's play time - see EmuSen_Settings_Reference.md §4.66.
    public static class PlayTime
    {
        public const int Disabled = 0, NoLimit = 24;

        public static readonly (int Hours, string Text)[] Choices =
            [(Disabled, "Disabled"), .. Enumerable.Range(1, 23).Select(h => (h, h == 1 ? "1 hour" : $"{h} hours")), (NoLimit, "No limit")];

        // Null records nothing: tracking is off, or the launch ran longer than the limit, as a device left asleep would.
        public static TimeSpan? Tracked(TimeSpan measured, int maxHours) => maxHours switch
        {
            <= Disabled => null,
            >= NoLimit => measured,
            _ => measured > TimeSpan.FromHours(maxHours) ? null : measured,
        };
    }
}
