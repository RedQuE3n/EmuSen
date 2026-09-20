using System;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The vector unit on coprocessor two: its registers, its flags and how an instruction reaches them - see Mars_RspVector.md.
    public sealed partial class Rsp
    {
        public const int VectorRegisters = 32;
        public const int Elements = 8;

        // Thirty-two registers of eight 16-bit elements, flattened register first - see Mars_RspVector.md §1.
        public readonly ushort[] Vector = new ushort[VectorRegisters * Elements];

        // Forty-eight bits per element, kept wrapped - see Mars_RspVector.md §6.
        public readonly ulong[] Accumulator = new ulong[Elements];

        public ushort Vco;
        public ushort Vcc;
        public byte Vce;

        // Bit 25 of a coprocessor-two word is what separates a vector operation from a transfer.
        private const uint VectorOperation = 1u << 25;

        [System.Runtime.CompilerServices.MethodImpl(System.Runtime.CompilerServices.MethodImplOptions.AggressiveInlining)]
        private void ExecuteCop2(uint instruction)
        {
            if ((instruction & VectorOperation) != 0)
            {
                if (UseSimd) ExecuteVectorSimd(instruction);
                else ExecuteVector(instruction);
                return;
            }

            int register = Rd(instruction);
            int element = (int)((instruction >> 7) & 0xF);

            switch (Rs(instruction))
            {
                case 0x00: Write(Rt(instruction), (uint)(int)(short)ReadElementBytes(register, element)); return;
                case 0x02: Write(Rt(instruction), ReadControl(register)); return;
                case 0x04: WriteElementBytes(register, element, Read(Rt(instruction))); return;
                case 0x06: WriteControl(register, Read(Rt(instruction))); return;
                default: return;
            }
        }

        // Out of a register the second byte wraps to its start; into one it stops at the end - see Mars_RspVector.md §3.
        private ushort ReadElementBytes(int register, int index) =>
            (ushort)((VectorByte(register, index) << 8) | VectorByte(register, (index + 1) & 0xF));

        private void WriteElementBytes(int register, int index, uint value)
        {
            SetVectorByte(register, index, (byte)(value >> 8));
            if (index < 15) SetVectorByte(register, index + 1, (byte)value);
        }

        // Only the index's low two bits count, and the extension flags are the one register narrower than sixteen bits - see §3.
        private uint ReadControl(int index) => (index & 3) switch
        {
            0 => (uint)(int)(short)Vco,
            1 => (uint)(int)(short)Vcc,
            _ => Vce,
        };

        private void WriteControl(int index, uint value)
        {
            switch (index & 3)
            {
                case 0: Vco = (ushort)value; break;
                case 1: Vcc = (ushort)value; break;
                default: Vce = (byte)value; break;
            }
        }

        private void ExecuteVector(uint instruction)
        {
            WidenAccumulator();

            int vt = Rt(instruction);
            int vs = Rd(instruction);
            int vd = (int)((instruction >> 6) & 0x1F);
            int selector = (int)((instruction >> 21) & 0xF);

            // Every source element is copied out before anything is written, which is what aliasing needs - see §2.
            Span<ushort> s = stackalloc ushort[Elements];
            Span<ushort> t = stackalloc ushort[Elements];
            Span<ushort> d = stackalloc ushort[Elements];

            for (int i = 0; i < Elements; i++)
            {
                s[i] = Vector[vs * Elements + i];
                t[i] = Vector[vt * Elements + Selected(selector, i)];
                d[i] = Vector[vd * Elements + i];
            }

            switch (instruction & 0x3F)
            {
                case 0x00: Fraction(s, t, d, unsigned: false, accumulate: false); break;
                case 0x01: Fraction(s, t, d, unsigned: true, accumulate: false); break;
                case 0x02: Round(t, d, shifted: (vs & 1) != 0, positive: true); break;
                case 0x03: Quarter(s, t, d); break;
                case 0x04: Low(s, t, d, accumulate: false); break;
                case 0x05: Middle(s, t, d, accumulate: false); break;
                case 0x06: Normal(s, t, d, accumulate: false); break;
                case 0x07: High(s, t, d, accumulate: false); break;
                case 0x08: Fraction(s, t, d, unsigned: false, accumulate: true); break;
                case 0x09: Fraction(s, t, d, unsigned: true, accumulate: true); break;
                case 0x0A: Round(t, d, shifted: (vs & 1) != 0, positive: false); break;
                case 0x0B: AccumulatedQuarter(d); break;
                case 0x0C: Low(s, t, d, accumulate: true); break;
                case 0x0D: Middle(s, t, d, accumulate: true); break;
                case 0x0E: Normal(s, t, d, accumulate: true); break;
                case 0x0F: High(s, t, d, accumulate: true); break;

                case 0x10: Add(s, t, d, subtract: false); break;
                case 0x11: Add(s, t, d, subtract: true); break;
                case 0x13: Absolute(s, t, d); break;
                case 0x14: AddCarrying(s, t, d); break;
                case 0x15: SubtractCarrying(s, t, d); break;
                case 0x1D: ReadAccumulator(selector, d); break;

                case 0x20: Compare(s, t, d, VectorComparison.Less); break;
                case 0x21: Compare(s, t, d, VectorComparison.Equal); break;
                case 0x22: Compare(s, t, d, VectorComparison.NotEqual); break;
                case 0x23: Compare(s, t, d, VectorComparison.GreaterOrEqual); break;
                case 0x24: ClipLow(s, t, d); break;
                case 0x25: ClipHigh(s, t, d); break;
                case 0x26: ClipOnesComplement(s, t, d); break;
                case 0x27: Merge(s, t, d); break;

                case 0x28: Logic(s, t, d, (a, b) => a & b); break;
                case 0x29: Logic(s, t, d, (a, b) => ~(a & b)); break;
                case 0x2A: Logic(s, t, d, (a, b) => a | b); break;
                case 0x2B: Logic(s, t, d, (a, b) => ~(a | b)); break;
                case 0x2C: Logic(s, t, d, (a, b) => a ^ b); break;
                case 0x2D: Logic(s, t, d, (a, b) => ~(a ^ b)); break;

                case 0x30: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Single, root: false); break;
                case 0x31: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Low, root: false); break;
                case 0x32: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.High, root: false); break;
                case 0x33: Move(vs & 7, t, d); break;
                case 0x34: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Single, root: true); break;
                case 0x35: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.Low, root: true); break;
                case 0x36: Reciprocate(Vector[vt * Elements + (selector & 7)], vs & 7, t, d, VectorReciprocal.High, root: true); break;

                // The two documented no-operations touch nothing at all - see Mars_RspVector.md §11.
                case 0x37:
                case 0x3F: return;

                // Every other function, documented or not, zeroes its destination and sums into the accumulator - see §11.
                default: SumIntoAccumulator(s, t, d); break;
            }

            d.CopyTo(Vector.AsSpan(vd * Elements, Elements));
        }

        // Which element of vt pairs with element i: all of them, quarters, halves, or one broadcast - see §2.
        private static int Selected(int selector, int i) => selector switch
        {
            0 or 1 => i,
            2 or 3 => selector - 2 + (i & 6),
            >= 4 and <= 7 => selector - 4 + (i & 4),
            _ => selector - 8,
        };

        private byte VectorByte(int register, int index)
        {
            ushort element = Vector[register * Elements + (index >> 1)];
            return (byte)((index & 1) == 0 ? element >> 8 : element);
        }

        private void SetVectorByte(int register, int index, byte value)
        {
            ref ushort element = ref Vector[register * Elements + (index >> 1)];
            element = (index & 1) == 0 ? (ushort)((element & 0x00FF) | (value << 8)) : (ushort)((element & 0xFF00) | value);
        }
    }
}
