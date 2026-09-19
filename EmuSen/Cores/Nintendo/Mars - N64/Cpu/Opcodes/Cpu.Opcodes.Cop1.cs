using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Everything coprocessor 1 decides from its sub-opcode alone: the moves and the loads - see Mars_Fpu.md §1.
    public sealed partial class Cpu
    {
        private void ExecuteCop1(uint instruction)
        {
            RequireCop1();

            switch (Rs(instruction))
            {
                case 0x00: Write32(Rt(instruction), ReadFpuWord(Rd(instruction))); return;
                case 0x01: Write(Rt(instruction), ReadFpuWide(Rd(instruction))); return;
                case 0x02: Write32(Rt(instruction), ReadFpuControl(Rd(instruction))); return;
                case 0x04: WriteFpuWord(Rd(instruction), (uint)Read(Rt(instruction))); return;
                case 0x05: WriteFpuWide(Rd(instruction), Read(Rt(instruction))); return;
                case 0x06: WriteFpuControl(Rd(instruction), (uint)Read(Rt(instruction))); return;

                case 0x08: BranchOnCondition(instruction); return;
                case 0x10: ExecuteCop1Format(instruction, wide: false); return;
                case 0x11: ExecuteCop1Format(instruction, wide: true); return;
                case 0x14: ExecuteCop1FromInteger(instruction, wide: false); return;
                case 0x15: ExecuteCop1FromInteger(instruction, wide: true); return;

                // Reserved here means the FPU decoded it and refused, not that the CPU failed to - see §5.
                default: throw RaiseUnimplementedOperation();
            }
        }

        private void LoadCop1(uint instruction, bool wide)
        {
            RequireCop1();

            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, wide ? 8 : 4, ExceptionCode.AddressErrorLoad);

            uint physical = TranslateAccess(Mirrored(address, wide ? 8 : 4), address);

            if (wide) WriteFpuWide(Rt(instruction), _bus.Read64(physical));
            else WriteFpuWord(Rt(instruction), _bus.Read32(physical));
        }

        private uint StoreCop1(uint instruction, bool wide)
        {
            RequireCop1();

            ulong address = EffectiveAddress(instruction);
            RequireAlignment(address, wide ? 8 : 4, ExceptionCode.AddressErrorStore);

            uint physical = TranslateAccess(Mirrored(address, wide ? 8 : 4), address, store: true);

            if (wide) _bus.Write64(physical, ReadFpuWide(Rt(instruction)));
            else _bus.Write32(physical, ReadFpuWord(Rt(instruction)));
            return physical;
        }

        // Scaffolding, not emulation: a real operation that Mars has not built stops loudly - see Mars_Fpu.md §6.
        private static NotImplementedException NotBuiltYet(string what) =>
            new($"Mars has not built {what} yet - see Mars_Fpu.md §6.");
    }
}
