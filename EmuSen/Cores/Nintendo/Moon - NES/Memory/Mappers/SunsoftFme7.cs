using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 69: a command/parameter pair drives everything, and $6000 can be RAM or ROM - see Moon_Memory.md §4.14.
    public sealed class SunsoftFme7 : IMapper
    {
        private const int PrgPageSize = 0x2000;
        private const int ChrPageSize = 0x0400;

        [SkipInState] private readonly Cartridge _cart;

        private int _command;
        private readonly int[] _chrBanks = new int[8];
        private readonly int[] _prgBanks = new int[3];

        // $6000's control byte: bit 6 picks RAM over ROM, bit 7 enables the RAM.
        private int _windowControl;

        private Mirroring _mirroring = Mirroring.Vertical;

        private bool _irqEnabled;
        private bool _irqCounterEnabled;
        private int _irqCounter;
        private bool _irqPending;

        public SunsoftFme7(Cartridge cart) => _cart = cart;

        public string Name => "Sunsoft FME-7";

        public Mirroring Mirroring => _mirroring;

        public bool IrqPending => _irqPending;

        public bool ClocksOnCpuCycle => true;

        public byte ReadPrg(ushort address)
        {
            if (address >= 0x6000 && address < 0x8000) return ReadWindow(address);
            if (address < 0x6000) return 0;

            int window = (address - 0x8000) / PrgPageSize;
            int bank = window == 3 ? (_cart.PrgRom.Length / PrgPageSize) - 1 : _prgBanks[window];

            int offset = (bank * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        // Bit 6 clear leaves the window showing PRG ROM, which is the mode Gimmick! boots in.
        private byte ReadWindow(ushort address)
        {
            if ((_windowControl & 0x40) == 0)
            {
                int offset = ((_windowControl & 0x3F) * PrgPageSize) + (address & 0x1FFF);
                return _cart.PrgRom[offset % _cart.PrgRom.Length];
            }

            return (_windowControl & 0x80) != 0 ? _cart.PrgRam[address & 0x1FFF] : (byte)0;
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address >= 0x6000 && address < 0x8000)
            {
                if ((_windowControl & 0xC0) == 0xC0) _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address & 0xE000)
            {
                case 0x8000: _command = data & 0x0F; break;
                case 0xA000: WriteParameter(data); break;

                // $C000/$E000 are the 5B's own registers; this core carries no expansion audio - see Moon_Memory.md §4.14.
                default: break;
            }
        }

        private void WriteParameter(byte data)
        {
            switch (_command)
            {
                case <= 7:
                    _chrBanks[_command] = data;
                    break;

                case 8:
                    _windowControl = data;
                    break;

                case <= 0xB:
                    _prgBanks[_command - 9] = data & 0x3F;
                    break;

                case 0xC:
                    _mirroring = (data & 0x03) switch
                    {
                        0 => Mirroring.Vertical,
                        1 => Mirroring.Horizontal,
                        2 => Mirroring.SingleScreenLower,
                        _ => Mirroring.SingleScreenUpper,
                    };
                    break;

                case 0xD:
                    _irqEnabled = (data & 0x01) != 0;
                    _irqCounterEnabled = (data & 0x80) != 0;
                    _irqPending = false;
                    break;

                case 0xE:
                    _irqCounter = (_irqCounter & 0xFF00) | data;
                    break;

                default:
                    _irqCounter = (_irqCounter & 0x00FF) | (data << 8);
                    break;
            }
        }

        // The counter wraps and keeps running; only the enable bit stops it - see Moon_Memory.md §4.14.
        public void OnCpuCycle()
        {
            if (!_irqCounterEnabled) return;

            _irqCounter--;
            if (_irqCounter >= 0) return;

            _irqCounter = 0xFFFF;
            if (_irqEnabled) _irqPending = true;
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
            ("Command", (ulong)_command, 4),
            ("Prg8000", (ulong)_prgBanks[0], 6), ("PrgA000", (ulong)_prgBanks[1], 6),
            ("PrgC000", (ulong)_prgBanks[2], 6),
            ("Window6000", (ulong)_windowControl, 8),
            ("IrqCounter", (ulong)_irqCounter, 16),
            ("IrqEnabled", (ulong)(_irqEnabled ? 1 : 0), 1),
            ("IrqCounting", (ulong)(_irqCounterEnabled ? 1 : 0), 1),
            ("IrqPending", (ulong)(_irqPending ? 1 : 0), 1),
        };
    }
}
