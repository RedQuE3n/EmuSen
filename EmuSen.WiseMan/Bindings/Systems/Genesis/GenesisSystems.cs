using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.WiseMan.Bindings.Cheats;
using EmuSen.WiseMan.Bindings.Systems;

namespace EmuSen.WiseMan.Bindings.Systems.Genesis
{
    // The Genesis's three system packs, the console and its two attachments, which share the console's two cheat formats - see Nephrite_Plan.md §4.1, §4.5.
    public static class GenesisSystems
    {
        public const string MegaDriveId = "md", MegaCdId = "mcd", S32xId = "32x";

        // The space names the Genesis's engines share, so a cheat means the same memory on each.
        public const string Wram = "WRAM";

        // The systems table's rows - see EmuSen_Backend.md §12.
        public static readonly SystemEntry MegaDrive = SystemTable.Entry(MegaDriveId), MegaCd = SystemTable.Entry(MegaCdId), S32x = SystemTable.Entry(S32xId);

        // Each codec is emusen-systems' under its id - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec ActionReplay() => SystemCode.Codec("md.ar");

        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("md.gg");

        // A Game Genie code for a patch, the pack's encoder.
        public static string EncodeGameGenie(int address, ushort value) => SystemCode.CheatEncode("md.gg", address, value);

        public static readonly SystemPack MegaDrivePack = new(MegaDrive, ActionReplay, GameGenie, PlayerControllers);
        // The Sega CD's backup RAM inside the unit and its RAM cartridge, one format read by SegaCdBramFormat; each the table's words with the pack's extension and module - see EmuSen_SaveMedia.md §6.2, §7.3.
        public static readonly SaveMedia.SaveMediumEntry BackupRam = SystemTable.Medium("mcd.backup", ".brm", SegaCdBramFormat.BackupRam),
            RamCartridge = SystemTable.Medium("mcd.ramcart", ".crm", SegaCdBramFormat.RamCartridge);

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
