using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 66: one register switches a 32K PRG bank and an 8K CHR bank at once - see Moon_Memory.md §4.7.
    public sealed class GxRom : IMapper
    {
        private const int PrgBankSize = 0x8000;

        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;
        private int _chrBank;

        public GxRom(Cartridge cart) => _cart = cart;

        public string Name => "GxROM";

        public Mirroring Mirroring => _cart.HeaderMirroring;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int offset = (_prgBank * PrgBankSize) + (address & 0x7FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            _prgBank = (data >> 4) & 0x03;
            _chrBank = data & 0x03;
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
