using EmuSen.Crystal.Cheats;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The plug letting CheatCommand and Mistress use the decoder without DianaSh referencing this project.
    public static class MarsCheatCodecs
    {
        public static ICheatCodeCodec GameShark() =>
            new DelegateCheatCodec("GameShark", CheatCodeKind.RamPoke, MarsDebugSpaces.Rdram, N64GameSharkCodec.CanDecode, N64GameSharkCodec.Decode, decodeWrites: N64GameSharkCodec.DecodeWrites);

        // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec? Explicit) Pair() => (GameShark(), null);
    }
}
