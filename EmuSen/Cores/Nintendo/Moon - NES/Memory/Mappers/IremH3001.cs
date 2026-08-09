using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 65: three switchable PRG windows and a 16-bit IRQ counter clocked by the CPU - see Moon_Memory.md §4.11.
    public sealed class IremH3001 : IMapper
    {
        private const int PrgPageSize = 0x2000;
        private const int ChrPageSize = 0x0400;

        [SkipInState] private readonly Cartridge _cart;

        private readonly int[] _prgBanks = new int[3];
        private readonly int[] _chrBanks = new int[8];

        private bool _horizontal;

        private bool _irqEnabled;
        private int _irqCounter;
        private int _irqReloadValue;
        private bool _irqPending;

        public IremH3001(Cartridge cart)
        {
            _cart = cart;
            _prgBanks[0] = 0;
            _prgBanks[1] = 1;
            _prgBanks[2] = (_cart.PrgRom.Length / PrgPageSize) - 2;

            for (int i = 0; i < _chrBanks.Length; i++) _chrBanks[i] = i;
        }

        public string Name => "Irem H3001";

        public Mirroring Mirroring => _horizontal ? Mirroring.Horizontal : Mirroring.Vertical;

        public bool IrqPending => _irqPending;

        public bool ClocksOnCpuCycle => true;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int window = (address - 0x8000) / PrgPageSize;
            int bank = window == 3 ? (_cart.PrgRom.Length / PrgPageSize) - 1 : _prgBanks[window];

            int offset = (bank * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        // The board decodes its registers in full, so only these exact addresses do anything.
        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address)
            {
                case 0x8000: _prgBanks[0] = data; break;
                case 0xA000: _prgBanks[1] = data; break;
                case 0xC000: _prgBanks[2] = data; break;

                case 0x9001: _horizontal = (data & 0x80) != 0; break;

                case 0x9003:
                    _irqEnabled = (data & 0x80) != 0;
                    _irqPending = false;
                    break;

                case 0x9004:
                    _irqCounter = _irqReloadValue;
                    _irqPending = false;
                    break;

                case 0x9005: _irqReloadValue = (_irqReloadValue & 0x00FF) | (data << 8); break;
                case 0x9006: _irqReloadValue = (_irqReloadValue & 0xFF00) | data; break;

                default:
                    if (address >= 0xB000 && address <= 0xB007) _chrBanks[address & 0x07] = data;
                    break;
            }
        }

        // Reaching zero fires once and disarms; only a $9004 reload starts it again.
        public void OnCpuCycle()
        {
            if (!_irqEnabled) return;

            _irqCounter = (_irqCounter - 1) & 0xFFFF;
            if (_irqCounter != 0) return;

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
            ("Prg8000", (ulong)_prgBanks[0], 8), ("PrgA000", (ulong)_prgBanks[1], 8),
            ("PrgC000", (ulong)_prgBanks[2], 8),
            ("IrqLatch", (ulong)_irqReloadValue, 16),
            ("IrqCounter", (ulong)_irqCounter, 16),
            ("IrqEnabled", (ulong)(_irqEnabled ? 1 : 0), 1),
            ("IrqPending", (ulong)(_irqPending ? 1 : 0), 1),
            ("Horizontal", (ulong)(_horizontal ? 1 : 0), 1),
        };
    }
}
