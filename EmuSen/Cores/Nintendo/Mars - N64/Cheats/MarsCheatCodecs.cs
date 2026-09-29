using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mars.Cheats
{
    // The plug letting CheatCommand and Mistress use the decoder without DianaOS referencing this project.
    public static class MarsCheatCodecs
    {
        public static ICheatCodeCodec GameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, MarsCore.SpaceRdram, N64GameSharkCodec.CanDecode, N64GameSharkCodec.Decode, decodeWrites: N64GameSharkCodec.DecodeWrites);

        // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec? Explicit) Pair() => (GameShark(), null);
    }
}
