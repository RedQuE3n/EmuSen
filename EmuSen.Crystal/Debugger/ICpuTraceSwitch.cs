namespace EmuSen.Crystal.Debugger
{
    // Injected, because counting instructions only means something to a core with a CPU loop - see §3.3b.
    public interface ICpuTraceSwitch
    {
        void Arm(int instructionCount);
        void Disarm();
    }
}
