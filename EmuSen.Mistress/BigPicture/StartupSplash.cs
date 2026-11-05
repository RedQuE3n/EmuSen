using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.BigPicture
{
    // Moonrise's facts for Mistress: a planet per system with an engine, each console's colour, and the words - see EmuSen_Interface.md §22.
    public static class StartupSplash
    {
        public const string Title = "EMUSEN";

        // The bells' name for the interface's sound stream, which a test's sink records.
        public const string BellsKey = NavigationSounds.Prefix + "moonrise";

        // Whether a new window shows the splash; the test assembly turns it off once, and a test about the splash turns its own window's on.
        public static bool ShownByDefault { get; set; } = true;

        // Each system's planet hue in degrees, the approved draft's and the Nintendo scouts' (2026-11-05); the one owner of a console's colour.
        public static readonly IReadOnlyDictionary<string, double> Hues = new Dictionary<string, double>(StringComparer.OrdinalIgnoreCase)
        {
            ["nes"] = 330, ["snes"] = 38, ["n64"] = 352, ["gb"] = 212, ["gbc"] = 190, ["sg1000"] = 205, ["sms"] = 220,
            ["md"] = 0, ["mcd"] = 190, ["32x"] = 30, ["psx"] = 230, ["ps2"] = 250, ["dreamcast"] = 25, ["saturn"] = 165,
        };

        // The Nintendo planets in the scouts' colours, each with its own feature - see EmuSen_Interface.md §24.6.
        public static readonly IReadOnlyDictionary<string, WorldBody> Planets = new Dictionary<string, WorldBody>(StringComparer.OrdinalIgnoreCase)
        {
            ["nes"] = Planet("#FFF8FB", "#F3CFE3", "#A9789F") with { Craters = 0.28 },
            ["snes"] = Planet("#FFF3CF", "#FFBE5C", "#B8652A") with { Bands = 0.5 },
            ["n64"] = Planet("#FFD3D8", "#EE4E66", "#7E1A38") with { Cap = 0.9, Moons = 2 },
            ["gb"] = Planet("#DCF1FF", "#4E9CEB", "#1F3A7A"),
            ["gbc"] = Planet("#E0FCFF", "#62D2E6", "#226A88"),
        };

        private static WorldBody Planet(string light, string middle, string rim) => new(WorldShape.Planet) { Palette = [Color.Parse(light), Color.Parse(middle), Color.Parse(rim)] };

        private static readonly WorldBody Shards = new(WorldShape.Shards), Gem = new(WorldShape.Gem);

        // What a console is drawn as on Choose a console: its own planet, else its maker's body (Sega shards, Sony a gem), else the plain planet.
        public static WorldBody BodyOf(string? system, string? maker) =>
            system is not null && Planets.TryGetValue(system, out WorldBody? planet) ? planet
            : string.Equals(maker, "Sega", StringComparison.OrdinalIgnoreCase) ? Shards
            : string.Equals(maker, "Sony", StringComparison.OrdinalIgnoreCase) ? Gem
            : WorldBody.Plain;

        // A system the table has no hue for gets one from its id, the same on every start.
        public static double HueOf(string id) =>
            Hues.TryGetValue(id, out double hue) ? hue : id.Aggregate(17u, (h, c) => unchecked(h * 31 + char.ToLowerInvariant(c))) % 360;

        // One planet for each system the table lists with an engine, in the table's order, each the body Choose a console draws for it.
        public static IReadOnlyList<SplashBody> Bodies(IEnumerable<SystemFacts> systems) =>
            systems.Where(s => s.Engines.Count > 0).Select(s => new SplashBody(s.DisplayName is { Length: > 0 } shown ? shown : s.Console, HueOf(s.Id)) { Body = BodyOf(s.Id, s.Manufacturer) }).ToList();

        public static string Caption(int consoles) => consoles == 1 ? "1 CONSOLE" : $"{consoles} CONSOLES";
    }
}
