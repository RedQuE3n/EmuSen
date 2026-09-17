using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Fpu;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The arithmetic formats, and how a result reaches the control word - see Mars_FpuMath.md §5.
    public sealed partial class Cpu
    {
        public const uint FcsrFlushToZero = 1u << 24;
        public const uint FcsrCondition = 1u << 23;

        // The two enables whose presence turns an underflow into a refusal - see Mars_FpuMath.md §4.1.
        private const uint UnderflowWatchers = (SoftFloatMath.Underflow | SoftFloatMath.Inexact) << 7;

        private const int FcsrCauseShift = 12;
        private const int FcsrFlagShift = 2;

        private void ExecuteCop1Format(uint instruction, bool wide)
        {
            var format = wide ? FloatFormat.Double : FloatFormat.Single;

            ulong left = ReadFpuSource(Fs(instruction), wide);
            ulong right = ReadFpuOperand(Ft(instruction), wide);

            uint mode = Fcsr & 3;
            bool flush = (Fcsr & FcsrFlushToZero) != 0;

            uint funct = instruction & 0x3F;

            // The sign operations never round, but they do classify, and a move does neither - see §5.1.
            switch (funct)
            {
                case 0x05: Deliver(SoftFloatMath.Sign(left, format, negate: false), Fd(instruction), wide); return;
                case 0x07: Deliver(SoftFloatMath.Sign(left, format, negate: true), Fd(instruction), wide); return;

                // A move is a register move and carries the whole register, whatever the format says - see §5.2.
                case 0x06: WriteFpuResult(Fd(instruction), ReadFpuSource(Fs(instruction), true), true); return;
            }

            var other = wide ? FloatFormat.Single : FloatFormat.Double;

            FloatResult result = funct switch
            {
                0x00 => SoftFloatMath.Add(left, right, format, mode, flush),
                0x01 => SoftFloatMath.Subtract(left, right, format, mode, flush),
                0x02 => SoftFloatMath.Multiply(left, right, format, mode, flush),
                0x03 => SoftFloatMath.Divide(left, right, format, mode, flush),
                0x04 => SoftFloatMath.SquareRoot(left, format, mode, flush),

                0x08 => SoftFloatConvert.ToInteger(left, format, true, SoftFloatMath.RoundNearest),
                0x09 => SoftFloatConvert.ToInteger(left, format, true, SoftFloatMath.RoundZero),
                0x0A => SoftFloatConvert.ToInteger(left, format, true, SoftFloatMath.RoundPositive),
                0x0B => SoftFloatConvert.ToInteger(left, format, true, SoftFloatMath.RoundNegative),
                0x0C => SoftFloatConvert.ToInteger(left, format, false, SoftFloatMath.RoundNearest),
                0x0D => SoftFloatConvert.ToInteger(left, format, false, SoftFloatMath.RoundZero),
                0x0E => SoftFloatConvert.ToInteger(left, format, false, SoftFloatMath.RoundPositive),
                0x0F => SoftFloatConvert.ToInteger(left, format, false, SoftFloatMath.RoundNegative),

                // Converting a format to itself is the one encoding the unit refuses - see Mars_FpuMath.md §6.
                0x20 or 0x21 => wide == (funct == 0x21)
                    ? FloatResult.Refused()
                    : SoftFloatConvert.Between(left, format, other, mode, flush),

                0x24 => SoftFloatConvert.ToInteger(left, format, false, mode),
                0x25 => SoftFloatConvert.ToInteger(left, format, true, mode),

                >= 0x30 => Compare(instruction, format, left, right),
                _ => throw RaiseUnimplementedOperation(),
            };

            if (funct >= 0x30) return;

            bool integer = funct is >= 0x08 and <= 0x0F or 0x24 or 0x25;
            bool destinationWide = integer ? funct is (>= 0x08 and <= 0x0B) or 0x25 : funct == 0x21 || (wide && funct < 0x20);

            Deliver(result, Fd(instruction), destinationWide);
        }

        // A raised cause replaces the previous one; only an unraised operation adds to the flags - see §5.1.
        private void Deliver(FloatResult result, int destination, bool wide)
        {
            if (result.Unimplemented) throw RaiseUnimplementedOperation();

            // An underflow anything is waiting for is refused outright rather than reported - see §4.1.
            if ((result.Flags & SoftFloatMath.Underflow) != 0 && (Fcsr & UnderflowWatchers) != 0)
            {
                throw RaiseUnimplementedOperation();
            }

            Fcsr = (Fcsr & ~(FcsrMaskableCauses | FcsrCauseUnimplemented)) | (result.Flags << FcsrCauseShift);

            if (EnabledCause() != 0) throw Raise(ExceptionCode.FloatingPoint, CurrentPc);

            Fcsr |= result.Flags << FcsrFlagShift;

            WriteFpuResult(destination, result.Bits, wide);
        }

        // A computed 32-bit result clears the rest of the register, which a word move does not - see §5.2.
        private void WriteFpuResult(int destination, ulong bits, bool wide) =>
            Fpr[destination] = wide ? bits : (uint)bits;

        // The integer source formats support nothing but the two conversions out of them - see §6.
        private void ExecuteCop1FromInteger(uint instruction, bool wide)
        {
            uint funct = instruction & 0x3F;
            if (funct is not (0x20 or 0x21)) throw RaiseUnimplementedOperation();

            bool toDouble = funct == 0x21;
            var target = toDouble ? FloatFormat.Double : FloatFormat.Single;

            FloatResult result = SoftFloatConvert.FromInteger(
                ReadFpuSource(Fs(instruction), wide), wide, target, Fcsr & 3, (Fcsr & FcsrFlushToZero) != 0);

            Deliver(result, Fd(instruction), toDouble);
        }

        // The condition bit, and which NaN raises, are the two halves this part inverts - see Mars_FpuMath.md §7.
        private FloatResult Compare(uint instruction, in FloatFormat format, ulong left, ulong right)
        {
            uint condition = instruction & 0xF;

            var a = SoftFloat.Unpack(left, format);
            var b = SoftFloat.Unpack(right, format);

            bool unordered = a.IsNan || b.IsNan;

            bool invalid = (condition & 8) != 0
                ? unordered
                : a.Class == FloatClass.Nan || b.Class == FloatClass.Nan;

            bool less = false;
            bool equal = false;

            if (!unordered)
            {
                int order = SoftFloat.Compare(a, b);
                less = order < 0;
                equal = order == 0;
            }

            bool met = ((condition & 4) != 0 && less)
                || ((condition & 2) != 0 && equal)
                || ((condition & 1) != 0 && unordered);

            uint raised = invalid ? SoftFloatMath.Invalid : 0;

            Fcsr = (Fcsr & ~(FcsrMaskableCauses | FcsrCauseUnimplemented)) | (raised << FcsrCauseShift);

            if (EnabledCause() != 0) throw Raise(ExceptionCode.FloatingPoint, CurrentPc);

            Fcsr |= raised << FcsrFlagShift;
            Fcsr = met ? Fcsr | FcsrCondition : Fcsr & ~FcsrCondition;

            return default;
        }

        // Four encodings only: the rest of the sub-opcode's space is reserved - see Mars_FpuMath.md §7.1.
        private void BranchOnCondition(uint instruction)
        {
            int selector = Rt(instruction);
            if (selector > 3) throw RaiseUnimplementedOperation();

            bool wanted = (selector & 1) != 0;
            bool met = ((Fcsr & FcsrCondition) != 0) == wanted;

            BranchIf(met, instruction, likely: (selector & 2) != 0);
        }

        // Half mode masks the low bit of the first source index and of nothing else - see Mars_FpuMath.md §5.3.
        private ulong ReadFpuSource(int index, bool wide) =>
            ReadFpuOperand(FpuFullMode ? index : index & ~1, wide);

        private ulong ReadFpuOperand(int index, bool wide) => wide ? Fpr[index] : (uint)Fpr[index];

        private static int Ft(uint instruction) => (int)((instruction >> 16) & 0x1F);

        private static int Fs(uint instruction) => (int)((instruction >> 11) & 0x1F);

        private static int Fd(uint instruction) => (int)((instruction >> 6) & 0x1F);
    }
}
