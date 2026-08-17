namespace EmuSen.Validation
{
    // The contract a core-agnostic "single-step" ground-truth test target implements - the same pattern.
    public interface ISingleStepTarget
    {
        // Full reset before each test case - must clear any state a previous test could have left behind (a.
        void Reset();

        // Named register access - deliberately string-keyed rather than a fixed struct, since different CPUs.
        void SetRegister(string name, int value);
        int GetRegister(string name);

        // Flat byte-addressable memory access, in whatever address space this target's own test format.
        void SetMemory(int address, byte value);
        byte GetMemory(int address);

        // Executes exactly one instruction from whatever PC/register state was set up via SetRegister above.
        void Step();
    }
}
