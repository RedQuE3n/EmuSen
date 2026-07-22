namespace EmuSen.Memory
{
    // A minimal hook for observing bus writes, deliberately living here
    // (under Cores/Snes/Memory, not Debug/) rather than being a debug-
    // toolchain type that MemoryBus would need to reference. Whoever wants
    // to observe writes - SnesDebugTarget's watch registry today, anything
    // else later - implements this and registers itself via
    // MemoryBus.WriteObserver, instead of MemoryBus reaching for a
    // concrete debug type (a WatchRegistry field and a Cpu back-reference
    // used to live directly on MemoryBus for exactly this reason - real
    // emulation state and debug-toolchain plumbing mixed in the same
    // class, purely because that was the convenient place to add a field
    // at the time).
    public interface IWriteObserver
    {
        void OnWrite(string spaceName, int address, byte value);
    }
}
