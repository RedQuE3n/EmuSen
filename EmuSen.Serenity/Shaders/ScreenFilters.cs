using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Serenity.Shaders
{
    // One entry a frontend can offer: a name it stores, and either a simple effect or a multi-pass filter - see EmuSen_Serenity.md §3.1.
    public sealed record ScreenFilterChoice(string Name, ShaderEffect Effect, ScreenFilter? Filter)
    {
        public bool Suits(string console) => Filter?.Consoles is not { } consoles || consoles.Contains(console, StringComparer.OrdinalIgnoreCase);
    }

    // The screen filters a frontend can offer, by the name it stores; the accurate ones list only the consoles they suit - see EmuSen_Serenity.md §3.1.
    public static class ScreenFilters
    {
        public const string None = "None";

        public static IReadOnlyList<ScreenFilterChoice> All { get; } = new[]
        {
            new ScreenFilterChoice(None, ShaderEffect.None, null),
            new ScreenFilterChoice(CrtFilters.Lottes.Name, ShaderEffect.None, CrtFilters.Lottes),
            new ScreenFilterChoice(HandheldFilters.DmgLcd.Name, ShaderEffect.None, HandheldFilters.DmgLcd),
            new ScreenFilterChoice(HandheldFilters.PocketLcd.Name, ShaderEffect.None, HandheldFilters.PocketLcd),
            new ScreenFilterChoice(HandheldFilters.LightLcd.Name, ShaderEffect.None, HandheldFilters.LightLcd),
            new ScreenFilterChoice(HandheldFilters.GbcLcd.Name, ShaderEffect.None, HandheldFilters.GbcLcd),
            new ScreenFilterChoice(HandheldFilters.AgbLcd.Name, ShaderEffect.None, HandheldFilters.AgbLcd),
            new ScreenFilterChoice(HandheldFilters.Ags101Lcd.Name, ShaderEffect.None, HandheldFilters.Ags101Lcd),
            new ScreenFilterChoice("Scanlines", ShaderEffect.Scanlines, null),
            new ScreenFilterChoice("Simple CRT", ShaderEffect.Crt, null),
        };

        public static IReadOnlyList<string> Names { get; } = All.Select(f => f.Name).ToArray();

        // What a console's list shows: every filter that suits it, None first.
        public static IReadOnlyList<string> NamesFor(string console) => All.Where(f => f.Suits(console)).Select(f => f.Name).ToArray();

        // An unknown or missing name is no filter, so a setting from a newer build never breaks the picture.
        public static ScreenFilterChoice Find(string? name) =>
            All.FirstOrDefault(f => string.Equals(f.Name, name, StringComparison.Ordinal)) ?? All[0];

        public static ShaderEffect ByName(string? name) => Find(name).Effect;
    }
}
