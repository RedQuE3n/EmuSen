namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // Every 6502 cycle is one bus access, so these two calls are also the CPU's clock - see Moon_CPU.md §2.
    public interface ICpuBus
    {
        byte Read(ushort address);

        void Write(ushort address, byte data);
    }
}
