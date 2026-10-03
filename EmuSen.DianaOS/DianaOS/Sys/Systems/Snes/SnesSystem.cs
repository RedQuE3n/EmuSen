using System;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Sys.Systems.Snes
{
    // The SNES's system pack: its systems-table entry and its two cheat formats - see EmuSen_CoreAPI.md §20.
    public static class SnesSystem
    {
        public const string Id = "snes";

        // The space names the SNES's engines share, so a cheat means the same memory on each.
        public const string CpuBus = "CpuBus", Wram = "WRAM";

        public static readonly SystemEntry Entry = new(
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
            "Super Nintendo");

        // OpenVGDB hashes an image without the 512-byte copier header some dumps carry, as No-Intro does - see EmuSen_Settings_Reference.md §4.39.
        public static byte[] WithoutCopierHeader(byte[] file) => file.Length % 1024 == 512 ? file[512..] : file;

        public static ICheatCodeCodec ProActionReplay() => new DelegateCheatCodec("Pro Action Replay", CheatCodeKind.RamPoke, CpuBus,
            SnesCheatFormats.IsProActionReplay, SnesCheatFormats.DecodeProActionReplay, decodeWrites: SnesCheatFormats.ProActionReplayWrites);

        public static ICheatCodeCodec GameGenie() => new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null,
            SnesCheatFormats.IsGameGenie, SnesCheatFormats.DecodeGameGenie);

        public static readonly SystemPack Pack = new(Entry, ProActionReplay, GameGenie);
    }
}
