using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // The plugs letting CheatCommand use both decoders without DianaOS referencing this project.
    public static class VenusCheatCodecs
    {
        // Decoded codes target the CPU bus, not a raw WRAM offset, so mirroring resolves as on hardware.
        public static ICheatCodeCodec ActionReplay() =>
            new DelegateCheatCodec("Pro Action Replay/Game Wizard", CheatCodeKind.RamPoke, "CpuBus", ActionReplayCodec.CanDecode, ActionReplayCodec.Decode);

        // "Game Genie", not "SNES Game Genie" - matches CheatCommand's pre-extraction message text exactly.
        public static ICheatCodeCodec GameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, GameGenieCodec.CanDecode, GameGenieCodec.Decode);

        // The pair a loaded SNES game, a console chosen in the UI, and no console at all get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (ActionReplay(), GameGenie());
    }
}
