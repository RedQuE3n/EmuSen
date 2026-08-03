using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 3: PRG is fixed, the whole 8K CHR window switches. Bus conflicts are not modelled.
    public sealed class CnRom : IMapper
    {
        [SkipInState] private readonly Cartridge _cart;

        private int _chrBank;

        public CnRom(Cartridge cart) => _cart = cart;

        public string Name => "CNROM";

        public Mirroring Mirroring => _cart.HeaderMirroring;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int offset = address - 0x8000;
            return _cart.PrgRom[offset & (_cart.PrgRom.Length - 1)];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000) _cart.PrgRam[address & 0x1FFF] = data;
            else _chrBank = data & 0x03;
        }

        public byte ReadChr(ushort address)
        {
            int offset = (_chrBank * Cartridge.ChrBankSize) + (address & 0x1FFF);
            return _cart.Chr[offset % _cart.Chr.Length];
        }

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[address & 0x1FFF] = data;
        }
    }
}
