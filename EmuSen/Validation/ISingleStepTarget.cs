namespace EmuSen.Validation
{
    // The contract a core-agnostic "single-step" ground-truth test target
    // implements - the same pattern as Debug/IDebugTarget.cs and
    // Cores/ICore.cs (a narrow interface any core plugs into, with a
    // shared engine - SingleStepTestRunner - built once on top of it).
    //
    // Modeled directly on the SingleStepTests/ProcessorTests methodology
    // already used to validate this project's 65816 CPU and SPC700 CPU:
    // set up a known register+memory state, execute exactly one
    // instruction, and check the resulting state against an
    // independently-generated ground-truth vector. Any CPU-like
    // component - a future core's own main CPU, a coprocessor, or
    // anything else with "registers + addressable memory + an
    // instruction-at-a-time step function" - can implement this once and
    // get the shared runner, JSON-agnostic reporting, and failure
    // diagnostics for free.
    public interface ISingleStepTarget
    {
        // Full reset before each test case - must clear any state a
        // previous test could have left behind (a halted CPU, latched
        // hardware registers, pending interrupts) that isn't part of a
        // test's own declared register set. Skipping this is exactly the
        // bug that poisoned every test after a WAI/STP opcode the first
        // time the 65816 CPU harness was built - one test's leftover
        // state silently broke every later one.
        void Reset();

        // Named register access - deliberately string-keyed rather than a
        // fixed struct, since different CPUs have completely different
        // register sets (65816: a/x/y/s/d/pc/pbr/dbr/p/e; SPC700: a/x/y/
        // sp/pc/psw; a future core's own CPU: whatever it has). A target
        // only needs to recognize whatever names its own test format's
        // JSON actually uses - see each core's own loader for the exact
        // mapping.
        void SetRegister(string name, int value);
        int GetRegister(string name);

        // Flat byte-addressable memory access, in whatever address space
        // this target's own test format assumes (65816: a 16MB flat
        // space; SPC700: 64KB). Real hardware register side effects
        // (auto-incrementing pointers, clear-on-read counters) should
        // still apply here if the target's own opcode execution depends
        // on them - see Cpu65816SingleStepTarget/Spc700SingleStepTarget's
        // own comments for how each handles that.
        void SetMemory(int address, byte value);
        byte GetMemory(int address);

        // Executes exactly one instruction from whatever PC/register
        // state was set up via SetRegister above.
        void Step();
    }
}
