using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus.Processor;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // What the SA-1's 65816 sees. No WRAM, no PPU, no S-CPU registers - just
    // ROM, BW-RAM, I-RAM and the shared register file, all at a flat two
    // master clocks per access - see Venus_SA1.md §2.
    public sealed class Sa1Bus : ICpuBus
    {
        [SkipInState] private readonly Sa1 _sa1;

        public Sa1Bus(Sa1 sa1)
        {
            _sa1 = sa1;
        }

        public byte Read8(uint address) => _sa1.ReadSa1(address);

        public void Write8(uint address, byte data) => _sa1.WriteSa1(address, data);

        public int GetAccessSpeedCycles(uint address) => Sa1.MasterClocksPerCycle;

        // The SA-1 has no equivalent of the S-CPU's DMA cycle stealing: its own
        // DMA is modelled as instantaneous, so nothing is ever owed here.
        public int TakePendingDmaCycles() => 0;
    }
}
