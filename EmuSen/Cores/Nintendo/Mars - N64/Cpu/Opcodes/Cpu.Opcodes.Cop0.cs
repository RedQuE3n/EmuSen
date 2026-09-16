using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Just enough of coprocessor zero to move values and to let the corpus start - see Mars_Cpu.md §8.
    public sealed partial class Cpu
    {
        public const int CountRegister = 9;
        public const int CompareRegister = 11;

        public readonly ulong[] Cop0 = new ulong[32];

        private void ExecuteCop0(uint instruction)
        {
            uint rs = (uint)Rs(instruction);

            // The emulator-extension opcodes, which real hardware ignores and the corpus calls unconditionally.
            if ((rs & 0x10) != 0)
            {
                uint funct = instruction & 0x3F;
                if (funct >= 0x20) return;

                throw Raise(ExceptionCode.ReservedInstruction, Pc);
            }

            switch (rs)
            {
                case 0x00: Write32(Rt(instruction), (uint)ReadCop0(Rd(instruction))); return;
                case 0x01: Write(Rt(instruction), ReadCop0(Rd(instruction))); return;
                case 0x04: WriteCop0(Rd(instruction), (ulong)(long)(int)(uint)Read(Rt(instruction))); return;
                case 0x05: WriteCop0(Rd(instruction), Read(Rt(instruction))); return;
                default: throw Raise(ExceptionCode.ReservedInstruction, Pc);
            }
        }

        // Count comes off the machine clock rather than out of storage - see Mars_Memory.md §3.1.
        private ulong ReadCop0(int register) =>
            register == CountRegister ? _bus.Count : Cop0[register];

        private void WriteCop0(int register, ulong value)
        {
            if (register == CountRegister)
            {
                _bus.SetCount((uint)value);
                return;
            }

            Cop0[register] = value;
        }
    }
}
