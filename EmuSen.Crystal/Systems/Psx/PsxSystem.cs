using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Media;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Systems.Psx
{
    // The PlayStation's system pack: its systems-table entry, its recogniser, its cheat codecs, its firmware and its screen - see Diamond_Plan.md §4, Diamond_Players.md.
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
            new[] { ".psexe", ".psx", ".cue", ".bin", ".iso", ".chd", ".m3u" },
            new[] { "Sony - PlayStation" },
            new[] { "PSX" },
            file => file,
            1.0,
            "psx",
            "Sony PlayStation",
            SelfDelimitingImages: false,
            Recognise: Recognise);

        // A disc's format named and not opened yet: unencrypted .pbp, a later step - see Diamond_CDROM.md §8.2.
        public static readonly IReadOnlyList<string> DiscExtensions = new[] { ".pbp" };

        // What the core's own info lists: the pack's but .m3u, which the runtime expands and no core receives - see EmuSen_CoreAPI_Extensions.md §2.4.
        public static readonly IReadOnlyList<string> CoreExtensions = new[] { ".psexe", ".psx", ".cue", ".bin", ".iso", ".chd" };

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

        // The licence's market and the BOOT line's serial, where the first sectors hold them; null for no PlayStation disc - see EmuSen_CoreAPI_Extensions.md §8.4.
        public static Recognition? Recognise(IMediaProbe probe)
        {
            byte[] head = probe.Read(12 * 1024);
            if (!IsDisc(head)) return null;
            var sectors = new DiscSectors(probe.Read(160 * 1024));
            string line2 = Encoding.ASCII.GetString(sectors.Sector(4).Slice(32, Math.Min(38, sectors.Sector(4).Length - 32)));
            string region = line2.Contains("Amer") ? "Americas" : line2.Contains("Euro") ? "Europe" : "Japan";
            return new Recognition(Id, SerialOf(sectors.RootFile("SYSTEM.CNF")), region);
        }

        // SYSTEM.CNF's BOOT line as the game's serial: "cdrom:\SLUS_007.70;1" is SLUS-00770 - see Diamond_Plan.md §4.1.
        public static string? SerialOf(byte[]? config)
        {
            if (config is null) return null;
            foreach (string line in Encoding.ASCII.GetString(config).Split('\n'))
            {
                if (!line.TrimStart().StartsWith("BOOT", StringComparison.OrdinalIgnoreCase) || line.IndexOf('=') < 0) continue;
                string file = line[(line.IndexOf('=') + 1)..].Trim();
                file = file[(Math.Max(file.LastIndexOf('\\'), file.LastIndexOf(':')) + 1)..];
                file = file.Split(';')[0].Replace(".", "").Replace('_', '-').Trim().ToUpperInvariant();
                return file.Length is >= 6 and <= 12 && file.Contains('-') ? file : null;
            }
            return null;
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

        // A code told GameShark or Xploder by its types' first digits, psx-spx's table, as a cheat file's or a player's code is.
        public static ICheatCodeCodec GameShark() => new DelegateCheatCodec("GameShark or Xploder", CheatCodeKind.RamPoke, Ram,
            PsxCheatFormats.IsEither, c => PsxCheatFormats.OneByte(PsxCheatFormats.EitherWrites(c), c), decodeWrites: PsxCheatFormats.EitherWrites);

        public static ICheatCodeCodec Xploder() => new DelegateCheatCodec("Xploder", CheatCodeKind.RamPoke, Ram,
            PsxCheatFormats.IsXploder, c => PsxCheatFormats.OneByte(PsxCheatFormats.XploderWrites(c), c), decodeWrites: PsxCheatFormats.XploderWrites);

        // The memory card, its two slots the core's files 0 and 4 - see EmuSen_SaveMedia.md §7.1.
        public static readonly SaveMedia.SaveMediumEntry Card = new("psx.card", "Memory Card", "Card", ".mcr");

        public static readonly SystemPack Pack = new(Entry, GameShark, Xploder, PlayerControllers, Media: new[] { Card });
    }
}
