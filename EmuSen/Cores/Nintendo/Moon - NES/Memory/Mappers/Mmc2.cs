using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 9: two CHR windows whose bank is chosen by a latch the PPU's own fetches set - see Moon_Memory.md §4.10.
    public sealed class Mmc2 : IMapper
    {
        private const int PrgPageSize = 0x2000;
        private const int ChrPageSize = 0x1000;

        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;

        // Indexed by the latch: [0] is the $FD bank, [1] the $FE one.
        private readonly int[] _leftBank = new int[2];
        private readonly int[] _rightBank = new int[2];

        // Power-on state is the $FE bank on both windows, matching the reference.
        private int _leftLatch = 1;
        private int _rightLatch = 1;

        private bool _horizontal;

        public Mmc2(Cartridge cart) => _cart = cart;

        public string Name => "MMC2";

        public Mirroring Mirroring => _horizontal ? Mirroring.Horizontal : Mirroring.Vertical;

        private int PrgPages => _cart.PrgRom.Length / PrgPageSize;

        // Only the first 8K window moves; the other three are nailed to the last three pages.
        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int window = (address - 0x8000) / PrgPageSize;
            int last = PrgPages - 1;
            int bank = window == 0 ? _prgBank : last - (3 - window);

            int offset = (bank * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        // Registers are decoded from the address's top nibble alone, so each spans 4K.
        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address >> 12)
            {
                case 0xA: _prgBank = data & 0x0F; break;
                case 0xB: _leftBank[0] = data & 0x1F; break;
                case 0xC: _leftBank[1] = data & 0x1F; break;
                case 0xD: _rightBank[0] = data & 0x1F; break;
                case 0xE: _rightBank[1] = data & 0x1F; break;
                case 0xF: _horizontal = (data & 0x01) != 0; break;
            }
        }

        // The fetch that trips a latch is still served by the outgoing bank - see Moon_Memory.md §4.10.
        public byte ReadChr(ushort address)
        {
            int bank = address < ChrPageSize ? _leftBank[_leftLatch] : _rightBank[_rightLatch];
            int offset = (bank * ChrPageSize) + (address & (ChrPageSize - 1));
            byte value = _cart.Chr[offset % _cart.Chr.Length];

            UpdateLatch(address);
            return value;
        }

        // The left window watches two exact addresses; the right one watches two eight-byte runs.
        private void UpdateLatch(ushort address)
        {
            if (address == 0x0FD8) _leftLatch = 0;
            else if (address == 0x0FE8) _leftLatch = 1;
            else if (address >= 0x1FD8 && address <= 0x1FDF) _rightLatch = 0;
            else if (address >= 0x1FE8 && address <= 0x1FEF) _rightLatch = 1;
        }

        public void WriteChr(ushort address, byte data)
        {
            if (!_cart.ChrIsRam) return;

            int bank = address < ChrPageSize ? _leftBank[_leftLatch] : _rightBank[_rightLatch];
            _cart.Chr[((bank * ChrPageSize) + (address & (ChrPageSize - 1))) % _cart.Chr.Length] = data;
        }

        public IReadOnlyList<(string Name, ulong Value, int Bits)> DebugState => new (string, ulong, int)[]
        {
            ("PrgBank", (ulong)_prgBank, 4),
            ("LeftFD", (ulong)_leftBank[0], 5), ("LeftFE", (ulong)_leftBank[1], 5),
            ("RightFD", (ulong)_rightBank[0], 5), ("RightFE", (ulong)_rightBank[1], 5),
            ("LeftLatch", (ulong)_leftLatch, 1), ("RightLatch", (ulong)_rightLatch, 1),
            ("Horizontal", (ulong)(_horizontal ? 1 : 0), 1),
        };
    }
}
