using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 0: no banking at all. A 16K board mirrors its one bank into both slots.
    public sealed class Nrom : IMapper
    {
        [SkipInState] private readonly Cartridge _cart;

        public Nrom(Cartridge cart) => _cart = cart;

        public string Name => "NROM";

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
        }

        public byte ReadChr(ushort address) => _cart.Chr[address & 0x1FFF];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[address & 0x1FFF] = data;
        }
    }
}
