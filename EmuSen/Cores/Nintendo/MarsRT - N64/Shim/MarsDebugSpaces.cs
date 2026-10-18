using EmuSen.Crystal.Debugger;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The machine's memories by name, and which of them a physical address names - see Mars_Debug.md §4.
    public static class MarsDebugSpaces
    {
        public const string Rdram = "RDRAM";
        public const string Dmem = "DMEM";
        public const string Imem = "IMEM";
        public const string PifRam = "PIFRAM";
        public const string Rom = "ROM";

        // The processor's 32-bit virtual addresses, as an int's bits - see Mars_Debug.md §4.
        public const string Cpu = "CPU";

        // Which memory a physical address names given the RDRAM installed, the N64 pack's map - see Mars_Debug.md §4.
        public static PhysicalAddress? Resolve(uint physical, long rdramLength) => EmuSen.Crystal.Systems.N64.N64Views.Physical(physical, rdramLength);
    }
}
