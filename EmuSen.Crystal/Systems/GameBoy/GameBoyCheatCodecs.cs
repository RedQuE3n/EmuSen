using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.GameBoy
{
    // The Game Boy's two code formats as codecs, which the console's engine and a console chosen in the UI share - see Mercury_Cheats.md §1.
    public static class GameBoyCheatCodecs
    {
        // The space a GameShark poke lands in: the CPU's bus, as the engine names it.
        public const string CpuBus = "CPUBUS";

        // "Game Genie", not "Game Boy Game Genie", so `add`'s detected-format message reads as it always has.
        public static ICheatCodeCodec GameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, GbGameGenieCodec.CanDecode, GbGameGenieCodec.Decode, GbGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec GameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, CpuBus, GbGameSharkCodec.CanDecode, GbGameSharkCodec.Decode);

        // The pair the engine and a console chosen in the UI get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (GameShark(), GameGenie());
    }
}
