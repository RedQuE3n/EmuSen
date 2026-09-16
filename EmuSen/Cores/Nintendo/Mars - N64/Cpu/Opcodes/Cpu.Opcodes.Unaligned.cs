using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The loads and stores that merge with the register or the memory they find - see Mars_Cpu.md §7.
    public sealed partial class Cpu
    {
        private void LoadWordLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (int)(address & 3) * 8;

            uint word = _bus.Read32(Translate(address & ~3UL));
            uint kept = shift == 0 ? 0 : (uint)Read(Rt(instruction)) & ((1u << shift) - 1);

            Write32(Rt(instruction), (word << shift) | kept);
        }

        private void LoadWordRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (3 - (int)(address & 3)) * 8;

            uint word = _bus.Read32(Translate(address & ~3UL));
            uint kept = (uint)Read(Rt(instruction)) & ~(0xFFFF_FFFFu >> shift);

            Write32(Rt(instruction), (word >> shift) | kept);
        }

        private void LoadDoubleLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (int)(address & 7) * 8;

            ulong value = _bus.Read64(Translate(address & ~7UL));
            ulong kept = shift == 0 ? 0 : Read(Rt(instruction)) & ((1UL << shift) - 1);

            Write(Rt(instruction), (value << shift) | kept);
        }

        private void LoadDoubleRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (7 - (int)(address & 7)) * 8;

            ulong value = _bus.Read64(Translate(address & ~7UL));
            ulong kept = Read(Rt(instruction)) & ~(0xFFFF_FFFF_FFFF_FFFFUL >> shift);

            Write(Rt(instruction), (value >> shift) | kept);
        }

        private void StoreWordLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (int)(address & 3) * 8;
            uint physical = Translate(address & ~3UL);

            uint kept = _bus.Read32(physical) & ~(0xFFFF_FFFFu >> shift);

            _bus.Write32(physical, kept | ((uint)Read(Rt(instruction)) >> shift));
        }

        private void StoreWordRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (3 - (int)(address & 3)) * 8;
            uint physical = Translate(address & ~3UL);

            uint kept = shift == 0 ? 0 : _bus.Read32(physical) & ((1u << shift) - 1);

            _bus.Write32(physical, kept | ((uint)Read(Rt(instruction)) << shift));
        }

        private void StoreDoubleLeft(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (int)(address & 7) * 8;
            uint physical = Translate(address & ~7UL);

            ulong kept = _bus.Read64(physical) & ~(0xFFFF_FFFF_FFFF_FFFFUL >> shift);

            _bus.Write64(physical, kept | (Read(Rt(instruction)) >> shift));
        }

        private void StoreDoubleRight(uint instruction)
        {
            ulong address = EffectiveAddress(instruction);
            int shift = (7 - (int)(address & 7)) * 8;
            uint physical = Translate(address & ~7UL);

            ulong kept = shift == 0 ? 0 : _bus.Read64(physical) & ((1UL << shift) - 1);

            _bus.Write64(physical, kept | (Read(Rt(instruction)) << shift));
        }
    }
}
