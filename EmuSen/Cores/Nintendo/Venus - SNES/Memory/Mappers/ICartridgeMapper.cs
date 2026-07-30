namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Which device on the cartridge decodes an address - see Venus_Memory.md §2.1a.
    public enum CartridgeRegion
    {
        Unmapped,
        Rom,
        Sram,
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
    }

    // One SNES memory map. Pure address arithmetic - no ROM/SRAM bytes, no
    // bounds checks, no mirroring; Cartridge owns all of that so a mapper
    // stays a testable function of (bank, offset). See Venus_Memory.md §2.1a.
    public interface ICartridgeMapper
    {
        string Name { get; }

        CartridgeAddress Resolve(byte bank, ushort offset);
    }
}
