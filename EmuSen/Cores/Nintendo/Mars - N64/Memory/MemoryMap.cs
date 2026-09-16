using System;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // Which of the five virtual segments an address falls in - see Mars_Memory.md §1.
    public enum AddressSegment
    {
        KUseg,
        KSeg0,
        KSeg1,
        KSSeg,
        KSeg3,
    }

    // The physical address map, named once so no device repeats a literal - see Mars_Memory.md §2.
    public static class MemoryMap
    {
        public const uint RdramBase = 0x0000_0000;
        public const uint RdramRegistersBase = 0x03F0_0000;

        public const uint SpDmemBase = 0x0400_0000;
        public const uint SpImemBase = 0x0400_1000;
        public const uint SpRegistersBase = 0x0404_0000;
        public const uint SpPcBase = 0x0408_0000;

        public const uint DpCommandBase = 0x0410_0000;
        public const uint DpSpanBase = 0x0420_0000;
        public const uint MiBase = 0x0430_0000;
        public const uint ViBase = 0x0440_0000;
        public const uint AiBase = 0x0450_0000;
        public const uint PiBase = 0x0460_0000;
        public const uint RiBase = 0x0470_0000;
        public const uint SiBase = 0x0480_0000;

        // Nonzero here is how libdragon's IPL3 is told RDRAM is already up - see Mars_TestOracle.md §3.
        public const uint RiSelect = RiBase + 0x0C;

        public const uint CartDomain2Address1 = 0x0500_0000;
        public const uint CartDomain1Address1 = 0x0600_0000;
        public const uint CartDomain2Address2 = 0x0800_0000;
        public const uint CartDomain1Address2 = 0x1000_0000;

        // A development cartridge's text port, which the hardware corpus reports through - see Mars_Memory.md §4.
        public const uint IsViewerBase = 0x13FF_0000;
        public const uint IsViewerSize = 0x1000;

        public const uint PifRomBase = 0x1FC0_0000;
        public const uint PifRamBase = 0x1FC0_07C0;
        public const uint PifRamSize = 64;

        public const uint SpMemSize = 0x1000;

        public static AddressSegment SegmentOf(ulong address)
        {
            uint top = (uint)(address >> 29) & 0x7;
            return top switch
            {
                0 or 1 or 2 or 3 => AddressSegment.KUseg,
                4 => AddressSegment.KSeg0,
                5 => AddressSegment.KSeg1,
                6 => AddressSegment.KSSeg,
                _ => AddressSegment.KSeg3,
            };
        }

        // The two segments that need no TLB: both strip the top bits, and differ only in cacheing.
        public static bool TryTranslateDirect(ulong address, out uint physical)
        {
            switch (SegmentOf(address))
            {
                case AddressSegment.KSeg0:
                case AddressSegment.KSeg1:
                    physical = (uint)(address & 0x1FFF_FFFF);
                    return true;

                default:
                    physical = 0;
                    return false;
            }
        }

        public static bool IsCached(ulong address) => SegmentOf(address) != AddressSegment.KSeg1;
    }
}
