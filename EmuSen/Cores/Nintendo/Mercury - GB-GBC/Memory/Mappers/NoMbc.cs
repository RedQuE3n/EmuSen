namespace EmuSen.Cores.Nintendo.Mercury.Memory.Mappers
{
    // 32K flat, with the optional 8K of unbanked RAM types $08/$09 add - see Mercury_Memory.md §4.1.
    public sealed class NoMbc : IMapper
    {
        private readonly Cartridge _cart;

        public NoMbc(Cartridge cart) => _cart = cart;

        public string Name => "ROM";

        public byte ReadRom(ushort address) =>
            address < _cart.Rom.Length ? _cart.Rom[address] : (byte)0xFF;

        // Nothing to configure; a game writing here is writing to ROM and nothing happens.
        public void WriteRom(ushort address, byte data) { }

        public byte ReadRam(ushort address)
        {
            int offset = address - 0xA000;
            return offset < _cart.Ram.Length ? _cart.Ram[offset] : (byte)0xFF;
        }

        public void WriteRam(ushort address, byte data)
        {
            int offset = address - 0xA000;
            if (offset < _cart.Ram.Length) _cart.Ram[offset] = data;
        }
    }
}
