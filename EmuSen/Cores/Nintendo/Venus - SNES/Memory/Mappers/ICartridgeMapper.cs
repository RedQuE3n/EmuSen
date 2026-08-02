namespace EmuSen.Cores.Nintendo.Venus.Memory.Mappers
{
    // Which device on the cartridge decodes an address - see Venus_Memory.md §2.1a.
    public enum CartridgeRegion
    {
        Unmapped,
        Rom,
        Sram,

        // Coprocessor cartridges only. IRam is the SA-1's 2KB internal RAM;
        // CoprocessorRegister is whichever chip's register window the address
        // fell in, dispatched by Cartridge to the chip it actually built; and
        // Sa1Vector is the SA-1 substituting a vector for the S-CPU, which is
        // an address decode rather than a value. See Venus_SA1.md §3 and
        // Venus_SuperFX.md §3.
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

    // One SNES memory map. Pure address arithmetic - no ROM/SRAM bytes, no
    // bounds checks, no mirroring; Cartridge owns all of that so a mapper
    // stays a testable function of (bank, offset). See Venus_Memory.md §2.1a.
    public interface ICartridgeMapper
    {
        string Name { get; }

        CartridgeAddress Resolve(byte bank, ushort offset);
    }
}
