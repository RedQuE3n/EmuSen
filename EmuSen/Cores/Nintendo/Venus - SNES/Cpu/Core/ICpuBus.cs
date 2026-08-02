namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // The whole bus surface a 65816 needs, so one Cpu class can drive both
    // the S-CPU (MemoryBus) and the SA-1's own CPU (Sa1Bus) - see Venus_SA1.md §2.1.
    public interface ICpuBus
    {
        byte Read8(uint address);

        void Write8(uint address, byte data);

        // Master clocks per bus access at this address - see Venus_Memory.md §1.6.
        int GetAccessSpeedCycles(uint address);

        // Cycles stolen by a DMA the last instruction kicked off, cleared on read - see Venus_SA1.md §2.1.
        int TakePendingDmaCycles();
    }
}
