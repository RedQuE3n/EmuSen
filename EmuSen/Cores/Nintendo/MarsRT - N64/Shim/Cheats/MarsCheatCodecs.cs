using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Systems.N64;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The N64 pack's codec under the shim's old name, until the shim goes in B4 - see EmuSen_Backend.md §13.
    public static class MarsCheatCodecs
    {
        public static ICheatCodeCodec GameShark() => N64System.GameShark();

        // The GameShark pokes; no N64 format patches ROM, so the explicit slot stays empty - see Mars_Cheats.md §1.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec? Explicit) Pair() => (GameShark(), null);
    }
}
