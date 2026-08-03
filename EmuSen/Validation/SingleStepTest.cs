using System.Collections.Generic;

namespace EmuSen.Validation
{
    // One ground-truth test case, already normalized into a core-agnostic
    // shape out of whatever JSON format its source test suite actually
    // uses - SingleStepTests/65816 and TomHarte/ProcessorTests/spc700 use
    // different field names for the same underlying concepts (e.g. "s"
    // vs "sp" for the stack pointer), so each core's own loader is
    // responsible for that mapping; everything downstream (the runner,
    // failure reporting) only ever sees this shape.
    public class SingleStepTest
    {
        // Human-readable only, e.g. "69 n 10" - has no parsed meaning,
        // just identifies which specific test failed in a report.
        public string Name = "";

        public Dictionary<string, int> InitialRegisters = new();
        public List<(int Address, byte Value)> InitialMemory = new();

        public Dictionary<string, int> FinalRegisters = new();
        public List<(int Address, byte Value)> FinalMemory = new();

        // Null when the suite ships no trace, which leaves the runner checking final state only - see IBusTraceTarget.
        public List<BusAccess>? ExpectedTrace;
    }
}
