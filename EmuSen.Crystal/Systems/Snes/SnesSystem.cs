using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.Snes
{
    // The SNES's system pack: its systems-table entry, its two cheat formats and its debugger views - see EmuSen_CoreAPI.md §20, §31.
    public static class SnesSystem
    {
        public const string Id = "snes";

        // The space names the SNES's engines share, so a cheat means the same memory on each.
        public const string CpuBus = "CpuBus", Wram = "WRAM";

        // The systems table's row - see EmuSen_Backend.md §12.
        public static readonly SystemEntry Entry = SystemTable.Entry(Id);

        // Each codec is emusen-systems' under its id - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec ProActionReplay() => SystemCode.Codec("snes.par");

        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("snes.gg");

        // A Game Genie code for a patch, the pack's encoder.
        public static string EncodeGameGenie(int address, byte value) => SystemCode.CheatEncode("snes.gg", address, value);

        public static readonly SystemPack Pack = new(Entry, ProActionReplay, GameGenie, DebugViews: SystemCode.Views(Id));
    }
}
