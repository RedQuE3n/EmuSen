using System;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The vector unit's arithmetic, element by element - see Mars_RspVector.md §6 to §11.
    public sealed partial class Rsp
    {
        private enum VectorComparison { Less, Equal, NotEqual, GreaterOrEqual }

        private enum VectorReciprocal { Single, Low, High }

        private const ulong AccumulatorMask = 0xFFFF_FFFF_FFFF;

        // What a high half leaves for the next low half, whether it is waiting, and the last upper word - see §10.
        private ushort _divideInput;
        private bool _divideInputLoaded;
        private ushort _divideOutput;

        // The corpus's rounding constant is added by the plain form only, not by the accumulating one - see §7.
        private void Fraction(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool unsigned, bool accumulate)
        {
            for (int i = 0; i < Elements; i++)
            {
                long product = (long)(short)s[i] * (short)t[i];
                long value = Accumulate(i, (accumulate ? Signed48(Accumulator[i]) : 0x8000) + (product << 1));

                d[i] = unsigned ? ClampUnsigned(value) : ClampSigned(value >> 16);
            }
        }

        private void Low(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool accumulate)
        {
            for (int i = 0; i < Elements; i++)
            {
                uint product = (uint)s[i] * t[i];
                d[i] = ClampLow(Accumulate(i, (accumulate ? Signed48(Accumulator[i]) : 0) + (product >> 16)));
            }
        }

        private void Middle(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool accumulate)
        {
            for (int i = 0; i < Elements; i++)
            {
                long product = (long)(short)s[i] * t[i];
                d[i] = ClampSigned(Accumulate(i, (accumulate ? Signed48(Accumulator[i]) : 0) + product) >> 16);
            }
        }

        private void Normal(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool accumulate)
        {
            for (int i = 0; i < Elements; i++)
            {
                long product = (long)s[i] * (short)t[i];
                d[i] = ClampLow(Accumulate(i, (accumulate ? Signed48(Accumulator[i]) : 0) + product));
            }
        }

        private void High(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool accumulate)
        {
            for (int i = 0; i < Elements; i++)
            {
                long product = (long)(short)s[i] * (short)t[i];
                d[i] = ClampSigned(Accumulate(i, (accumulate ? Signed48(Accumulator[i]) : 0) + (product << 16)) >> 16);
            }
        }

        // A negative product is biased by thirty-one before it is kept, and the result loses its low four bits - see §7.
        private void Quarter(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++)
            {
                long product = (long)(short)s[i] * (short)t[i];
                long value = Accumulate(i, (product << 16) + (product < 0 ? 0x1F_0000 : 0));

                d[i] = (ushort)(ClampSigned(value >> 17) & 0xFFF0);
            }
        }

        // The accumulating quarter multiply ignores both sources and only nudges the accumulator toward zero - see §9.
        private void AccumulatedQuarter(Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++)
            {
                long value = Signed48(Accumulator[i]);

                if ((value & 0x20_0000) == 0)
                {
                    long upper = value >> 22;
                    if (upper < 0) value += 0x20_0000;
                    else if (upper > 0) value -= 0x20_0000;
                }

                d[i] = (ushort)(ClampSigned(Accumulate(i, value) >> 17) & 0xFFF0);
            }
        }

        // The parity of vs's register number, not anything in it, decides whether vt is shifted - see §9.
        private void Round(ReadOnlySpan<ushort> t, Span<ushort> d, bool shifted, bool positive)
        {
            for (int i = 0; i < Elements; i++)
            {
                long value = Signed48(Accumulator[i]);
                if ((value >= 0) == positive) value += (long)(short)t[i] << (shifted ? 16 : 0);

                d[i] = ClampSigned(Accumulate(i, value) >> 16);
            }
        }

        // Carry in from the flags, clamp what is written, keep what was not clamped - see §8.
        private void Add(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, bool subtract)
        {
            for (int i = 0; i < Elements; i++)
            {
                int carry = (Vco >> i) & 1;
                int sum = subtract ? (short)s[i] - (short)t[i] - carry : (short)s[i] + (short)t[i] + carry;

                d[i] = ClampSigned(sum);
                SetAccumulatorLow(i, (ushort)sum);
            }

            Vco = 0;
        }

        // The one negation that overflows keeps its overflow in the accumulator and clamps in the destination - see §8.
        private void Absolute(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++)
            {
                short sign = (short)s[i];
                ushort value = sign < 0 ? (ushort)-t[i] : sign == 0 ? (ushort)0 : t[i];

                d[i] = sign < 0 && t[i] == 0x8000 ? (ushort)0x7FFF : value;
                SetAccumulatorLow(i, value);
            }
        }

        private void AddCarrying(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            int carries = 0;

            for (int i = 0; i < Elements; i++)
            {
                int sum = s[i] + t[i];

                d[i] = (ushort)sum;
                SetAccumulatorLow(i, (ushort)sum);
                if (sum > 0xFFFF) carries |= 1 << i;
            }

            Vco = (ushort)carries;
        }

        // The borrow goes in the low byte and inequality in the high one - see §8.
        private void SubtractCarrying(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            int flags = 0;

            for (int i = 0; i < Elements; i++)
            {
                int difference = s[i] - t[i];

                d[i] = (ushort)difference;
                SetAccumulatorLow(i, (ushort)difference);
                if (difference != 0) flags |= 0x100 << i;
                if (difference < 0) flags |= 1 << i;
            }

            Vco = (ushort)flags;
        }

        // Selectors eight, nine and ten read the high, middle and low thirds, and every other one reads zero - see §6.
        private void ReadAccumulator(int selector, Span<ushort> d)
        {
            int shift = selector switch { 8 => 32, 9 => 16, 10 => 0, _ => -1 };
            for (int i = 0; i < Elements; i++) d[i] = shift < 0 ? (ushort)0 : (ushort)(Accumulator[i] >> shift);
        }

        // An equal pair is settled by whatever carry and not-equal flags VCO already holds - see §8.
        private void Compare(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, VectorComparison comparison)
        {
            int flags = 0;

            for (int i = 0; i < Elements; i++)
            {
                bool equal = s[i] == t[i];
                bool carry = ((Vco >> i) & 1) != 0;
                bool notEqual = ((Vco >> (8 + i)) & 1) != 0;

                bool chosen = comparison switch
                {
                    VectorComparison.Less => (short)s[i] < (short)t[i] || (equal && carry && notEqual),
                    VectorComparison.Equal => equal && !notEqual,
                    VectorComparison.NotEqual => !equal || notEqual,
                    _ => (short)s[i] > (short)t[i] || (equal && !(carry && notEqual)),
                };

                d[i] = comparison switch
                {
                    VectorComparison.Equal => t[i],
                    VectorComparison.NotEqual => s[i],
                    _ => chosen ? s[i] : t[i],
                };

                SetAccumulatorLow(i, d[i]);
                if (chosen) flags |= 1 << i;
            }

            Vcc = (ushort)flags;
            Vco = 0;
        }

        // The low clip reads the flags a high clip left, and where not-equal is set the old comparison stands - see §8.
        private void ClipLow(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            int flags = Vcc;

            for (int i = 0; i < Elements; i++)
            {
                bool signsDiffered = ((Vco >> i) & 1) != 0;
                bool notEqual = ((Vco >> (8 + i)) & 1) != 0;
                bool extension = ((Vce >> i) & 1) != 0;
                bool lessOrEqual = ((flags >> i) & 1) != 0;
                bool greaterOrEqual = ((flags >> (8 + i)) & 1) != 0;

                if (signsDiffered)
                {
                    int sum = s[i] + t[i];
                    bool zero = (sum & 0xFFFF) == 0;
                    bool carried = sum > 0xFFFF;

                    if (!notEqual) lessOrEqual = (zero && !carried) || (extension && (zero || !carried));
                    d[i] = lessOrEqual ? (ushort)-t[i] : s[i];
                }
                else
                {
                    if (!notEqual) greaterOrEqual = s[i] >= t[i];
                    d[i] = greaterOrEqual ? t[i] : s[i];
                }

                SetAccumulatorLow(i, d[i]);
                flags = (flags & ~(0x101 << i)) | (lessOrEqual ? 1 << i : 0) | (greaterOrEqual ? 0x100 << i : 0);
            }

            Vcc = (ushort)flags;
            Vco = 0;
            Vce = 0;
        }

        private void ClipHigh(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            int carries = 0;
            int compares = 0;
            int extensions = 0;

            for (int i = 0; i < Elements; i++)
            {
                short a = (short)t[i];
                short b = (short)s[i];
                bool signsDiffer = (a ^ b) < 0;
                bool lessOrEqual, greaterOrEqual, extension, notEqual;

                if (signsDiffer)
                {
                    int sum = a + b;

                    greaterOrEqual = a < 0;
                    lessOrEqual = sum <= 0;
                    extension = sum == -1;
                    notEqual = sum != 0 && sum != -1;
                    d[i] = lessOrEqual ? (ushort)-a : (ushort)b;
                }
                else
                {
                    int difference = b - a;

                    lessOrEqual = a < 0;
                    greaterOrEqual = difference >= 0;
                    extension = false;
                    notEqual = difference != 0;
                    d[i] = greaterOrEqual ? (ushort)a : (ushort)b;
                }

                SetAccumulatorLow(i, d[i]);
                carries |= (signsDiffer ? 1 << i : 0) | (notEqual ? 0x100 << i : 0);
                compares |= (lessOrEqual ? 1 << i : 0) | (greaterOrEqual ? 0x100 << i : 0);
                extensions |= extension ? 1 << i : 0;
            }

            Vco = (ushort)carries;
            Vcc = (ushort)compares;
            Vce = (byte)extensions;
        }

        // The one's-complement clip negates by inverting, and leaves no carry or extension flags behind - see §8.
        private void ClipOnesComplement(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            int compares = 0;

            for (int i = 0; i < Elements; i++)
            {
                short a = (short)t[i];
                short b = (short)s[i];
                bool lessOrEqual, greaterOrEqual;

                if ((a ^ b) < 0)
                {
                    greaterOrEqual = a < 0;
                    lessOrEqual = a + b < 0;
                    d[i] = lessOrEqual ? (ushort)~a : (ushort)b;
                }
                else
                {
                    lessOrEqual = a < 0;
                    greaterOrEqual = b - a >= 0;
                    d[i] = greaterOrEqual ? (ushort)a : (ushort)b;
                }

                SetAccumulatorLow(i, d[i]);
                compares |= (lessOrEqual ? 1 << i : 0) | (greaterOrEqual ? 0x100 << i : 0);
            }

            Vco = 0;
            Vcc = (ushort)compares;
            Vce = 0;
        }

        private void Merge(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++)
            {
                d[i] = ((Vcc >> i) & 1) != 0 ? s[i] : t[i];
                SetAccumulatorLow(i, d[i]);
            }

            Vco = 0;
        }

        private void Logic(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d, Func<int, int, int> operation)
        {
            for (int i = 0; i < Elements; i++)
            {
                d[i] = (ushort)operation(t[i], s[i]);
                SetAccumulatorLow(i, d[i]);
            }
        }

        // One element moves, chosen by vs's register number, and the whole selected vt still reaches the accumulator - see §10.
        private void Move(int element, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++) SetAccumulatorLow(i, t[i]);
            d[element] = t[element];
        }

        private void Reciprocate(ushort input, int element, ReadOnlySpan<ushort> t, Span<ushort> d, VectorReciprocal kind, bool root)
        {
            for (int i = 0; i < Elements; i++) SetAccumulatorLow(i, t[i]);

            if (kind == VectorReciprocal.High)
            {
                d[element] = _divideOutput;
                _divideInput = input;
                _divideInputLoaded = true;
                return;
            }

            uint operand = kind == VectorReciprocal.Low && _divideInputLoaded
                ? ((uint)_divideInput << 16) | input
                : (uint)(int)(short)input;
            uint result = root ? Reciprocals.RootOf(operand) : Reciprocals.Of(operand);

            d[element] = (ushort)result;
            _divideOutput = (ushort)(result >> 16);
            _divideInputLoaded = false;
        }

        private void SumIntoAccumulator(ReadOnlySpan<ushort> s, ReadOnlySpan<ushort> t, Span<ushort> d)
        {
            for (int i = 0; i < Elements; i++)
            {
                SetAccumulatorLow(i, (ushort)(s[i] + t[i]));
                d[i] = 0;
            }
        }

        private static long Signed48(ulong value) => (long)(value << 16) >> 16;

        // The accumulator wraps first and every clamp reads what it wrapped to - see §6.1.
        private long Accumulate(int element, long value)
        {
            Accumulator[element] = (ulong)value & AccumulatorMask;
            return Signed48(Accumulator[element]);
        }

        private void SetAccumulatorLow(int element, ushort value) =>
            Accumulator[element] = (Accumulator[element] & ~0xFFFFUL) | value;

        private static ushort ClampSigned(long value) =>
            value < short.MinValue ? (ushort)0x8000 : value > short.MaxValue ? (ushort)0x7FFF : (ushort)value;

        private static ushort ClampUnsigned(long value) =>
            value < 0 ? (ushort)0 : value > 0x7FFF_FFFF ? (ushort)0xFFFF : (ushort)(value >> 16);

        // A low word is kept only while the word above it fits in sixteen signed bits - see §7.
        private static ushort ClampLow(long value) =>
            (value >> 16) < short.MinValue ? (ushort)0 : (value >> 16) > short.MaxValue ? (ushort)0xFFFF : (ushort)value;
    }
}
