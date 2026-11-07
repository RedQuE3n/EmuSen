using System.Collections.Generic;
using EmuSen.Crystal.Cheats;

namespace EmuSen.Crystal.Systems.Nes
{
    // The NES's system pack: its systems-table entry, its two cheat formats and its debugger views - see EmuSen_CoreAPI.md §20, §31.
    public static class NesSystem
    {
        public const string Id = "nes";

        // The space names every NES engine publishes, so a cheat or a view means the same memory on each.
        public const string Ram = "RAM", PrgRam = "PRGRAM", Chr = "CHR", Ciram = "CIRAM", Oam = "OAM", Palette = "PALETTE", CpuBus = "CPUBUS", PpuRegisters = "PPUREG", ApuRegisters = "APUREG";

        // The systems table's row, Famicom Disk System being other hardware and not claimed - see EmuSen_Backend.md §12.
        public static readonly SystemEntry Entry = SystemTable.Entry(Id);

        // Each codec is emusen-systems' under its id - see EmuSen_Backend.md §13.
        public static ICheatCodeCodec GameGenie() => SystemCode.Codec("nes.gg");

        public static ICheatCodeCodec Raw() => SystemCode.Codec("nes.raw");

        // The pair a loaded NES game and a console chosen in the UI both get: detected, then explicit.
        public static (ICheatCodeCodec AutoDetect, ICheatCodeCodec Explicit) Pair() => (Raw(), GameGenie());

        // A mapper's board by name; null for a number the pack names none for.
        public static string? BoardName(int mapper) => SystemCode.BoardName(mapper);

        public static readonly SystemPack Pack = new(Entry, Raw, GameGenie);
    }
}
