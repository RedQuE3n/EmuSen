using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy's two code formats as codecs, which the console's engine and a console chosen in the UI share - see Mercury_Cheats.md §1.
    public static class GameBoyCheatCodecs
    {
        // The space a GameShark poke lands in: the CPU's bus, as the engine names it.
        public const string CpuBus = "CPUBUS";

        // "Game Genie", not "Game Boy Game Genie", so `add`'s detected-format message reads as it always has.
        public static ICheatCodeCodec ManagedGameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, GbGameGenieCodec.CanDecode, GbGameGenieCodec.Decode, GbGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec ManagedGameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, CpuBus, GbGameSharkCodec.CanDecode, GbGameSharkCodec.Decode);

        // Each codec is emusen-systems' under its id, its Managed twin the C# - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("gb.gg", ManagedGameGenie());

        public static ICheatCodeCodec GameShark() => SystemCode.Codec("gb.gs", ManagedGameShark());

        // The pair the engine and a console chosen in the UI get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (GameShark(), GameGenie());
    }
}
