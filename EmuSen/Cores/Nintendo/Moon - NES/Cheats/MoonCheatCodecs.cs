using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Moon.Cheats
{
    // The plugs letting CheatCommand use both decoders without DianaOS referencing this project.
    public static class MoonCheatCodecs
    {
        // "Game Genie", not "NES Game Genie", so the detected-format message reads as it always has.
        public static ICheatCodeCodec GameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, NesGameGenieCodec.CanDecode, NesGameGenieCodec.Decode, NesGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec Raw() =>
            new DelegateCheatCodec("NES address:value", CheatCodeKind.RamPoke, MoonCore.SpaceCpuBus, NesRawCodec.CanDecode, NesRawCodec.Decode);

        // The pair a loaded NES game and a console chosen in the UI both get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (Raw(), GameGenie());
    }
}
