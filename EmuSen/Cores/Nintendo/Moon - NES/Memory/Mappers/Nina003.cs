using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 79: its one register sits below the cartridge window, at $4100-$5FFF - see Moon_Memory.md §4.7.
    public sealed class Nina003 : IMapper
    {
        private const int PrgBankSize = 0x8000;

        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;
        private int _chrBank;

        public Nina003(Cartridge cart) => _cart = cart;

        public string Name => "NINA-003-006";

        public Mirroring Mirroring => _cart.HeaderMirroring;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int offset = (_prgBank * PrgBankSize) + (address & 0x7FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if ((address & 0xE100) == 0x4100)
            {
                _prgBank = (data >> 3) & 0x01;
                _chrBank = data & 0x07;
                return;
            }

            if (address >= 0x6000 && address < 0x8000) _cart.PrgRam[address & 0x1FFF] = data;
        }

        private int ChrOffset(ushort address) =>
            ((_chrBank * Cartridge.ChrBankSize) + (address & 0x1FFF)) % _cart.Chr.Length;

        public byte ReadChr(ushort address) => _cart.Chr[ChrOffset(address)];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[ChrOffset(address)] = data;
        }
    }
}
