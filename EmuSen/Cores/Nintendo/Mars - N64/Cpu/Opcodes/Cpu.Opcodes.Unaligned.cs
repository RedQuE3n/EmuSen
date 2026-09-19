using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The loads and stores that merge with the register or the memory they find - see Mars_Cpu.md §7.
    public sealed partial class Cpu
    {
        private void LoadWordLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (int)(access & 3) * 8;

            uint word = _bus.Read32(TranslateAccess(access & ~3UL, address));
            uint kept = shift == 0 ? 0 : (uint)Read(Rt(instruction)) & ((1u << shift) - 1);

            Write32(Rt(instruction), (word << shift) | kept);
        }

        private void LoadWordRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (3 - (int)(access & 3)) * 8;

            uint word = _bus.Read32(TranslateAccess(access & ~3UL, address));
            ulong previous = Read(Rt(instruction));
            uint merged = (word >> shift) | ((uint)previous & ~(0xFFFF_FFFFu >> shift));

            // Unlike its partner, a partial merge keeps the register's upper half - see Mars_Cpu.md §7.3.
            if (shift == 0) Write32(Rt(instruction), merged);
            else Write(Rt(instruction), (previous & 0xFFFF_FFFF_0000_0000UL) | merged);
        }

        private void LoadDoubleLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (int)(access & 7) * 8;

            ulong value = _bus.Read64(TranslateAccess(access & ~7UL, address));
            ulong kept = shift == 0 ? 0 : Read(Rt(instruction)) & ((1UL << shift) - 1);

            Write(Rt(instruction), (value << shift) | kept);
        }

        private void LoadDoubleRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (7 - (int)(access & 7)) * 8;

            ulong value = _bus.Read64(TranslateAccess(access & ~7UL, address));
            ulong kept = Read(Rt(instruction)) & ~(0xFFFF_FFFF_FFFF_FFFFUL >> shift);

            Write(Rt(instruction), (value >> shift) | kept);
        }

        private uint StoreWordLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (int)(access & 3) * 8;
            uint physical = TranslateAccess(access & ~3UL, address, store: true);

            uint kept = _bus.Read32(physical) & ~(0xFFFF_FFFFu >> shift);

            _bus.Write32(physical, kept | ((uint)Read(Rt(instruction)) >> shift));
            return physical;
        }

        private uint StoreWordRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (3 - (int)(access & 3)) * 8;
            uint physical = TranslateAccess(access & ~3UL, address, store: true);

            uint kept = shift == 0 ? 0 : _bus.Read32(physical) & ((1u << shift) - 1);

            _bus.Write32(physical, kept | ((uint)Read(Rt(instruction)) << shift));
            return physical;
        }

        private uint StoreDoubleLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (int)(access & 7) * 8;
            uint physical = TranslateAccess(access & ~7UL, address, store: true);

            ulong kept = _bus.Read64(physical) & ~(0xFFFF_FFFF_FFFF_FFFFUL >> shift);

            _bus.Write64(physical, kept | (Read(Rt(instruction)) >> shift));
            return physical;
        }

        private uint StoreDoubleRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            ulong access = Mirrored(address, 1);
            int shift = (7 - (int)(access & 7)) * 8;
            uint physical = TranslateAccess(access & ~7UL, address, store: true);

            ulong kept = shift == 0 ? 0 : _bus.Read64(physical) & ((1UL << shift) - 1);

            _bus.Write64(physical, kept | (Read(Rt(instruction)) << shift));
            return physical;
        }
    }
}
