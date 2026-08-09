namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Core
{
    // All the SM83 can see - see Mercury_Cpu.md §2.
    public interface ICpuBus
    {
        byte Read(ushort address);

        void Write(ushort address, byte data);

        // Runs the rest of the machine forward while the CPU is mid-instruction - see Mercury_Cpu.md §3.
        void Tick(int cycles);

        // What STOP actually does: on a CGB it performs the speed switch KEY1 armed - see Mercury_Cgb.md §5.
        void Stop();
    }
}
