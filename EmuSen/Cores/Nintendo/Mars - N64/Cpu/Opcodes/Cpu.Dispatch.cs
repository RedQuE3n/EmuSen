using System;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The whole decode, as a switch per field group - see Mars_Cpu.md §1.1.
    public sealed partial class Cpu
    {
        private int _extraCycles;

        // A count of what reaches this switch by opcode, for the recompiler's census when EMUSEN_MARS_CENSUS is set; a constant to the compiler otherwise - see Mars_Recompiler.md §14.
        public static readonly bool Census = Environment.GetEnvironmentVariable("EMUSEN_MARS_CENSUS") == "1";
        public static readonly long[] CensusCounts = new long[256];

        private static void Count(uint instruction)
        {
            uint op = instruction >> 26;
            int index = op switch
            {
                0x00 => 64 + (int)(instruction & 0x3F),
                0x01 => 128 + (int)((instruction >> 16) & 0x1F),
                0x11 => ((instruction >> 21) & 0x1F) is 0x10 or 0x11 ? 192 + (int)(instruction & 0x3F) : 160 + (int)((instruction >> 21) & 0x1F),
                _ => (int)op,
            };
            CensusCounts[index]++;
        }

        private void Execute(uint instruction)
        {
            if (Census) Count(instruction);
            uint op = instruction >> 26;

            // Kernel mode never restricts, so nothing is classified in the common case - see Mars_Privilege.md §4.
            if (Mode != PrivilegeMode.Kernel && !WideAddressing && IsDoubleword(op, instruction))
            {
                throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }

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
                case 0x11: ExecuteCop1(instruction); return;
                case 0x12: ExecuteCop2(instruction); return;

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
                // Nothing is cached, but the address is still checked before nothing happens - see Mars_Cpu.md §13.
                case 0x2F: Cache(instruction); return;

                case 0x31: LoadCop1(instruction, wide: false); return;
                case 0x35: LoadCop1(instruction, wide: true); return;
                case 0x39: StoreCop1(instruction, wide: false); return;
                case 0x3D: StoreCop1(instruction, wide: true); return;

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
                case 0x03: ShiftRightArithmetic(instruction, Sa(instruction)); return;
                case 0x04: Write32(Rd(instruction), (uint)Read(Rt(instruction)) << (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x06: Write32(Rd(instruction), (uint)Read(Rt(instruction)) >> (int)(Read(Rs(instruction)) & 0x1F)); return;
                case 0x07: ShiftRightArithmetic(instruction, (int)(Read(Rs(instruction)) & 0x1F)); return;

                case 0x08: JumpRegister(instruction, link: false); return;
                case 0x09: JumpRegister(instruction, link: true); return;

                // Nothing is reordered or buffered, so the ordering this asks for already holds - see §13.
                case 0x0F: return;

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

                case 0x30 or 0x31 or 0x32 or 0x33 or 0x34 or 0x36: ExecuteTrap(instruction); return;

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

                case 0x08 or 0x09 or 0x0A or 0x0B or 0x0C or 0x0E: ExecuteTrapImmediate(instruction); return;

                case 0x10: BranchIf(value < 0, instruction, likely: false, link: true); return;
                case 0x11: BranchIf(value >= 0, instruction, likely: false, link: true); return;
                case 0x12: BranchIf(value < 0, instruction, likely: true, link: true); return;
                case 0x13: BranchIf(value >= 0, instruction, likely: true, link: true); return;

                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        // The doubleword instructions, which the other two modes may run only with their own addressing bit.
        private static bool IsDoubleword(uint op, uint instruction) => op switch
        {
            0x00 => (instruction & 0x3F) is 0x14 or 0x16 or 0x17 or 0x1C or 0x1D or 0x1E or 0x1F
                or 0x2C or 0x2D or 0x2E or 0x2F or 0x38 or 0x3A or 0x3B or 0x3C or 0x3E or 0x3F,
            0x18 or 0x19 or 0x1A or 0x1B or 0x2C or 0x2D or 0x34 or 0x37 or 0x3C or 0x3F => true,
            _ => false,
        };

        private static int Rs(uint instruction) => (int)((instruction >> 21) & 0x1F);

        private static int Rt(uint instruction) => (int)((instruction >> 16) & 0x1F);

        private static int Rd(uint instruction) => (int)((instruction >> 11) & 0x1F);

        private static int Sa(uint instruction) => (int)((instruction >> 6) & 0x1F);

        private static ulong Immediate(uint instruction) => instruction & 0xFFFF;

        private static long SignedImmediate(uint instruction) => (short)instruction;
    }
}
