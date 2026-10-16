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

        // The physical map's boundaries, as the bus decodes them - see Mars_Memory.md §2.
        private const uint RdramRegistersBase = 0x03F0_0000, SpDmemBase = 0x0400_0000, SpRegistersBase = 0x0404_0000, SpMemSize = 0x1000;
        private const uint CartDomain2Address1 = 0x0500_0000, CartDomain2Address2 = 0x0800_0000, CartDomain1Address2 = 0x1000_0000;
        private const uint PifRomBase = 0x1FC0_0000, PifRamBase = 0x1FC0_07C0, PifRamSize = 64;

        // Which memory a physical address names given the RDRAM installed, or the register block it falls in, not addressable.
        public static PhysicalAddress? Resolve(uint physical, long rdramLength)
        {
            if (physical < rdramLength) return new PhysicalAddress(Rdram, (int)physical);
            if (physical < RdramRegistersBase) return new PhysicalAddress("RDRAM (not installed)", (int)physical, false);
            if (physical < SpDmemBase) return new PhysicalAddress("RDRAM registers", (int)(physical - RdramRegistersBase), false);

            if (physical < SpRegistersBase)
            {
                uint local = (physical - SpDmemBase) % (2 * SpMemSize);
                return local < SpMemSize
                    ? new PhysicalAddress(Dmem, (int)local)
                    : new PhysicalAddress(Imem, (int)(local - SpMemSize));
            }

            if (physical < CartDomain2Address1) return new PhysicalAddress("interface registers", (int)physical, false);
            if (physical < CartDomain2Address2) return new PhysicalAddress("64DD", (int)physical, false);
            if (physical < CartDomain1Address2) return new PhysicalAddress("save chip", (int)(physical - CartDomain2Address2), false);
            if (physical >= PifRamBase && physical < PifRamBase + PifRamSize) return new PhysicalAddress(PifRam, (int)(physical - PifRamBase));
            if (physical < PifRomBase) return new PhysicalAddress(Rom, (int)(physical - CartDomain1Address2));

            return new PhysicalAddress("PIF ROM", (int)(physical - PifRomBase), false);
        }
    }
}
