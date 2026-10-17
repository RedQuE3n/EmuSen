using System;
using System.Collections.Generic;

namespace EmuSen.Serenity.Shaders
{
    // The television standard a picture is drawn to, and how a filter that models it is told the game's - see EmuSen_CRT.md §26.
    public static class TelevisionStandards
    {
        // Machine info's words for the standard a machine runs to - see EmuSen_CoreAPI.md §6.4.
        public const string Ntsc = "ntsc", Pal = "pal";

        // The built-in filters' parameter: 0 NTSC, 1 PAL, and FollowsGame, which the frontend replaces with the game's.
        public const string Parameter = "standard";
        public const float FollowsGame = 2;

        // The switches RetroArch presets in the pack declare for PAL, each 0 for off and 1 for on - see EmuSen_CRT.md §26.3.
        public static IReadOnlyList<string> PresetSwitches { get; } = new[] { "PAL", "uModePAL", "u_pal" };

        public static bool IsPal(float value) => value > 0.5f && value < 1.5f;

        // The values a filter is drawn with: the standard given for every name a filter may know it by, over the player's own only where chosen says so.
        public static Dictionary<string, float> Apply(IReadOnlyDictionary<string, float> values, string? standard, bool chosen)
        {
            var drawn = new Dictionary<string, float>(values, StringComparer.Ordinal);
            float pal = string.Equals(standard, Pal, StringComparison.OrdinalIgnoreCase) ? 1 : 0;
            if (chosen || !drawn.TryGetValue(Parameter, out float own) || own >= 1.5f) drawn[Parameter] = pal;
            foreach (string id in PresetSwitches) if (chosen || !drawn.ContainsKey(id)) drawn[id] = pal;
            return drawn;
        }
    }
}
