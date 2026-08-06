namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Every 6502 cycle is one bus access, so these two calls are also the CPU's clock - see Moon_CPU.md §2.
    public interface ICpuBus
    {
        byte Read(ushort address);

        void Write(ushort address, byte data);

        // Runs the cycle-accurate peripherals for the cycle this access occupies - see Moon_CPU.md §5.5.
        void Tick() { }

        // Cycles stolen by DMA, which still clock those peripherals - see Moon_CPU.md §5.5.
        void TickStolen(int cycles)
        {
            for (int i = 0; i < cycles; i++) Tick();
        }
    }
}
