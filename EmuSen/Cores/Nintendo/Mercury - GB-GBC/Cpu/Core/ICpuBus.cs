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

        // IE & IF's five live bits, which a dispatch reads again after its first push - see Mercury_Native.md §13.3.
        byte PendingInterrupts { get; }

        // The CPU entering or leaving HALT, during which an hblank HDMA moves nothing - see Mercury_Native.md §14.2.
        void SetHalted(bool halted);
    }
}
