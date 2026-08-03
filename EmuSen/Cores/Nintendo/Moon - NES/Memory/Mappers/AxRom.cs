using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 7: a 32K PRG window, and the board picks the nametable page rather than the header.
    public sealed class AxRom : IMapper
    {
        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;
        private bool _upperNametable;

        public AxRom(Cartridge cart) => _cart = cart;

        public string Name => "AxROM";

        public Mirroring Mirroring => _upperNametable ? Mirroring.SingleScreenUpper : Mirroring.SingleScreenLower;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int offset = (_prgBank * 0x8000) + (address & 0x7FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            _prgBank = data & 0x07;
            _upperNametable = (data & 0x10) != 0;
        }

        public byte ReadChr(ushort address) => _cart.Chr[address & 0x1FFF];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[address & 0x1FFF] = data;
        }
    }
}
