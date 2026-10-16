using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Systems.Genesis
{
    // The Genesis's three system packs, the console and its two attachments, which share the console's two cheat formats - see Nephrite_Plan.md §4.1, §4.5.
    public static class GenesisSystems
    {
        public const string MegaDriveId = "md", MegaCdId = "mcd", S32xId = "32x";

        // The space names the Genesis's engines share, so a cheat means the same memory on each.
        public const string Wram = "WRAM";

        public static readonly SystemEntry MegaDrive = new(
            MegaDriveId,
            "Sega Genesis / Mega Drive",
            "Genesis",
            "Sega",
            1988,
            new[] { ".md", ".gen", ".bin", ".smd" },
            new[] { "Sega - Mega Drive - Genesis" },
            new[] { "MD" },
            file => file,
            0.7,
            "genesis",
            "Sega Genesis",
            SelfDelimitingImages: false,
            Recognise: probe => IsCompactDisc(probe.Read(0x8010)) ? null : new Recognition(MegaDriveId));

        public static readonly SystemEntry MegaCd = new(
            MegaCdId,
            "Sega CD / Mega-CD",
            "Sega CD",
            "Sega",
            1991,
            new[] { ".iso", ".cue", ".chd" },
            new[] { "Sega - Mega-CD - Sega CD" },
            new[] { "SCD" },
            file => file,
            0.7,
            "segacd",
            "Sega CD",
            SelfDelimitingImages: false,
            // A sheet is read by its first track, which the PlayStation's pack also takes; a CHD by its folder's name - see Nephrite_OpenBios.md §11.
            Recognise: probe => IsSegaCd(probe.Read(0x40)) ? new Recognition(MegaCdId) : null,
            AttachedTo: MegaDrive.Console);

        public static readonly SystemEntry S32x = new(
            S32xId,
            "Sega 32X",
            "32X",
            "Sega",
            1994,
            new[] { ".32x" },
            new[] { "Sega - 32X" },
            new[] { "32X" },
            file => file,
            0.7,
            "sega32x",
            "Sega Mega Drive 32X",
            SelfDelimitingImages: false,
            AttachedTo: MegaDrive.Console);

        private static readonly byte[] Sync = { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 };

        // A compact disc's image rather than a cartridge's: a raw track's sync, or an .iso's volume descriptor at sector 16 - Nephrite's own rule, media.rs.
        public static bool IsCompactDisc(ReadOnlySpan<byte> head) =>
            (head.Length >= 12 && head[..12].SequenceEqual(Sync)) || (head.Length >= 0x8006 && head.Slice(0x8001, 5).SequenceEqual("CD001"u8));

        // A Sega CD's disc: SEGADISCSYSTEM at the start of its first sector's data.
        public static bool IsSegaCd(ReadOnlySpan<byte> head)
        {
            int data = head.Length >= 16 && head[..12].SequenceEqual(Sync) ? (head[15] == 1 ? 16 : 24) : 0;
            return head.Length >= data + 14 && head.Slice(data, 14).SequenceEqual("SEGADISCSYSTEM"u8);
        }

        public static ICheatCodeCodec ActionReplay() => new DelegateCheatCodec("Action Replay", CheatCodeKind.RamPoke, Wram,
            GenesisCheatFormats.IsActionReplay, GenesisCheatFormats.DecodeActionReplay, decodeWrites: GenesisCheatFormats.ActionReplayWrites);

        public static ICheatCodeCodec GameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            GenesisCheatFormats.IsPatch, GenesisCheatFormats.DecodePatch, decodeWrites: GenesisCheatFormats.PatchWrites);

        public static readonly SystemPack MegaDrivePack = new(MegaDrive, ActionReplay, GameGenie, PlayerControllers);
        // The Sega CD's backup RAM inside the unit and its RAM cartridge, one format read by SegaCdBramFormat - see EmuSen_SaveMedia.md §6.2, §7.3.
        public static readonly SaveMedia.SaveMediumEntry BackupRam = new("mcd.backup", "Internal Backup RAM", "Backup RAM", ".brm", SegaCdBramFormat.BackupRam),
            RamCartridge = new("mcd.ramcart", "Backup RAM Cartridge", "Cartridge", ".crm", SegaCdBramFormat.RamCartridge);

        public static readonly SystemPack MegaCdPack = new(MegaCd, ActionReplay, GameGenie, PlayerControllers, Media: new[] { RamCartridge, BackupRam });
        public static readonly SystemPack S32xPack = new(S32x, ActionReplay, GameGenie, PlayerControllers);

        public const string Pad3 = "md.pad3", Pad6 = "md.pad6";

        // Each player's pad from the two port settings: port 1's pads, then port 2's unless a 4 Way Play has it - see EmuSen_Settings_Reference.md §4.101.
        public static IReadOnlyList<string> PlayerControllers(Func<string, string?> setting)
        {
            static (int Pads, string Kind, bool BothPorts) Plug(string? value) => value switch
            {
                "md.pad6" => (1, Pad6, false),
                "md.teamplayer3" => (4, Pad3, false),
                "md.teamplayer6" => (4, Pad6, false),
                "md.4way3" => (4, Pad3, true),
                "md.4way6" => (4, Pad6, true),
                _ => (1, Pad3, false),
            };
            var first = Plug(setting("pad1"));
            var players = Enumerable.Repeat(first.Kind, first.Pads).ToList();
            // The 4 Way Play is port 1's choice alone; on port 2 it reads as a pad of its kind.
            if (!first.BothPorts && Plug(setting("pad2")) is var second) players.AddRange(Enumerable.Repeat(second.Kind, second.BothPorts ? 1 : second.Pads));
            return players;
        }
    }
}
