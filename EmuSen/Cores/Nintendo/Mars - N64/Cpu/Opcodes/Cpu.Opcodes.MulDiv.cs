using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Multiply and divide, the only instructions whose stall counts the vendor manual tabulates - see Mars_Cpu.md §5.
    public sealed partial class Cpu
    {
        public const int MultiplyStall = 5;
        public const int MultiplyDoubleStall = 8;
        public const int DivideStall = 37;
        public const int DivideDoubleStall = 69;

        // How wide the multiplier reads its second operand, which is neither 32 nor 64 - see Mars_Cpu.md §14.
        public const int MultiplyOperandBits = 35;

        private void Multiply(uint instruction, bool unsigned)
        {
            long result = unsigned
                ? (long)((ulong)(uint)Read(Rs(instruction)) * (uint)Read(Rt(instruction)))
                : unchecked((long)Read(Rs(instruction)) * SignExtendOperand(Read(Rt(instruction))));

            Lo = (ulong)(long)(int)result;
            Hi = (ulong)(long)(int)(result >> 32);
            _extraCycles = MultiplyStall;
        }

        private static long SignExtendOperand(ulong value) =>
            (long)(value << (64 - MultiplyOperandBits)) >> (64 - MultiplyOperandBits);

        private void MultiplyDouble(uint instruction, bool unsigned)
        {
            ulong left = Read(Rs(instruction));
            ulong right = Read(Rt(instruction));

            if (unsigned)
            {
                Lo = left * right;
                Hi = System.Math.BigMul(left, right, out _);
            }
            else
            {
                long high = System.Math.BigMul((long)left, (long)right, out long low);
                Lo = (ulong)low;
                Hi = (ulong)high;
            }

            _extraCycles = MultiplyDoubleStall;
        }

        // Division by zero and the one overflowing case have defined results rather than an exception.
        private void Divide(uint instruction, bool unsigned)
        {
            _extraCycles = DivideStall;

            if (unsigned)
            {
                uint left = (uint)Read(Rs(instruction));
                uint right = (uint)Read(Rt(instruction));

                if (right == 0)
                {
                    Lo = 0xFFFF_FFFF_FFFF_FFFF;
                    Hi = (ulong)(long)(int)left;
                    return;
                }

                Lo = (ulong)(long)(int)(left / right);
                Hi = (ulong)(long)(int)(left % right);
                return;
            }

            int signedLeft = (int)(uint)Read(Rs(instruction));
            int signedRight = (int)(uint)Read(Rt(instruction));

            if (signedRight == 0)
            {
                Lo = signedLeft < 0 ? 1UL : 0xFFFF_FFFF_FFFF_FFFF;
                Hi = (ulong)(long)signedLeft;
                return;
            }

            if (signedLeft == int.MinValue && signedRight == -1)
            {
                Lo = unchecked((ulong)(long)int.MinValue);
                Hi = 0;
                return;
            }

            Lo = (ulong)(long)(signedLeft / signedRight);
            Hi = (ulong)(long)(signedLeft % signedRight);
        }

        private void DivideDouble(uint instruction, bool unsigned)
        {
            _extraCycles = DivideDoubleStall;

            if (unsigned)
            {
                ulong left = Read(Rs(instruction));
                ulong right = Read(Rt(instruction));

                if (right == 0)
                {
                    Lo = 0xFFFF_FFFF_FFFF_FFFF;
                    Hi = left;
                    return;
                }

                Lo = left / right;
                Hi = left % right;
                return;
            }

            long signedLeft = (long)Read(Rs(instruction));
            long signedRight = (long)Read(Rt(instruction));

            if (signedRight == 0)
            {
                Lo = signedLeft < 0 ? 1UL : 0xFFFF_FFFF_FFFF_FFFF;
                Hi = (ulong)signedLeft;
                return;
            }

            if (signedLeft == long.MinValue && signedRight == -1)
            {
                Lo = unchecked((ulong)long.MinValue);
                Hi = 0;
                return;
            }

            Lo = (ulong)(signedLeft / signedRight);
            Hi = (ulong)(signedLeft % signedRight);
        }
    }
}
