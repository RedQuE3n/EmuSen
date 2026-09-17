using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // One mapping pair: two consecutive pages described by one comparison - see Mars_Tlb.md §1.
    public struct TlbEntry
    {
        public ulong EntryHi;
        public ulong PageMask;
        public ulong EntryLo0;
        public ulong EntryLo1;
    }

    // What a lookup found, which is three different answers rather than a success and a failure.
    public enum TlbResult
    {
        Missing,
        Invalid,
        NotWritable,
        Mapped,
    }

    // The thirty-two entries, searched in order - see Mars_Tlb.md §2.
    public sealed class Tlb
    {
        public const int EntryCount = 32;

        public const ulong EntryLoGlobal = 1UL << 0;
        public const ulong EntryLoValid = 1UL << 1;
        public const ulong EntryLoDirty = 1UL << 2;

        public readonly TlbEntry[] Entries = new TlbEntry[EntryCount];

        public TlbResult TryTranslate(ulong address, ulong asid, bool store, out uint physical, out int index)
        {
            physical = 0;
            index = -1;

            for (int i = 0; i < Entries.Length; i++)
            {
                ref TlbEntry entry = ref Entries[i];

                // The mask covers the pair; one page is half of it, and the bit above chooses which - see Mars_Tlb.md §1.1.
                ulong pairMask = entry.PageMask | 0x1FFF;
                ulong pageMask = pairMask >> 1;

                if (((address ^ entry.EntryHi) & ~pairMask & 0xFFFF_FFFFUL) != 0) continue;

                // A global pair ignores the address-space id, which is how a shared mapping stays shared.
                bool global = (entry.EntryLo0 & entry.EntryLo1 & EntryLoGlobal) != 0;
                if (!global && (entry.EntryHi & 0xFF) != (asid & 0xFF)) continue;

                index = i;

                ulong half = (address & (pageMask + 1)) != 0 ? entry.EntryLo1 : entry.EntryLo0;

                if ((half & EntryLoValid) == 0) return TlbResult.Invalid;
                if (store && (half & EntryLoDirty) == 0) return TlbResult.NotWritable;

                ulong frame = ((half >> 6) & 0x00FF_FFFF) << 12;
                physical = (uint)((frame & ~pageMask) | (address & pageMask));
                return TlbResult.Mapped;
            }

            return TlbResult.Missing;
        }

        // A probe answers only whether something matches, without caring that it is usable.
        public int Probe(ulong entryHi)
        {
            for (int i = 0; i < Entries.Length; i++)
            {
                ref TlbEntry entry = ref Entries[i];

                ulong pairMask = entry.PageMask | 0x1FFF;
                if (((entryHi ^ entry.EntryHi) & ~pairMask & 0xFFFF_FFFFUL) != 0) continue;

                bool global = (entry.EntryLo0 & entry.EntryLo1 & EntryLoGlobal) != 0;
                if (!global && (entry.EntryHi & 0xFF) != (entryHi & 0xFF)) continue;

                return i;
            }

            return -1;
        }
    }
}
