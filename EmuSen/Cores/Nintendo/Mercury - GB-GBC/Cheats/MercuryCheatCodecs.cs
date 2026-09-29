using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mercury.Cheats
{
    // The plugs letting CheatCommand use both decoders without DianaOS referencing this project.
    public static class MercuryCheatCodecs
    {
        // "Game Genie", not "Game Boy Game Genie", so `add`'s detected-format message reads as it always has.
        public static ICheatCodeCodec GameGenie() =>
            new DelegateCheatCodec("Game Genie", CheatCodeKind.RomPatch, null, GbGameGenieCodec.CanDecode, GbGameGenieCodec.Decode, GbGameGenieCodec.DecodeCompare);

        public static ICheatCodeCodec GameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, MercuryCore.SpaceCpuBus, GbGameSharkCodec.CanDecode, GbGameSharkCodec.Decode);

        // The pair both Game Boy engines and a console chosen in the UI get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (GameShark(), GameGenie());
    }
}
