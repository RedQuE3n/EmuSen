using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;

namespace EmuSen.Mistress.BigPicture
{
    // Moonrise's facts for Mistress: a planet per system with an engine, drawn by the systems data's look, and the words - see EmuSen_Interface.md §22.
    public static class StartupSplash
    {
        public const string Title = "EMUSEN";

        // The bells' name for the interface's sound stream, which a test's sink records.
        public const string BellsKey = NavigationSounds.Prefix + "moonrise";

        // Whether a new window shows the splash; the test assembly turns it off once, and a test about the splash turns its own window's on.
        public static bool ShownByDefault { get; set; } = true;

        // A console's hue and body are the systems data's look, which the runtime gives every system it lists; one it does not list is the plain planet.
        private static SystemLook? LookOf(string? system) => system is null ? null : RuntimeCatalog.Systems.FirstOrDefault(s => string.Equals(s.Id, system, StringComparison.OrdinalIgnoreCase))?.Look;

        public static double HueOf(string id) => LookOf(id)?.Hue ?? 0;

        public static WorldBody BodyOf(string? system) => LookOf(system) is { } look ? Body(look) : WorldBody.Plain;

        // The shape a maker's consoles are drawn as, its first listed console's; the planet for a maker with none.
        public static WorldShape ShapeOfMaker(string? maker) =>
            RuntimeCatalog.Systems.FirstOrDefault(s => s.Look is not null && string.Equals(s.Manufacturer, maker, StringComparison.OrdinalIgnoreCase))?.Look is { } look ? Shape(look.Body) : WorldShape.Planet;

        private static WorldShape Shape(string body) => body switch { "shards" => WorldShape.Shards, "gem" => WorldShape.Gem, _ => WorldShape.Planet };

        // Made once for each look the catalog read, so every screen draws a console as the same body.
        private static readonly ConditionalWeakTable<SystemLook, WorldBody> Drawn = new();

        private static WorldBody Body(SystemLook look) => Drawn.GetValue(look, l => new(Shape(l.Body))
        {
            Palette = l.Palette.Count == 3 ? l.Palette.Select(Color.Parse).ToArray() : null,
            Craters = l.Craters,
            Bands = l.Bands,
            Cap = l.Cap,
            Moons = l.Moons,
        });

        // One planet for each system the table lists with an engine, in the table's order, each the body Choose a console draws for it.
        public static IReadOnlyList<SplashBody> Bodies(IEnumerable<SystemFacts> systems) =>
            systems.Where(s => s.Engines.Count > 0).Select(s => new SplashBody(s.DisplayName is { Length: > 0 } shown ? shown : s.Console, s.Look?.Hue ?? 0) { Body = s.Look is { } look ? Body(look) : WorldBody.Plain }).ToList();

        public static string Caption(int consoles) => consoles == 1 ? "1 CONSOLE" : $"{consoles} CONSOLES";
    }
}
