namespace EmuSen.DianaOS
{
    // A core's own live CPU instruction-trace arm/disarm switch - see
    // TraceCommand's own header comment on why this needs injecting
    // rather than being a plain DianaOS-owned toggle the way
    // DianaOSLogging.MasterEnabled is: "trace the CPU for the next N
    // instructions" only means something to a core that actually has a
    // CPU with an instruction loop to count against, not something every
    // core necessarily models the same way.
    public interface ICpuTraceSwitch
    {
        void Arm(int instructionCount);
        void Disarm();
    }
}
