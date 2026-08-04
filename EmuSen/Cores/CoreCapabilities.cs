using System.Collections.Generic;

namespace EmuSen.Cores
{
    // A core that can attribute its own frame time by subsystem - see EmuSen_Multicore.md §5.
    public interface IFrameProfiler
    {
        // Ordered phases of the last frame; a "parent/child" name is inside that parent, not beside it.
        IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases { get; }
    }

    // A core with more than one separately-haltable processor - see EmuSen_Multicore.md §5.
    public interface ICoprocessorHalt
    {
        bool IsHaltedOnCoprocessor { get; }

        // What to call the processor a halt landed on, for a break message.
        string HaltedProcessorName { get; }
    }

    // Cumulative since power-on; <PerFrameBudget> is what a full-rate frame would allow.
    public readonly record struct CoprocessorClocks(string Name, long Executed, long Offered, long PerFrameBudget);

    // A core whose extra processors run on a clock budget worth reporting - see EmuSen_Multicore.md §5.
    public interface ICoprocessorLoad
    {
        // Empty when the loaded cartridge has no such chip.
        IReadOnlyList<CoprocessorClocks> CoprocessorClocks { get; }
    }

    // A core buffering verbose instruction traces that must reach the log before it closes.
    public interface ITraceFlushable
    {
        void FlushVerboseTrace();
    }
}
