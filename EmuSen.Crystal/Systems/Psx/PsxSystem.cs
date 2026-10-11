using System;
using System.Collections.Generic;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Systems.Psx
{
    // The PlayStation's system pack: its systems-table entry, the formats it will route, its firmware and its screen - see Diamond_Plan.md §4, Diamond_Native.md §2.4.
    public static class PsxSystem
    {
        public const string Id = "psx";

        // The space name the PlayStation's engines share, so a cheat means the same memory on each.
        public const string Ram = "RAM";

        public static readonly SystemEntry Entry = new(
            Id,
            "Sony PlayStation",
            "PlayStation",
            "Sony",
            1994,
            new[] { ".psexe", ".psx" },
            new[] { "Sony - PlayStation" },
            new[] { "PSX" },
            file => file,
            1.0,
            "psx",
            "Sony PlayStation",
            SelfDelimitingImages: false);

        // A disc's formats, declared and not routed until the disc stage and routing by content - see Diamond_Native.md §2.4.
        public static readonly IReadOnlyList<string> DiscExtensions = new[] { ".cue", ".chd", ".iso", ".m3u", ".pbp" };

        // The BIOS a console of each market holds, all optional, the core's open replacement in their place - see Diamond_Plan.md §5.9.
        public static readonly IReadOnlyList<(string Market, string Name, long Size)> Bios = new[]
        {
            ("Japan", "scph5500.bin", 524_288L),
            ("Americas", "scph5501.bin", 524_288L),
            ("Europe", "scph5502.bin", 524_288L),
        };

        // A television's 4:3, the shape every PlayStation picture is shown in.
        public const double DisplayAspect = 4.0 / 3.0;

        public const string DualShock = "psx.dualshock", DigitalPad = "psx.pad";

        // Each player's pad from the two port settings; a port set to nothing has no player.
        public static IReadOnlyList<string> PlayerControllers(Func<string, string?> setting)
        {
            var players = new List<string>();
            foreach (var (key, fallback) in new[] { ("pad1", DualShock), ("pad2", DualShock) })
                if ((setting(key) ?? fallback) is var pad && pad != "none") players.Add(pad);
            return players;
        }

        public static readonly SystemPack Pack = new(Entry, null, null, PlayerControllers);
    }
}
