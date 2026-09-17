namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // Which of the three the machine is running in, derived from Status - see Mars_Privilege.md §1.
    public enum PrivilegeMode
    {
        Kernel,
        Supervisor,
        User,
    }

    // What the map says about an address: the mode decides, so this is not a property of the address.
    public enum SegmentAccess
    {
        Illegal,
        Mapped,
        Direct,
    }

    public readonly struct Segment
    {
        public static readonly Segment Illegal = new(SegmentAccess.Illegal, 0, false);
        public static readonly Segment Mapped = new(SegmentAccess.Mapped, 0, true);

        public readonly SegmentAccess Access;
        public readonly uint Physical;
        public readonly bool Cached;

        private Segment(SegmentAccess access, uint physical, bool cached)
        {
            Access = access;
            Physical = physical;
            Cached = cached;
        }

        public static Segment Direct(uint physical, bool cached) =>
            new(SegmentAccess.Direct, physical, cached);
    }

    // The virtual address map, which is three different maps chosen by mode - see Mars_Privilege.md §2.
    public static class Segments
    {
        // Where the 64-bit map stops describing its own regions and repeats the 32-bit one - see §2.2.
        public const ulong CompatibilityBase = 0xFFFF_FFFF_8000_0000;

        private const ulong RegionOffset = 0x0000_00FF_FFFF_FFFF;
        private const ulong RegionGap = 0x3FFF_FF00_0000_0000;

        // Only the low four gigabytes of each physical window exist, and the rest is not an address.
        private const ulong PhysicalGap = 0x07FF_FFFF_0000_0000;

        private const ulong UncachedAttribute = 2;

        public static Segment Decode(ulong address, PrivilegeMode mode, bool wide) =>
            wide && address < CompatibilityBase ? Wide(address, mode) : Narrow(address, mode);

        private static Segment Narrow(ulong address, PrivilegeMode mode)
        {
            // Outside 64-bit addressing an address must be the sign extension of its own bit 31 - see §2.1.
            if (address != (ulong)(long)(int)(uint)address) return Segment.Illegal;

            uint top = (uint)(address >> 29) & 7;

            if (top < 4) return Segment.Mapped;
            if (mode == PrivilegeMode.User) return Segment.Illegal;
            if (mode == PrivilegeMode.Supervisor) return top == 6 ? Segment.Mapped : Segment.Illegal;

            return top switch
            {
                4 => Segment.Direct((uint)(address & 0x1FFF_FFFF), cached: true),
                5 => Segment.Direct((uint)(address & 0x1FFF_FFFF), cached: false),
                _ => Segment.Mapped,
            };
        }

        private static Segment Wide(ulong address, PrivilegeMode mode)
        {
            bool addressable = (address & RegionGap) == 0;

            switch (address >> 62)
            {
                case 0: return addressable ? Segment.Mapped : Segment.Illegal;
                case 1: return mode != PrivilegeMode.User && addressable ? Segment.Mapped : Segment.Illegal;
                case 3: return mode == PrivilegeMode.Kernel && addressable ? Segment.Mapped : Segment.Illegal;
            }

            // The eight physical windows, which only kernel mode can name at all - see §2.3.
            if (mode != PrivilegeMode.Kernel || (address & PhysicalGap) != 0) return Segment.Illegal;

            return Segment.Direct(
                (uint)address, cached: ((address >> 59) & 7) != UncachedAttribute);
        }
    }
}
