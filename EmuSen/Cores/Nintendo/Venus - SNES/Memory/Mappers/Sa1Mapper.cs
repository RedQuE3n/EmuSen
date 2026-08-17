namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Map mode $23. Unlike LoROM/HiROM this one isn't a fixed function of (bank, offset): the SA-1's bank - see Venus_SA1.md §3.
    public sealed class Sa1Mapper : ICartridgeMapper
    {
        private readonly Coprocessors.Sa1.Sa1 _sa1;

        public Sa1Mapper(Coprocessors.Sa1.Sa1 sa1)
        {
            _sa1 = sa1;
        }

        public string Name => "SA-1";

        public CartridgeAddress Resolve(byte bank, ushort offset) => _sa1.ResolveScpu(bank, offset);
    }
}
