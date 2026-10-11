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
            new[] { ".psexe", ".psx", ".chd" },
            new[] { "Sony - PlayStation" },
            new[] { "PSX" },
            file => file,
            1.0,
            "psx",
            "Sony PlayStation",
            SelfDelimitingImages: false);

        // A disc's formats the core opens but the host cannot route yet: a sheet's tracks, .iso and a lone .bin's collisions, .m3u, .pbp - see Diamond_CDROM.md §8.
        public static readonly IReadOnlyList<string> DiscExtensions = new[] { ".cue", ".iso", ".bin", ".m3u", ".pbp" };

        // The licence text's first line, at the start of sector 4's user data (psx-spx "System Area").
        private static readonly byte[] Licence = System.Text.Encoding.ASCII.GetBytes("          Licensed  by          ");

        // Whether an .iso's user sectors or a raw track's 2,352-byte ones hold a PlayStation disc, the recogniser routing by content asks - see EmuSen_CoreAPI_Extensions.md §8.4, Diamond_CDROM.md §8.
        public static bool IsDisc(ReadOnlySpan<byte> image)
        {
            if (image.Length >= 4 * 2048 + 32 && image.Slice(4 * 2048, 32).SequenceEqual(Licence)) return true;
            ReadOnlySpan<byte> sync = stackalloc byte[] { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };
            int raw = 4 * 2352;
            if (image.Length < raw + 24 + 32 || !image[..12].SequenceEqual(sync) || !image.Slice(raw, 12).SequenceEqual(sync)) return false;
            int data = raw + (image[raw + 15] == 1 ? 16 : 24);
            return image.Slice(data, 32).SequenceEqual(Licence);
        }

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
