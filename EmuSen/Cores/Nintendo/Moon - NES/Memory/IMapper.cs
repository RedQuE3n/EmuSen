using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Moon.Memory
{
    // How a cartridge answers the PPU's nametable fetches - see Moon_Memory.md §3.
    public enum Mirroring
    {
        Horizontal,
        Vertical,
        SingleScreenLower,
        SingleScreenUpper,
        FourScreen,
    }

    // The cartridge board. Writes to ROM are how a mapper is configured - see Moon_Memory.md §4.
    public interface IMapper
    {
        string Name { get; }

        // CPU $4020-$FFFF. Open bus is the caller's problem, not the board's.
        byte ReadPrg(ushort address);

        void WritePrg(ushort address, byte data);

        // PPU $0000-$1FFF, the pattern tables.
        byte ReadChr(ushort address);

        void WriteChr(ushort address, byte data);

        Mirroring Mirroring { get; }

        // Every address the PPU puts on its bus, for boards that watch A12 - see Moon_Memory.md §4.6a.
        void OnPpuAddress(ushort address, long ppuClock) { }

        // Opt-in, because the bus checks it once and a board that says no costs nothing - see Moon_Memory.md §4.8.
        bool ClocksOnCpuCycle => false;

        // Only cycles the PPU also lived through, which is not every CPU cycle here - see Moon_Memory.md §4.8.
        void OnCpuCycle() { }

        // A board that answers the PPU's nametable fetches from its own CHR - see Moon_Memory.md §4.9.
        bool SuppliesNametables => false;

        byte ReadNametable(ushort address) => 0;

        bool IrqPending => false;

        // Board registers for `regs`; a board with nothing worth showing reports none - see Moon_Debug.md §3.1.
        IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState =>
            System.Array.Empty<(string, ulong, int)>();
    }
}
