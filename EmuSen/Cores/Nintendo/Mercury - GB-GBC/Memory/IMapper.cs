using System;
using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mercury.Memory
{
    // The cartridge board. Writes into ROM space are how a board is configured - see Mercury_Memory.md §4.
    public interface IMapper
    {
        string Name { get; }

        // $0000-$7FFF.
        byte ReadRom(ushort address);

        void WriteRom(ushort address, byte data);

        // $A000-$BFFF. Returns $FF when RAM is absent or not enabled, matching an open bus.
        byte ReadRam(ushort address);

        void WriteRam(ushort address, byte data);

        // Clocked once per CPU machine cycle, for boards with a real-time clock - see Mercury_Memory.md §4.4.
        void Tick(int cycles) { }

        // Board registers for `regs`; a board with nothing worth showing reports none.
        IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState =>
            Array.Empty<(string, ulong, int)>();
    }
}
