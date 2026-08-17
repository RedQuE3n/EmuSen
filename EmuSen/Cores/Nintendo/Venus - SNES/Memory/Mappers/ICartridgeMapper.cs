namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Which device on the cartridge decodes an address - see Venus_Memory.md §2.1a.
    public enum CartridgeRegion
    {
        Unmapped,
        Rom,
        Sram,

        // Coprocessor cartridges only - see Venus_SA1.md §3.
        IRam,
        CoprocessorRegister,
        Sa1Vector,
    }

    // Where a CPU address lands on the cartridge, before bounds/mirroring.
    public readonly struct CartridgeAddress
    {
        public readonly CartridgeRegion Region;
        public readonly int Offset;

        private CartridgeAddress(CartridgeRegion region, int offset)
        {
            Region = region;
            Offset = offset;
        }

        public static readonly CartridgeAddress Unmapped = new(CartridgeRegion.Unmapped, 0);
        public static CartridgeAddress Rom(int offset) => new(CartridgeRegion.Rom, offset);
        public static CartridgeAddress Sram(int offset) => new(CartridgeRegion.Sram, offset);
        public static CartridgeAddress IRam(int offset) => new(CartridgeRegion.IRam, offset);
        public static CartridgeAddress CoprocessorRegister(int offset) => new(CartridgeRegion.CoprocessorRegister, offset);
        public static CartridgeAddress Sa1Vector(int offset) => new(CartridgeRegion.Sa1Vector, offset);
    }

    // One SNES memory map - see Venus_Memory.md §2.1a.
    public interface ICartridgeMapper
    {
        string Name { get; }

        CartridgeAddress Resolve(byte bank, ushort offset);
    }
}
