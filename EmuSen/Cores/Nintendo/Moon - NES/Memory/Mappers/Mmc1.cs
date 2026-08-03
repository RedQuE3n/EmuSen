using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 1: a serial shift register clocked five writes at a time - see Moon_Memory.md §4.4.
    public sealed class Mmc1 : IMapper
    {
        // Power-on control: PRG mode 3, so the last bank sits at $C000 and the reset vector is reachable.
        private const int ResetControl = 0x0C;

        [SkipInState] private readonly Cartridge _cart;

        private int _shift = 0x10;
        private int _control = ResetControl;
        private int _chrBank0;
        private int _chrBank1;
        private int _prgBank;

        private long _lastWriteCycle = -1;

        public Mmc1(Cartridge cart) => _cart = cart;

        public string Name => "MMC1";

        public Mirroring Mirroring => (_control & 0x03) switch
        {
            0 => Mirroring.SingleScreenLower,
            1 => Mirroring.SingleScreenUpper,
            2 => Mirroring.Vertical,
            _ => Mirroring.Horizontal,
        };

        private int PrgMode => (_control >> 2) & 0x03;

        private bool ChrIsTwoBanks => (_control & 0x10) != 0;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int lastBank = _cart.PrgBanks - 1;
            int bank = PrgMode switch
            {
                // Modes 0 and 1 ignore the low bit and switch all 32K at once.
                0 or 1 => (_prgBank & 0x0E) + (address < 0xC000 ? 0 : 1),
                2 => address < 0xC000 ? 0 : _prgBank,
                _ => address < 0xC000 ? _prgBank : lastBank,
            };

            int offset = (bank * Cartridge.PrgBankSize) + (address & 0x3FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        // A write with bit 7 set clears the register; otherwise five writes assemble one value.
        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            // An RMW opcode writes twice on consecutive cycles and the board only sees the first.
            bool consecutive = _cart.CpuCycle == _lastWriteCycle + 1;
            _lastWriteCycle = _cart.CpuCycle;
            if (consecutive) return;

            if ((data & 0x80) != 0)
            {
                _shift = 0x10;
                _control |= ResetControl;
                return;
            }

            bool complete = (_shift & 0x01) != 0;
            _shift = (_shift >> 1) | ((data & 0x01) << 4);

            if (!complete) return;

            int value = _shift & 0x1F;
            _shift = 0x10;

            switch (address & 0xE000)
            {
                case 0x8000: _control = value; break;
                case 0xA000: _chrBank0 = value; break;
                case 0xC000: _chrBank1 = value; break;
                default: _prgBank = value & 0x0F; break;
            }
        }

        public byte ReadChr(ushort address) => _cart.Chr[ChrOffset(address)];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[ChrOffset(address)] = data;
        }

        private int ChrOffset(ushort address)
        {
            int offset;

            if (ChrIsTwoBanks)
            {
                int bank = address < 0x1000 ? _chrBank0 : _chrBank1;
                offset = (bank * 0x1000) + (address & 0x0FFF);
            }
            else
            {
                offset = ((_chrBank0 & 0x1E) * 0x1000) + (address & 0x1FFF);
            }

            return offset % _cart.Chr.Length;
        }
    }
}
