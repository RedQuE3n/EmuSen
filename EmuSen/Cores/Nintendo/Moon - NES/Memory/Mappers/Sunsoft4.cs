using System.Collections.Generic;
using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 68: four 2K CHR windows, and CHR ROM that can stand in for the nametables - see Moon_Memory.md §4.13.
    public sealed class Sunsoft4 : IMapper
    {
        private const int PrgPageSize = 0x4000;
        private const int ChrPageSize = 0x0800;
        private const int NametableSize = 0x0400;

        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;
        private readonly int[] _chrBanks = new int[4];

        // 1K CHR pages, not 2K ones: the nametable registers count in a different unit to the CHR windows.
        private readonly int[] _nametableBanks = new int[2];

        private bool _useChrForNametables;
        private bool _prgRamEnabled;
        private Mirroring _mirroring = Mirroring.Vertical;

        public Sunsoft4(Cartridge cart) => _cart = cart;

        public string Name => "Sunsoft-4";

        public Mirroring Mirroring => _mirroring;

        public bool SuppliesNametables => _useChrForNametables;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _prgRamEnabled ? _cart.PrgRam[address & 0x1FFF] : (byte)0;

            int last = (_cart.PrgRom.Length / PrgPageSize) - 1;
            int bank = address < 0xC000 ? _prgBank : last;

            int offset = (bank * PrgPageSize) + (address & (PrgPageSize - 1));
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                if (_prgRamEnabled) _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            switch (address & 0xF000)
            {
                case 0x8000: _chrBanks[0] = data; break;
                case 0x9000: _chrBanks[1] = data; break;
                case 0xA000: _chrBanks[2] = data; break;
                case 0xB000: _chrBanks[3] = data; break;

                // The nametable pages live in the upper half of CHR, which is what bit 7 forces.
                case 0xC000: _nametableBanks[0] = data | 0x80; break;
                case 0xD000: _nametableBanks[1] = data | 0x80; break;

                case 0xE000:
                    _mirroring = (data & 0x03) switch
                    {
                        0 => Mirroring.Vertical,
                        1 => Mirroring.Horizontal,
                        2 => Mirroring.SingleScreenLower,
                        _ => Mirroring.SingleScreenUpper,
                    };
                    _useChrForNametables = (data & 0x10) != 0;
                    break;

                case 0xF000:
                    _prgBank = data & 0x07;
                    _prgRamEnabled = (data & 0x10) != 0;
                    break;
            }
        }

        // The board's own mirroring still decides which of the two pages a nametable shows.
        public byte ReadNametable(ushort address)
        {
            int index = (address - 0x2000) & 0x0FFF;
            int table = index / NametableSize;

            int register = _mirroring switch
            {
                Mirroring.Horizontal => table >> 1,
                Mirroring.Vertical => table & 1,
                Mirroring.SingleScreenUpper => 1,
                _ => 0,
            };

            int offset = (_nametableBanks[register] * NametableSize) + (index % NametableSize);
            return _cart.Chr[offset % _cart.Chr.Length];
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
            ("PrgBank", (ulong)_prgBank, 3),
            ("Chr0", (ulong)_chrBanks[0], 8), ("Chr1", (ulong)_chrBanks[1], 8),
            ("Chr2", (ulong)_chrBanks[2], 8), ("Chr3", (ulong)_chrBanks[3], 8),
            ("Nt0", (ulong)_nametableBanks[0], 8), ("Nt1", (ulong)_nametableBanks[1], 8),
            ("ChrNametables", (ulong)(_useChrForNametables ? 1 : 0), 1),
            ("PrgRamEnabled", (ulong)(_prgRamEnabled ? 1 : 0), 1),
        };
    }
}
