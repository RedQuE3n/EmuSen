using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Shaders
{
    // The platform library's catalog of built-in filters and console screens, read once; the library alone makes their passes and presets - see EmuSen_Serenity.md §3.13.
    internal static class FilterCatalog
    {
        private sealed record Read(IReadOnlyList<ScreenFilter> Filters, IReadOnlyList<ConsoleScreens.Screen> Screens, IReadOnlyList<ConsoleScreens.Owed> Owed, IReadOnlyList<string> Televisions, string Reduce);

        private static readonly Lazy<Read> Catalog = new(() =>
        {
            JsonElement c = FilterLibrary.Catalog;
            return new Read(
                c.GetProperty("filters").EnumerateArray().Select(FilterLibrary.Filter).ToArray(),
                c.GetProperty("screens").EnumerateArray().Select(s => new ConsoleScreens.Screen(s.GetProperty("console").GetString()!, s.GetProperty("kind").GetString() == "Lcd" ? ConsoleScreens.Kind.Lcd : ConsoleScreens.Kind.Television, s.GetProperty("profile").GetString()!)
                    { HostsPicture = s.GetProperty("hosts_picture").GetBoolean(), Gap = s.GetProperty("gap").GetString() }).ToArray(),
                c.GetProperty("owed").EnumerateArray().Select(o => new ConsoleScreens.Owed(o.GetProperty("console").GetString()!, o.GetProperty("note").GetString()!)).ToArray(),
                c.GetProperty("televisions").EnumerateArray().Select(t => t.GetString()!).ToArray(),
                c.GetProperty("reduce").GetString()!);
        });

        // The catalog's filters a front end offers, in its order: every filter but the simple effects.
        public static IEnumerable<ScreenFilter> Offered => Catalog.Value.Filters.Where(f => !f.Key!.StartsWith("effect.", StringComparison.Ordinal));

        public static ScreenFilter Keyed(string key) => Catalog.Value.Filters.First(f => f.Key == key);

        public static ScreenFilter Named(string name) => Keyed(name);

        public static IReadOnlyList<ConsoleScreens.Screen> Screens => Catalog.Value.Screens;

        public static IReadOnlyList<ConsoleScreens.Owed> Owed => Catalog.Value.Owed;

        public static IReadOnlyList<string> Televisions => Catalog.Value.Televisions;

        public static string ReduceGlsl => Catalog.Value.Reduce;
    }
}
