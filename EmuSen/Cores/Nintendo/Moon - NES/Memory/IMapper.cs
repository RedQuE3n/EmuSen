namespace EmuSen.Cores.Nintendo.Moon.Memory
{
    // How a cartridge answers the PPU's nametable fetches - see Moon_Memory.md §3.
    public enum Mirroring
    {
        Horizontal,
        Vertical,
        SingleScreenLower,
        SingleScreenUpper,
        FourScreen,
    }

    // The cartridge board. Writes to ROM are how a mapper is configured - see Moon_Memory.md §4.
    public interface IMapper
    {
        string Name { get; }

        // CPU $4020-$FFFF. Open bus is the caller's problem, not the board's.
        byte ReadPrg(ushort address);

        void WritePrg(ushort address, byte data);

        // PPU $0000-$1FFF, the pattern tables.
        byte ReadChr(ushort address);

        void WriteChr(ushort address, byte data);

        Mirroring Mirroring { get; }

        // Called at the end of each rendered scanline for boards that count them - see Moon_Memory.md §4.5.
        void OnScanline() { }

        bool IrqPending => false;
    }
}
