using System.Collections.Generic;

namespace EmuSen.Validation
{
    // One ground-truth test case, already normalized into a core-agnostic shape out of whatever JSON.
    public class SingleStepTest
    {
        // Human-readable only; the runner reports it verbatim on a failure.
        public string Name = "";

        public Dictionary<string, int> InitialRegisters = new();
        public List<(int Address, byte Value)> InitialMemory = new();

        public Dictionary<string, int> FinalRegisters = new();
        public List<(int Address, byte Value)> FinalMemory = new();

        // Null when the suite ships no trace, which leaves the runner checking final state only - see IBusTraceTarget.
        public List<BusAccess>? ExpectedTrace;
    }
}
