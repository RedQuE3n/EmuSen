using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Systems.Psx
{
    // The PlayStation's system pack: its systems-table entry, its recogniser, its cheat codecs, its firmware and its screen - see Diamond_Plan.md §4, Diamond_Players.md.
    public static class PsxSystem
    {
        public const string Id = "psx";

        // The space name the PlayStation's engines share, so a cheat means the same memory on each.
        public const string Ram = "RAM";

        // The systems table's row, its recogniser the library's - see EmuSen_Backend.md §12, §13.3.
        public static readonly SystemEntry Entry = SystemTable.Entry(Id);

        // A disc's format named and not opened yet: unencrypted .pbp, a later step - see Diamond_CDROM.md §8.2.
        public static readonly IReadOnlyList<string> DiscExtensions = new[] { ".pbp" };

        // What the core's own info lists: the pack's but .m3u, which the runtime expands and no core receives - see EmuSen_CoreAPI_Extensions.md §2.4.
        public static readonly IReadOnlyList<string> CoreExtensions = new[] { ".psexe", ".psx", ".cue", ".bin", ".iso", ".chd" };

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

        // Each codec is emusen-systems' under its id, the first telling GameShark from Xploder by its types' first digits - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec GameShark() => SystemCode.Codec("psx.gs");

        public static ICheatCodeCodec Xploder() => SystemCode.Codec("psx.xp");

        // The memory card, its two slots the core's files 0 and 4: the table's words with the pack's extension and module - see EmuSen_SaveMedia.md §7.1, EmuSen_Backend.md §13.
        public static readonly SaveMedia.SaveMediumEntry Card = SystemTable.Medium(PsxCardFormat.Id, ".mcr", PsxCardFormat.Instance);

        public static readonly SystemPack Pack = new(Entry, GameShark, Xploder, PlayerControllers, Media: new[] { Card });
    }
}
