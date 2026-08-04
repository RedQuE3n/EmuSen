using EmuSen.Common;

namespace EmuSen.Cores.Nintendo.Moon.Memory.Mappers
{
    // iNES 71: UxROM's layout, but the bank register lives at $C000-$FFFF - see Moon_Memory.md §4.7.
    public sealed class Camerica : IMapper
    {
        [SkipInState] private readonly Cartridge _cart;

        private int _prgBank;

        // Fire Hawk alone drives single-screen mirroring from $9000 - see Moon_Memory.md §4.7.
        private bool _mirroringControlSeen;
        private bool _upperPage;

        public Camerica(Cartridge cart) => _cart = cart;

        public string Name => "Camerica BF909x";

        public Mirroring Mirroring => _mirroringControlSeen
            ? (_upperPage ? Mirroring.SingleScreenUpper : Mirroring.SingleScreenLower)
            : _cart.HeaderMirroring;

        public byte ReadPrg(ushort address)
        {
            if (address < 0x8000) return _cart.PrgRam[address & 0x1FFF];

            int bank = address < 0xC000 ? _prgBank : _cart.PrgBanks - 1;
            int offset = (bank * Cartridge.PrgBankSize) + (address & 0x3FFF);
            return _cart.PrgRom[offset % _cart.PrgRom.Length];
        }

        public void WritePrg(ushort address, byte data)
        {
            if (address < 0x8000)
            {
                _cart.PrgRam[address & 0x1FFF] = data;
                return;
            }

            // $9000 is the tell; a cart that never writes there keeps its header mirroring.
            if (address >= 0x9000 && address < 0xA000)
            {
                _mirroringControlSeen = true;
                _upperPage = (data & 0x10) != 0;
                return;
            }

            if (address >= 0xC000) _prgBank = data;
        }

        public byte ReadChr(ushort address) => _cart.Chr[address & 0x1FFF];

        public void WriteChr(ushort address, byte data)
        {
            if (_cart.ChrIsRam) _cart.Chr[address & 0x1FFF] = data;
        }
    }
}
