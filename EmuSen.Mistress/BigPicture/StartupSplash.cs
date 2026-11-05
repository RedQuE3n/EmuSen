using System;
using System.Collections.Generic;
using System.Linq;
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

        // Each system's planet hue in degrees, the approved draft's; the one owner of a console's splash colour.
        public static readonly IReadOnlyDictionary<string, double> Hues = new Dictionary<string, double>(StringComparer.OrdinalIgnoreCase)
        {
            ["nes"] = 8, ["snes"] = 280, ["n64"] = 140, ["gb"] = 95, ["gbc"] = 320, ["sg1000"] = 205, ["sms"] = 220,
            ["md"] = 0, ["mcd"] = 190, ["32x"] = 30, ["psx"] = 230, ["ps2"] = 250, ["dreamcast"] = 25, ["saturn"] = 165,
        };

        // A system the table has no hue for gets one from its id, the same on every start.
        public static double HueOf(string id) =>
            Hues.TryGetValue(id, out double hue) ? hue : id.Aggregate(17u, (h, c) => unchecked(h * 31 + char.ToLowerInvariant(c))) % 360;

        // One planet for each system the table lists with an engine, in the table's order.
        public static IReadOnlyList<SplashBody> Bodies(IEnumerable<SystemFacts> systems) =>
            systems.Where(s => s.Engines.Count > 0).Select(s => new SplashBody(s.DisplayName is { Length: > 0 } shown ? shown : s.Console, HueOf(s.Id))).ToList();

        public static string Caption(int consoles) => consoles == 1 ? "1 CONSOLE" : $"{consoles} CONSOLES";
    }
}
