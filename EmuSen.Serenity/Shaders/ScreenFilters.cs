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

    // The screen filters a frontend can offer, by the name it stores: none, the platform library's in its catalog's order, then the simple effects - see EmuSen_Serenity.md §3.1.
    public static class ScreenFilters
    {
        public const string None = "None";

        private static readonly Lazy<IReadOnlyList<ScreenFilterChoice>> Choices = new(() =>
            new[] { new ScreenFilterChoice(None, ShaderEffect.None, null) }
                .Concat(FilterCatalog.Offered.Select(f => new ScreenFilterChoice(f.Name, ShaderEffect.None, f)))
                .Append(new ScreenFilterChoice("Scanlines", ShaderEffect.Scanlines, null))
                .Append(new ScreenFilterChoice("Simple CRT", ShaderEffect.Crt, null))
                .ToArray());

        public static IReadOnlyList<ScreenFilterChoice> All => Choices.Value;

        public static IReadOnlyList<string> Names => All.Select(f => f.Name).ToArray();

        // What a console's list shows: every filter that suits it, None first.
        public static IReadOnlyList<string> NamesFor(string console) => All.Where(f => f.Suits(console)).Select(f => f.Name).ToArray();

        // An unknown or missing name is no filter, so a setting from a newer build never breaks the picture.
        public static ScreenFilterChoice Find(string? name) =>
            All.FirstOrDefault(f => string.Equals(f.Name, name, StringComparison.Ordinal)) ?? All[0];

        public static ShaderEffect ByName(string? name) => Find(name).Effect;
    }
}
