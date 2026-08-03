using System.Collections.Generic;

namespace EmuSen.Validation
{
    // One bus access: <IsWrite> distinguishes the two, since a suite's trace records both.
    public readonly record struct BusAccess(int Address, byte Value, bool IsWrite);

    // Opt-in for a target whose suite ships per-cycle bus activity - see Moon_CPU.md §7.2.
    public interface IBusTraceTarget
    {
        // The accesses the most recent Step() made, in order.
        IReadOnlyList<BusAccess> LastStepTrace { get; }
    }
}
