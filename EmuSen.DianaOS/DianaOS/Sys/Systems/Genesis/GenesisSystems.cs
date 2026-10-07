using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems.Genesis
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
            SelfDelimitingImages: false);

        public static readonly SystemEntry MegaCd = new(
            MegaCdId,
            "Sega CD / Mega-CD",
            "Sega CD",
            "Sega",
            1991,
            new[] { ".iso" },
            new[] { "Sega - Mega-CD - Sega CD" },
            new[] { "SCD" },
            file => file,
            0.7,
            "segacd",
            "Sega CD",
            SelfDelimitingImages: false);

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
            SelfDelimitingImages: false);

        public static ICheatCodeCodec ActionReplay() => new DelegateCheatCodec("Action Replay", CheatCodeKind.RamPoke, Wram,
            GenesisCheatFormats.IsActionReplay, GenesisCheatFormats.DecodeActionReplay, decodeWrites: GenesisCheatFormats.ActionReplayWrites);

        public static ICheatCodeCodec GameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            GenesisCheatFormats.IsPatch, GenesisCheatFormats.DecodePatch, decodeWrites: GenesisCheatFormats.PatchWrites);

        public static readonly SystemPack MegaDrivePack = new(MegaDrive, ActionReplay, GameGenie, PlayerControllers);
        public static readonly SystemPack MegaCdPack = new(MegaCd, ActionReplay, GameGenie, PlayerControllers);
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
