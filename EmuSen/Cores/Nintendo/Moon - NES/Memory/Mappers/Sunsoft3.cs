using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 67: four 2K CHR windows and a 16-bit IRQ counter written one byte at a time - see Moon_Memory.md §4.12.
    public sealed class Sunsoft3 : IMapper
    {
        private const int PrgPageSize = 0x4000;
        private const int ChrPageSize = 0x0800;

        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;
        private readonly int[] _chrBanks = new int[4];
        private Mirroring _mirroring = Mirroring.Vertical;

        // Which half of the counter the next $C800 write lands in; $D800 resets it to the high one.
        private bool _irqLowNext;

        private bool _irqEnabled;
        private int _irqCounter;
        private bool _irqPending;

        public Sunsoft3(Cartridge cart) => _cart = cart;

        public string Name => "Sunsoft-3";

        public Mirroring Mirroring => _mirroring;

        public bool IrqPending => _irqPending;

        public bool ClocksOnCpuCycle => true;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int last = (_cart.PrgRom.Length / PrgPageSize) - 1;
            int bank = address < 0xC000 ? _prgBank : last;

            int offset = (bank * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address & 0xF800)
            {
                case 0x8800: _chrBanks[0] = data; break;
                case 0x9800: _chrBanks[1] = data; break;
                case 0xA800: _chrBanks[2] = data; break;
                case 0xB800: _chrBanks[3] = data; break;

                case 0xC800:
                    _irqCounter = _irqLowNext
                        ? (_irqCounter & 0xFF00) | data
                        : (_irqCounter & 0x00FF) | (data << 8);
                    _irqLowNext = !_irqLowNext;
                    break;

                case 0xD800:
                    _irqEnabled = (data & 0x10) != 0;
                    _irqLowNext = false;
                    _irqPending = false;
                    break;

                case 0xE800:
                    _mirroring = (data & 0x03) switch
                    {
                        0 => Mirroring.Vertical,
                        1 => Mirroring.Horizontal,
                        2 => Mirroring.SingleScreenLower,
                        _ => Mirroring.SingleScreenUpper,
                    };
                    break;

                case 0xF800: _prgBank = data; break;
            }
        }

        // It fires on the underflow past zero, not on zero itself, then disarms.
        public void OnCpuCycle()
        {
            if (!_irqEnabled) return;

            _irqCounter--;
            if (_irqCounter >= 0) return;

            _irqCounter = 0xFFFF;
            _irqEnabled = false;
            _irqPending = true;
        }

        private int ChrOffset(ushort address) =>
            ((_chrBanks[(address & 0x1FFF) / ChrPageSize] * ChrPageSize) + (address & (ChrPageSize - 1)))
            % _cart.Chr.Length;

        public byte ReadChr(ushort address) => _cart.Chr[ChrOffset(address)];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[ChrOffset(address)] = data;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new (string, ulong, int)[]
        {
            ("PrgBank", (ulong)_prgBank, 8),
            ("Chr0", (ulong)_chrBanks[0], 8), ("Chr1", (ulong)_chrBanks[1], 8),
            ("Chr2", (ulong)_chrBanks[2], 8), ("Chr3", (ulong)_chrBanks[3], 8),
            ("IrqCounter", (ulong)_irqCounter, 16),
            ("IrqEnabled", (ulong)(_irqEnabled ? 1 : 0), 1),
            ("IrqPending", (ulong)(_irqPending ? 1 : 0), 1),
        };
    }
}
