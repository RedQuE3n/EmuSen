using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 4: eight bank registers plus the scanline IRQ counter - see Moon_Memory.md §4.6.
    public sealed class Mmc3 : IMapper
    {
        private const int PrgPageSize = 0x2000;
        private const int ChrPageSize = 0x0400;

        [SkipInState] private readonly Cartridge _cart;

        // $8000: low three bits pick which of _registers $8001 writes.
        private int _bankSelect;

        private int _prgMode;
        private int _chrMode;

        // R0/R1 are 2K CHR pairs, R2-R5 single 1K pages, R6/R7 8K PRG banks.
        private readonly int[] _registers = new int[8];

        private int _mirroring;
        private bool _wramEnabled = true;
        private bool _wramWriteProtected;

        private int _irqCounter;
        private int _irqReloadValue;
        private bool _irqReload;
        private bool _irqEnabled;
        private bool _irqPending;

        public Mmc3(Cartridge cart) => _cart = cart;

        public string Name => "MMC3";

        // A four-screen cart ignores $A000 - see Moon_Memory.md §3.
        public Mirroring Mirroring => _cart.HeaderMirroring == Mirroring.FourScreen
            ? Mirroring.FourScreen
            : (_mirroring & 0x01) != 0 ? Mirroring.Horizontal : Mirroring.Vertical;

        public bool IrqPending => _irqPending;

        private int PrgPages => _cart.PrgRom.Length / PrgPageSize;

        // Which 8K PRG bank each of the four CPU windows shows; -1/-2 count from the end.
        private int PrgBankFor(ushort address)
        {
            int window = (address - 0x8000) / PrgPageSize;
            int last = PrgPages - 1;

            return _prgMode == 0
                ? window switch { 0 => _registers[6], 1 => _registers[7], 2 => last - 1, _ => last }
                : window switch { 0 => last - 1, 1 => _registers[7], 2 => _registers[6], _ => last };
        }

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000)
            {
                return _wramEnabled ? _cart.PrgRam[address & 0x1FFF] : (byte)0;
            }

            int offset = (PrgBankFor(address) * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        // Registers are picked by address bit 13 pairs plus bit 0, so $8000/$8001 repeat every two bytes.
        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                if (_wramEnabled && !_wramWriteProtected) _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address & 0xE001)
            {
                case 0x8000:
                    _bankSelect = data & 0x07;
                    _prgMode = (data >> 6) & 0x01;
                    _chrMode = (data >> 7) & 0x01;
                    break;

                case 0x8001:
                    // R0 and R1 address 2K pairs, so their low bit is not theirs to set.
                    _registers[_bankSelect] = _bankSelect <= 1 ? data & 0xFE : data;
                    break;

                case 0xA000:
                    _mirroring = data;
                    break;

                case 0xA001:
                    _wramEnabled = (data & 0x80) != 0;
                    _wramWriteProtected = (data & 0x40) != 0;
                    break;

                case 0xC000:
                    _irqReloadValue = data;
                    break;

                case 0xC001:
                    _irqCounter = 0;
                    _irqReload = true;
                    break;

                case 0xE000:
                    _irqEnabled = false;
                    _irqPending = false;
                    break;

                case 0xE001:
                    _irqEnabled = true;
                    break;
            }
        }

        // Which 1K CHR page each of the eight PPU windows shows; $8000 bit 7 swaps the halves.
        private int ChrPageFor(ushort address)
        {
            int window = (address & 0x1FFF) / ChrPageSize;
            if (_chrMode != 0) window ^= 0x04;

            return window switch
            {
                0 => _registers[0] & 0xFE,
                1 => _registers[0] | 0x01,
                2 => _registers[1] & 0xFE,
                3 => _registers[1] | 0x01,
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

        // What `regs` shows for this board - see Moon_Debug.md §3.1.
        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new (string, ulong, int)[]
        {
            ("BankSelect", (ulong)_bankSelect, 3),
            ("PrgMode", (ulong)_prgMode, 1),
            ("ChrMode", (ulong)_chrMode, 1),
            ("R0", (ulong)_registers[0], 8), ("R1", (ulong)_registers[1], 8),
            ("R2", (ulong)_registers[2], 8), ("R3", (ulong)_registers[3], 8),
            ("R4", (ulong)_registers[4], 8), ("R5", (ulong)_registers[5], 8),
            ("R6", (ulong)_registers[6], 8), ("R7", (ulong)_registers[7], 8),
            ("IrqLatch", (ulong)_irqReloadValue, 8),
            ("IrqCounter", (ulong)_irqCounter, 8),
            ("IrqReload", (ulong)(_irqReload ? 1 : 0), 1),
            ("IrqEnabled", (ulong)(_irqEnabled ? 1 : 0), 1),
            ("IrqPending", (ulong)(_irqPending ? 1 : 0), 1),
            ("IrqsFired", _irqsFired, 32),
            ("WramEnabled", (ulong)(_wramEnabled ? 1 : 0), 1),
        };

        // Diagnostic only; a board that never fires is the usual sign of a bad IRQ setup.
        private ulong _irqsFired;

        // A12 must have been low a while for the rise to register - see Moon_Memory.md §4.6a.
        private const int A12MinimumLowClocks = 3;

        // -1 rather than 0, because clock 0 is a real dot the very first fetch can land on.
        private long _a12LowSince = -1;

        // Every PPU fetch is offered here; only a genuine low-to-high A12 transition counts.
        public void OnPpuAddress(ushort address, long ppuClock)
        {
            if ((address & 0x1000) == 0)
            {
                if (_a12LowSince < 0) _a12LowSince = ppuClock;
                return;
            }

            bool rising = _a12LowSince >= 0 && ppuClock - _a12LowSince >= A12MinimumLowClocks;
            _a12LowSince = -1;
            if (!rising) return;

            if (_irqCounter == 0 || _irqReload) _irqCounter = _irqReloadValue;
            else _irqCounter--;

            if (_irqCounter == 0 && _irqEnabled)
            {
                if (!_irqPending) _irqsFired++;
                _irqPending = true;
            }

            _irqReload = false;
        }
    }
}
