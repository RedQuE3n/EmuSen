namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.Sa1
{
    // SA-1 arithmetic unit ($2250-$2254 in, $2306-$230B out): signed multiply,
    // signed/unsigned divide, and a 40-bit multiply-accumulate - see Venus_SA1.md §6.
    public sealed class Sa1Math
    {
        private const int ModeMultiply = 0;
        private const int ModeDivide = 1;

        private byte _control;
        private ushort _a, _b;

        // 40 bits wide: the accumulate mode is the only thing that needs more than 32.
        public long Result;
        public bool Overflow;

        private int Mode => _control & 0x01;
        private bool Accumulate => (_control & 0x02) != 0;

        // Writing the control register also clears the accumulator - see Venus_SA1.md §6.
        public void WriteControl(byte value)
        {
            _control = value;
            Result = 0;
            Overflow = false;
        }

        public void WriteALow(byte value) => _a = (ushort)((_a & 0xFF00) | value);
        public void WriteAHigh(byte value) => _a = (ushort)((_a & 0x00FF) | (value << 8));
        public void WriteBLow(byte value) => _b = (ushort)((_b & 0xFF00) | value);

        // Writing the high byte of the multiplier/divisor is what runs the operation.
        public void WriteBHighTrigger(byte value)
        {
            _b = (ushort)((_b & 0x00FF) | (value << 8));
            Execute();

            // Hardware clears the operand after a plain multiply or divide, but
            // keeps it for accumulate so a running sum can reuse it.
            if (!Accumulate || Mode == ModeDivide) _b = 0;
        }

        private void Execute()
        {
            if (Mode == ModeDivide)
            {
                ExecuteDivide();
                return;
            }

            // Both operands are signed for a multiply; only the divisor below is unsigned.
            long product = (short)_a * (long)(short)_b;
            if (Accumulate)
            {
                // Kept as a full signed long so a running sum that dips negative
                // keeps accumulating correctly; the register reads only ever
                // expose the low 40 bits of it either way.
                Result += product;
                Overflow = Result > 0x7FFFFFFFFFL || Result < -0x8000000000L;
                return;
            }

            Result = (uint)product;
            Overflow = false;
        }

        // Signed dividend, unsigned divisor; quotient lands in the low word and
        // the remainder in the high word - see Venus_SA1.md §6.1.
        private void ExecuteDivide()
        {
            short dividend = (short)_a;
            ushort divisor = _b;

            int quotient, remainder;
            if (divisor == 0)
            {
                quotient = 0;
                remainder = dividend;
            }
            else
            {
                quotient = dividend / divisor;
                remainder = dividend % divisor;

                // Hardware floors toward negative infinity rather than truncating,
                // so a negative dividend keeps the remainder non-negative.
                if (remainder < 0)
                {
                    quotient--;
                    remainder += divisor;
                }
            }

            Result = ((uint)(ushort)remainder << 16) | (ushort)quotient;
            Overflow = false;
        }
    }
}
