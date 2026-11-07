using EmuSen.WiseMan.Bindings.Cheats;

namespace EmuSen.WiseMan.Bindings.Systems.GameBoy
{
    // The Game Boy's two code formats as codecs, which the console's engine and a console chosen in the UI share - see Mercury_Cheats.md §1.
    public static class GameBoyCheatCodecs
    {
        // The space a GameShark poke lands in: the CPU's bus, as the engine names it.
        public const string CpuBus = "CPUBUS";

        // Each codec is emusen-systems' under its id - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("gb.gg");

        public static ICheatCodeCodec GameShark() => SystemCode.Codec("gb.gs");

        // The pair the engine and a console chosen in the UI get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (GameShark(), GameGenie());
    }
}
