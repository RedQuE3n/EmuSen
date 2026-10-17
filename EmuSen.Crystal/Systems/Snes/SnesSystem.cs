using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Input;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Systems.Snes
{
    // The SNES's system pack: its systems-table entry, its two cheat formats and its debugger views - see EmuSen_CoreAPI.md §20, §31.
    public static class SnesSystem
    {
        public const string Id = "snes";

        // The space names the SNES's engines share, so a cheat means the same memory on each.
        public const string CpuBus = "CpuBus", Wram = "WRAM";

        // The two ports on the console's front; a multitap is not modelled - see EmuSen_Input.md §8.1.
        public const int Ports = 2;

        // The twelve buttons of the SNES pad, PadButton's first twelve in the pad's own order - see EmuSen_Input.md §3.
        public static readonly IReadOnlyList<PadButton> PadButtons = new[]
        {
            PadButton.B, PadButton.Y, PadButton.Select, PadButton.Start,
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.X, PadButton.L, PadButton.R,
        };

        // The C# literal, the failover; Entry is the systems table's row with this literal's code - see EmuSen_Backend.md §12.
        public static readonly SystemEntry Literal = new(
            Id,
            "Super Nintendo Entertainment System",
            "SNES",
            "Nintendo",
            1990,
            new[] { ".smc", ".sfc" },
            new[] { "Nintendo - Super Nintendo Entertainment System", "Nintendo - Satellaview" },
            new[] { "SNES" },
            WithoutCopierHeader,
            0.73,
            "snes",
            "Super Nintendo",
            SelfDelimitingImages: false,
            ScreenScraperSystem: 4);

        public static readonly SystemEntry Entry = SystemTable.Entry(Literal);

        // OpenVGDB hashes an image without the 512-byte copier header some dumps carry, as No-Intro does - see EmuSen_Settings_Reference.md §4.39.
        public static byte[] WithoutCopierHeader(byte[] file) => file.Length % 1024 == 512 ? file[512..] : file;

        public static ICheatCodeCodec ProActionReplay() => new DelegateCheatCodec("Pro Action Replay", CheatCodeKind.RamPoke, CpuBus,
            SnesCheatFormats.IsProActionReplay, SnesCheatFormats.DecodeProActionReplay, decodeWrites: SnesCheatFormats.ProActionReplayWrites);

        public static ICheatCodeCodec GameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            SnesCheatFormats.IsGameGenie, SnesCheatFormats.DecodeGameGenie);

        public static readonly SystemPack Pack = new(Entry, ProActionReplay, GameGenie, DebugViews: SnesDebugViews.Instance);
    }
}
