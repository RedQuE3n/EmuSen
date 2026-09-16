using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Coprocessor two, which on this part is one latch behind a usability bit - see Mars_Fpu.md §8.
    public sealed partial class Cpu
    {
        public const ulong StatusCop2Usable = 1UL << 30;

        // Every index names it, so there is one register here rather than a file - see Mars_Fpu.md §8.
        public ulong Cop2Latch;

        private void ExecuteCop2(uint instruction)
        {
            if ((Cop0[StatusRegister] & StatusCop2Usable) == 0)
            {
                throw Raise(ExceptionCode.CoprocessorUnusable, CurrentPc, coprocessor: 2);
            }

            switch (Rs(instruction))
            {
                case 0x00: Write32(Rt(instruction), (uint)Cop2Latch); return;
                case 0x01: Write(Rt(instruction), Cop2Latch); return;

                // What the control registers hold is not measured anywhere - see Mars_Fpu.md §8.1.
                case 0x02: Write32(Rt(instruction), 0); return;
                case 0x06: return;

                case 0x04: case 0x05: Cop2Latch = Read(Rt(instruction)); return;

                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc, coprocessor: 2);
            }
        }
    }
}
