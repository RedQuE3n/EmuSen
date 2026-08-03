using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 2: $8000-$BFFF switches, $C000-$FFFF is nailed to the last bank. CHR is always RAM.
    public sealed class UxRom : IMapper
    {
        [SkipInState] private readonly Cartridge _cart;

        private int _bank;

        public UxRom(Cartridge cart) => _cart = cart;

        public string Name => "UxROM";

        public Mirroring Mirroring => _cart.HeaderMirroring;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int bank = address < 0xC000 ? _bank : _cart.PrgBanks - 1;
            int offset = (bank * Cartridge.PrgBankSize) + (address & 0x3FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000) _cart.PrgRam[address & 0x1FFF] = data;
            else _bank = data & 0x0F;
        }

        public byte ReadChr(ushort address) => _cart.Chr[address & 0x1FFF];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[address & 0x1FFF] = data;
        }
    }
}
