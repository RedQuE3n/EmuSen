using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 64: MMC3's shape with a third switchable PRG bank and a counter that can clock off the CPU - see Moon_Memory.md §4.15.
    public sealed class Rambo1 : IMapper
    {
        private const int PrgPageSize = 0x2000;
        private const int ChrPageSize = 0x0400;

        // A12 has to sit low far longer here than on an MMC3 - see Moon_Memory.md §4.15.
        public const int A12MinimumLowDots = 30;

        // Hardware takes a moment to pull /IRQ down, and the two paths differ - see Moon_Memory.md §4.15.
        private const int PpuIrqDelay = 2;
        private const int CpuIrqDelay = 1;

        [SkipInState] private readonly Cartridge _cart;

        // $8000 keeps the whole byte: register index in bits 0-3, layout in bits 5-7.
        private int _bankSelect;

        private readonly int[] _registers = new int[16];

        private bool _horizontal;

        private readonly A12Watcher _a12 = new(A12MinimumLowDots);

        private bool _irqEnabled;
        private bool _irqCycleMode;
        private bool _irqReload;
        private int _irqCounter;
        private int _irqReloadValue;
        private int _irqDelay;
        private bool _irqPending;

        // In cycle mode the counter clocks once every four CPU cycles, not every one.
        private int _cpuClockDivider;

        // A mode change owes the counter one more clock before it takes effect - see Moon_Memory.md §4.15.
        private bool _forceClock;

        public Rambo1(Cartridge cart) => _cart = cart;

        public string Name => "RAMBO-1";

        public Mirroring Mirroring => _cart.HeaderMirroring == Mirroring.FourScreen
            ? Mirroring.FourScreen
            : _horizontal ? Mirroring.Horizontal : Mirroring.Vertical;

        public bool IrqPending => _irqPending;

        public bool ClocksOnCpuCycle => true;

        // Bit 6 moves the fixed page from the middle of the map to the bottom of it.
        private int PrgBankFor(ushort address)
        {
            int window = (address - 0x8000) / PrgPageSize;
            int last = (_cart.PrgRom.Length / PrgPageSize) - 1;
            if (window == 3) return last;

            return (_bankSelect & 0x40) != 0
                ? window switch { 0 => _registers[15], 1 => _registers[6], _ => _registers[7] }
                : window switch { 0 => _registers[6], 1 => _registers[7], _ => _registers[15] };
        }

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int offset = (PrgBankFor(address) * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address & 0xE001)
            {
                case 0x8000: _bankSelect = data; break;
                case 0x8001: _registers[_bankSelect & 0x0F] = data; break;
                case 0xA000: _horizontal = (data & 0x01) != 0; break;
                case 0xC000: _irqReloadValue = data; break;
                case 0xC001: SetCycleMode((data & 0x01) != 0); break;

                case 0xE000:
                    _irqEnabled = false;
                    _irqPending = false;
                    _irqDelay = 0;
                    break;

                case 0xE001: _irqEnabled = true; break;
            }
        }

        // Leaving cycle mode still owes one clock, which is what keeps Skull & Crossbones in sync.
        private void SetCycleMode(bool cycleMode)
        {
            if (_irqCycleMode && !cycleMode) _forceClock = true;

            _irqCycleMode = cycleMode;
            if (cycleMode) _cpuClockDivider = 0;
            _irqReload = true;
        }

        public void OnCpuCycle()
        {
            if (_irqDelay > 0 && --_irqDelay == 0) _irqPending = true;

            if (!_irqCycleMode && !_forceClock) return;

            _cpuClockDivider = (_cpuClockDivider + 1) & 0x03;
            if (_cpuClockDivider != 0) return;

            ClockCounter(CpuIrqDelay);
            _forceClock = false;
        }

        // The scanline path is disconnected entirely while the counter runs off the CPU.
        public void OnPpuAddress(ushort address, long ppuClock)
        {
            if (_irqCycleMode) return;
            if (_a12.Rose(address, ppuClock)) ClockCounter(PpuIrqDelay);
        }

        // A reload lands one or two above the latch, not on it - see Moon_Memory.md §4.15.
        private void ClockCounter(int delay)
        {
            if (_irqReload)
            {
                _irqCounter = (_irqReloadValue + (_irqReloadValue <= 1 ? 1 : 2)) & 0xFF;
                _irqReload = false;
            }
            else if (_irqCounter == 0)
            {
                _irqCounter = (_irqReloadValue + 1) & 0xFF;
            }

            _irqCounter = (_irqCounter - 1) & 0xFF;
            if (_irqCounter == 0 && _irqEnabled) _irqDelay = delay;
        }

        // Bit 7 swaps the halves as MMC3's does; bit 5 splits the two 2K pairs into 1K pages.
        private int ChrPageFor(ushort address)
        {
            int window = (address & 0x1FFF) / ChrPageSize;
            if ((_bankSelect & 0x80) != 0) window ^= 0x04;

            bool oneKilobyteMode = (_bankSelect & 0x20) != 0;

            return window switch
            {
                0 => _registers[0],
                1 => oneKilobyteMode ? _registers[8] : _registers[0] + 1,
                2 => _registers[1],
                3 => oneKilobyteMode ? _registers[9] : _registers[1] + 1,
                4 => _registers[2],
                5 => _registers[3],
                6 => _registers[4],
                _ => _registers[5],
            };
        }

        private int ChrOffset(ushort address) =>
            ((ChrPageFor(address) * ChrPageSize) + (address & (ChrPageSize - 1))) % _cart.Chr.Length;

        public byte ReadChr(ushort address) => _cart.Chr[ChrOffset(address)];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[ChrOffset(address)] = data;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new (string, ulong, int)[]
        {
            ("BankSelect", (ulong)_bankSelect, 8),
            ("R0", (ulong)_registers[0], 8), ("R1", (ulong)_registers[1], 8),
            ("R2", (ulong)_registers[2], 8), ("R3", (ulong)_registers[3], 8),
            ("R4", (ulong)_registers[4], 8), ("R5", (ulong)_registers[5], 8),
            ("R6", (ulong)_registers[6], 8), ("R7", (ulong)_registers[7], 8),
            ("R8", (ulong)_registers[8], 8), ("R9", (ulong)_registers[9], 8),
            ("R15", (ulong)_registers[15], 8),
            ("IrqLatch", (ulong)_irqReloadValue, 8),
            ("IrqCounter", (ulong)_irqCounter, 8),
            ("IrqCycleMode", (ulong)(_irqCycleMode ? 1 : 0), 1),
            ("IrqEnabled", (ulong)(_irqEnabled ? 1 : 0), 1),
            ("IrqPending", (ulong)(_irqPending ? 1 : 0), 1),
        };
    }
}
