using System;
using System.Collections.Generic;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Library
{
    // The mark an engine carries before its name, a property of the engine like its name - see EmuSen_Interface.md §24.6.
    public static class EngineMarks
    {
        // The astronomical symbols of the cores named for the Moon and the planets, by the engine's own name.
        private static readonly Dictionary<string, PlanetMark> Astronomical = new(StringComparer.OrdinalIgnoreCase)
        {
            ["Moon"] = new() { Stroke = "M15 3.5a8.5 8.5 0 1 0 0 17a7 7 0 1 1 0-17z" },
            ["Mercury"] = new() { Stroke = "M8 3a4 4 0 0 0 8 0M12 6.5a4.5 4.5 0 1 0 0 9a4.5 4.5 0 1 0 0-9M12 15.5V22M9 19h6" },
            ["Venus"] = new() { Stroke = "M12 3a5 5 0 1 0 0 10a5 5 0 1 0 0-10M12 13v9M8.5 18.5h7" },
            ["Mars"] = new() { Stroke = "M10 9a5.5 5.5 0 1 0 0 11a5.5 5.5 0 1 0 0-11M14 10l6-6M15 4h5v5" },
        };

        // Every other EmuSen core's mark is its body's glyph: a shard, a gem, or a small planet.
        public static readonly PlanetMark Shard = new() { Stroke = "M12 3l3.5 17h-7zM6 10l2 10M18 9l-2 11" };
        public static readonly PlanetMark Gem = new() { Stroke = "M4 9l3.5-4.5h9L20 9l-8 11zM4 9h16" };
        public static readonly PlanetMark Planet = new() { Stroke = "M12 5a7 7 0 1 0 0 14a7 7 0 1 0 0-14" };

        // A hosted libretro core's badge: a little pixel alien of the project's own drawing, not libretro's logo.
        public static readonly PlanetMark Libretro = new() { Pixels = ["X.......X", ".X.....X.", ".XXXXXXX.", "XX.XXX.XX", "XXXXXXXXX", ".XX...XX.", ".X.....X.", "XX.....XX"] };

        // An engine's mark: the badge for a libretro core, an astronomical symbol by its name, else the glyph of the body its maker's consoles are drawn as.
        public static PlanetMark Of(string name, bool libretro, string? maker) =>
            libretro ? Libretro
            : Astronomical.TryGetValue(name, out PlanetMark? symbol) ? symbol
            : BigPicture.StartupSplash.BodyOf(null, maker).Shape switch { WorldShape.Shards => Shard, WorldShape.Gem => Gem, _ => Planet };
    }
}
