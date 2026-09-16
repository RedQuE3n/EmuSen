using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The whole decode, as a switch per field group - see Mars_Cpu.md §1.1.
    public sealed partial class Cpu
    {
        private int _extraCycles;

        private void Execute(uint instruction)
        {
            uint op = instruction >> 26;

            switch (op)
            {
                case 0x00: ExecuteSpecial(instruction); return;
                case 0x01: ExecuteRegImm(instruction); return;
                case 0x02: Jump(instruction); return;
                case 0x03: JumpAndLink(instruction); return;

                case 0x04: BranchIf(Read(Rs(instruction)) == Read(Rt(instruction)), instruction, likely: false); return;
                case 0x05: BranchIf(Read(Rs(instruction)) != Read(Rt(instruction)), instruction, likely: false); return;
                case 0x06: BranchIf((long)Read(Rs(instruction)) <= 0, instruction, likely: false); return;
                case 0x07: BranchIf((long)Read(Rs(instruction)) > 0, instruction, likely: false); return;

                case 0x08: AddImmediate(instruction, trapOnOverflow: true); return;
                case 0x09: AddImmediate(instruction, trapOnOverflow: false); return;
                case 0x0A: SetLessThanImmediate(instruction, unsigned: false); return;
                case 0x0B: SetLessThanImmediate(instruction, unsigned: true); return;
                case 0x0C: Write(Rt(instruction), Read(Rs(instruction)) & Immediate(instruction)); return;
                case 0x0D: Write(Rt(instruction), Read(Rs(instruction)) | Immediate(instruction)); return;
                case 0x0E: Write(Rt(instruction), Read(Rs(instruction)) ^ Immediate(instruction)); return;
                case 0x0F: Write32(Rt(instruction), (uint)(Immediate(instruction) << 16)); return;

                case 0x10: ExecuteCop0(instruction); return;

                case 0x14: BranchIf(Read(Rs(instruction)) == Read(Rt(instruction)), instruction, likely: true); return;
                case 0x15: BranchIf(Read(Rs(instruction)) != Read(Rt(instruction)), instruction, likely: true); return;
                case 0x16: BranchIf((long)Read(Rs(instruction)) <= 0, instruction, likely: true); return;
                case 0x17: BranchIf((long)Read(Rs(instruction)) > 0, instruction, likely: true); return;

                case 0x18: AddImmediate64(instruction, trapOnOverflow: true); return;
                case 0x19: AddImmediate64(instruction, trapOnOverflow: false); return;

                case 0x1A: LoadDoubleLeft(instruction); return;
                case 0x1B: LoadDoubleRight(instruction); return;

                case 0x20: Load(instruction, 1, signed: true); return;
                case 0x21: Load(instruction, 2, signed: true); return;
                case 0x22: LoadWordLeft(instruction); return;
                case 0x23: Load(instruction, 4, signed: true); return;
                case 0x24: Load(instruction, 1, signed: false); return;
                case 0x25: Load(instruction, 2, signed: false); return;
                case 0x26: LoadWordRight(instruction); return;
                case 0x27: Load(instruction, 4, signed: false); return;
                // Nothing is cached, so every cache operation is already complete - see Mars_Cpu.md §11.
                case 0x2F: return;

                case 0x30: LoadLinked(instruction, 4); return;
                case 0x34: LoadLinked(instruction, 8); return;
                case 0x37: Load(instruction, 8, signed: false); return;

                case 0x28: Store(instruction, 1); return;
                case 0x29: Store(instruction, 2); return;
                case 0x2A: StoreWordLeft(instruction); return;
                case 0x2B: Store(instruction, 4); return;
                case 0x2C: StoreDoubleLeft(instruction); return;
                case 0x2D: StoreDoubleRight(instruction); return;
                case 0x2E: StoreWordRight(instruction); return;
                case 0x38: StoreConditional(instruction, 4); return;
                case 0x3C: StoreConditional(instruction, 8); return;
                case 0x3F: Store(instruction, 8); return;

                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        private void ExecuteSpecial(uint instruction)
        {
            switch (instruction & 0x3F)
            {
                case 0x00: Write32(Rd(instruction), (uint)Read(Rt(instruction)) << Sa(instruction)); return;
                case 0x02: Write32(Rd(instruction), (uint)Read(Rt(instruction)) >> Sa(instruction)); return;
                case 0x03: Write32(Rd(instruction), (uint)((int)(uint)Read(Rt(instruction)) >> Sa(instruction))); return;
                case 0x04: Write32(Rd(instruction), (uint)Read(Rt(instruction)) << (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x06: Write32(Rd(instruction), (uint)Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x07: Write32(Rd(instruction), (uint)((int)(uint)Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x1F))); return;

                case 0x08: JumpRegister(instruction, link: false); return;
                case 0x09: JumpRegister(instruction, link: true); return;

                case 0x0C: throw Raise(ExceptionCode.Syscall, CurrentPc);
                case 0x0D: throw Raise(ExceptionCode.Breakpoint, CurrentPc);

                case 0x10: Write(Rd(instruction), Hi); return;
                case 0x11: Hi = Read(Rs(instruction)); return;
                case 0x12: Write(Rd(instruction), Lo); return;
                case 0x13: Lo = Read(Rs(instruction)); return;

                case 0x14: Write(Rd(instruction), Read(Rt(instruction)) << (int)(Read(Rs(instruction)) & 0x3F)); return;
                case 0x16: Write(Rd(instruction), Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x3F)); return;
                case 0x17: Write(Rd(instruction), (ulong)((long)Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x3F))); return;

                case 0x18: Multiply(instruction, unsigned: false); return;
                case 0x19: Multiply(instruction, unsigned: true); return;
                case 0x1A: Divide(instruction, unsigned: false); return;
                case 0x1B: Divide(instruction, unsigned: true); return;
                case 0x1C: MultiplyDouble(instruction, unsigned: false); return;
                case 0x1D: MultiplyDouble(instruction, unsigned: true); return;
                case 0x1E: DivideDouble(instruction, unsigned: false); return;
                case 0x1F: DivideDouble(instruction, unsigned: true); return;

                case 0x20: Add(instruction, trapOnOverflow: true); return;
                case 0x21: Add(instruction, trapOnOverflow: false); return;
                case 0x22: Subtract(instruction, trapOnOverflow: true); return;
                case 0x23: Subtract(instruction, trapOnOverflow: false); return;
                case 0x24: Write(Rd(instruction), Read(Rs(instruction)) & Read(Rt(instruction))); return;
                case 0x25: Write(Rd(instruction), Read(Rs(instruction)) | Read(Rt(instruction))); return;
                case 0x26: Write(Rd(instruction), Read(Rs(instruction)) ^ Read(Rt(instruction))); return;
                case 0x27: Write(Rd(instruction), ~(Read(Rs(instruction)) | Read(Rt(instruction)))); return;

                case 0x2A: Write(Rd(instruction), (long)Read(Rs(instruction)) < (long)Read(Rt(instruction)) ? 1UL : 0UL); return;
                case 0x2B: Write(Rd(instruction), Read(Rs(instruction)) < Read(Rt(instruction)) ? 1UL : 0UL); return;

                case 0x2C: Add64(instruction, trapOnOverflow: true); return;
                case 0x2D: Add64(instruction, trapOnOverflow: false); return;
                case 0x2E: Subtract64(instruction, trapOnOverflow: true); return;
                case 0x2F: Subtract64(instruction, trapOnOverflow: false); return;

                case 0x38: Write(Rd(instruction), Read(Rt(instruction)) << Sa(instruction)); return;
                case 0x3A: Write(Rd(instruction), Read(Rt(instruction)) >> Sa(instruction)); return;
                case 0x3B: Write(Rd(instruction), (ulong)((long)Read(Rt(instruction)) >> Sa(instruction))); return;
                case 0x3C: Write(Rd(instruction), Read(Rt(instruction)) << (Sa(instruction) + 32)); return;
                case 0x3E: Write(Rd(instruction), Read(Rt(instruction)) >> (Sa(instruction) + 32)); return;
                case 0x3F: Write(Rd(instruction), (ulong)((long)Read(Rt(instruction)) >> (Sa(instruction) + 32))); return;

                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        private void ExecuteRegImm(uint instruction)
        {
            long value = (long)Read(Rs(instruction));

            switch (Rt(instruction))
            {
                case 0x00: BranchIf(value < 0, instruction, likely: false); return;
                case 0x01: BranchIf(value >= 0, instruction, likely: false); return;
                case 0x02: BranchIf(value < 0, instruction, likely: true); return;
                case 0x03: BranchIf(value >= 0, instruction, likely: true); return;

                case 0x10: BranchIf(value < 0, instruction, likely: false, link: true); return;
                case 0x11: BranchIf(value >= 0, instruction, likely: false, link: true); return;
                case 0x12: BranchIf(value < 0, instruction, likely: true, link: true); return;
                case 0x13: BranchIf(value >= 0, instruction, likely: true, link: true); return;

                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        private static int Rs(uint instruction) => (int)((instruction >> 21) & 0x1F);

        private static int Rt(uint instruction) => (int)((instruction >> 16) & 0x1F);

        private static int Rd(uint instruction) => (int)((instruction >> 11) & 0x1F);

        private static int Sa(uint instruction) => (int)((instruction >> 6) & 0x1F);

        private static ulong Immediate(uint instruction) => instruction & 0xFFFF;

        private static long SignedImmediate(uint instruction) => (short)instruction;
    }
}
