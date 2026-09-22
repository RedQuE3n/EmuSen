using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Serenity.Shaders
{
    // The screen filters a frontend can offer, by the name it stores; the list grows as real CRT and LCD filters land - see EmuSen_Serenity.md §3.1.
    public static class ScreenFilters
    {
        public const string None = "None";

        public static IReadOnlyList<(string Name, ShaderEffect Effect)> All { get; } = new[]
        {
            (None, ShaderEffect.None),
            ("Scanlines", ShaderEffect.Scanlines),
            ("Simple CRT", ShaderEffect.Crt),
        };

        public static IReadOnlyList<string> Names { get; } = All.Select(f => f.Name).ToArray();

        // An unknown or missing name is no filter, so a setting from a newer build never breaks the picture.
        public static ShaderEffect ByName(string? name) =>
            All.FirstOrDefault(f => string.Equals(f.Name, name, StringComparison.Ordinal)) is { Name: not null } found ? found.Effect : ShaderEffect.None;
    }
}
