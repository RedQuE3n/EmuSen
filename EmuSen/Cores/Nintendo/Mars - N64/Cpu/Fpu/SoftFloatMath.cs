using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Fpu
{
    // What an operation produced, and what it wants recorded - see Mars_FpuMath.md §5.
    internal struct FloatResult
    {
        public ulong Bits;
        public uint Flags;

        // The unmaskable one: the part declined the operation rather than computing it - see §3.
        public bool Unimplemented;

        public static FloatResult Refused() => new() { Unimplemented = true };

        public static FloatResult Exact(ulong bits) => new() { Bits = bits };

        public static FloatResult Raised(ulong bits, uint flags) => new() { Bits = bits, Flags = flags };
    }

    // Arithmetic on numbers this part will actually accept, rounded exactly once - see Mars_FpuMath.md.
    internal static class SoftFloatMath
    {
        public const uint Inexact = 1 << 0;
        public const uint Underflow = 1 << 1;
        public const uint Overflow = 1 << 2;
        public const uint DivideByZero = 1 << 3;
        public const uint Invalid = 1 << 4;

        public const uint RoundNearest = 0;
        public const uint RoundZero = 1;
        public const uint RoundPositive = 2;
        public const uint RoundNegative = 3;

        // Where the leading one sits while an operation is in progress - see Mars_FpuMath.md §4.
        private const int WorkingBit = 127;

        public static FloatResult Add(ulong left, ulong right, in FloatFormat format, uint mode, bool flush) =>
            AddSigned(left, right, format, mode, flush, negateRight: false);

        public static FloatResult Subtract(ulong left, ulong right, in FloatFormat format, uint mode, bool flush) =>
            AddSigned(left, right, format, mode, flush, negateRight: true);

        public static FloatResult Multiply(ulong left, ulong right, in FloatFormat format, uint mode, bool flush)
        {
            var a = SoftFloat.Unpack(left, format);
            var b = SoftFloat.Unpack(right, format);

            if (Refuses(a, b)) return FloatResult.Refused();
            if (a.IsNan || b.IsNan) return Invalidated(format);

            bool sign = a.Sign ^ b.Sign;

            bool zero = a.Class == FloatClass.Zero || b.Class == FloatClass.Zero;
            bool infinite = a.Class == FloatClass.Infinity || b.Class == FloatClass.Infinity;

            if (zero && infinite) return Invalidated(format);
            if (infinite) return FloatResult.Exact(SoftFloat.Infinite(sign, format));
            if (zero) return FloatResult.Exact(SoftFloat.Zero(sign, format));

            UInt128 product = (UInt128)a.Significand * b.Significand;
            int exponent = a.Exponent + b.Exponent;

            if ((product >> WorkingBit) != 0) exponent += 1;
            else product <<= 1;

            return Round(sign, exponent, product, sticky: false, format, mode, flush);
        }

        public static FloatResult Divide(ulong left, ulong right, in FloatFormat format, uint mode, bool flush)
        {
            var a = SoftFloat.Unpack(left, format);
            var b = SoftFloat.Unpack(right, format);

            if (Refuses(a, b)) return FloatResult.Refused();
            if (a.IsNan || b.IsNan) return Invalidated(format);

            bool sign = a.Sign ^ b.Sign;

            if (a.Class == FloatClass.Infinity)
            {
                return b.Class == FloatClass.Infinity
                    ? Invalidated(format)
                    : FloatResult.Exact(SoftFloat.Infinite(sign, format));
            }

            if (b.Class == FloatClass.Infinity) return FloatResult.Exact(SoftFloat.Zero(sign, format));

            if (b.Class == FloatClass.Zero)
            {
                return a.Class == FloatClass.Zero
                    ? Invalidated(format)
                    : FloatResult.Raised(SoftFloat.Infinite(sign, format), DivideByZero);
            }

            if (a.Class == FloatClass.Zero) return FloatResult.Exact(SoftFloat.Zero(sign, format));

            UInt128 numerator = (UInt128)a.Significand << 64;
            UInt128 quotient = numerator / b.Significand;
            bool sticky = numerator - (quotient * b.Significand) != 0;

            int leading = HighestBit(quotient);
            int shift = WorkingBit - leading;

            return Round(sign, a.Exponent - b.Exponent - 64 - shift + WorkingBit,
                quotient << shift, sticky, format, mode, flush);
        }

        // Not bit operations: they classify the operand first, and refuse the same two shapes - see §5.1.
        public static FloatResult Sign(ulong bits, in FloatFormat format, bool negate)
        {
            var value = SoftFloat.Unpack(bits, format);

            if (Refuses(value)) return FloatResult.Refused();
            if (value.IsNan) return Invalidated(format);

            return FloatResult.Exact(negate ? bits ^ format.SignBit : bits & ~format.SignBit);
        }

        public static FloatResult SquareRoot(ulong bits, in FloatFormat format, uint mode, bool flush)
        {
            var value = SoftFloat.Unpack(bits, format);

            if (Refuses(value)) return FloatResult.Refused();
            if (value.IsNan) return Invalidated(format);
            if (value.Class == FloatClass.Zero) return FloatResult.Exact(SoftFloat.Zero(value.Sign, format));
            if (value.Sign) return Invalidated(format);
            if (value.Class == FloatClass.Infinity) return FloatResult.Exact(SoftFloat.Infinite(false, format));

            int power = value.Exponent - 63;
            UInt128 radicand = value.Significand;

            // The exponent has to be even before the root can be taken as an integer one - see §8.
            if ((power & 1) != 0)
            {
                radicand <<= 1;
                power -= 1;
            }

            radicand <<= 62;

            UInt128 root = IntegerSquareRoot(radicand, out bool exact);
            int leading = HighestBit(root);
            int shift = WorkingBit - leading;

            return Round(false, (power / 2) - 31 - shift + WorkingBit,
                root << shift, !exact, format, mode, flush);
        }

        // Digit by digit, so the remainder says exactly whether the root was exact - see §8.
        private static UInt128 IntegerSquareRoot(UInt128 value, out bool exact)
        {
            UInt128 root = UInt128.Zero;
            UInt128 remainder = UInt128.Zero;

            for (int shift = 126; shift >= 0; shift -= 2)
            {
                remainder = (remainder << 2) | ((value >> shift) & 3);

                // Four times the root so far, not twice: the digit contributes d(4R + d) - see §8.
                UInt128 candidate = (root << 2) | UInt128.One;
                root <<= 1;

                if (remainder >= candidate)
                {
                    remainder -= candidate;
                    root |= UInt128.One;
                }
            }

            exact = remainder == UInt128.Zero;
            return root;
        }

        private static FloatResult AddSigned(
            ulong left, ulong right, in FloatFormat format, uint mode, bool flush, bool negateRight)
        {
            var a = SoftFloat.Unpack(left, format);
            var b = SoftFloat.Unpack(right, format);

            if (Refuses(a, b)) return FloatResult.Refused();
            if (a.IsNan || b.IsNan) return Invalidated(format);

            if (negateRight) b.Sign = !b.Sign;

            if (a.Class == FloatClass.Infinity || b.Class == FloatClass.Infinity)
            {
                if (a.Class == FloatClass.Infinity && b.Class == FloatClass.Infinity && a.Sign != b.Sign)
                {
                    return Invalidated(format);
                }

                bool sign = a.Class == FloatClass.Infinity ? a.Sign : b.Sign;
                return FloatResult.Exact(SoftFloat.Infinite(sign, format));
            }

            // Two zeros keep a sign only when they agree, except rounding down, where the sum sinks - see §4.2.
            if (a.Class == FloatClass.Zero && b.Class == FloatClass.Zero)
            {
                bool sign = a.Sign == b.Sign ? a.Sign : mode == RoundNegative;
                return FloatResult.Exact(SoftFloat.Zero(sign, format));
            }

            if (a.Class == FloatClass.Zero) return FloatResult.Exact(Repack(b, format));
            if (b.Class == FloatClass.Zero) return FloatResult.Exact(Repack(a, format));

            if (b.Exponent > a.Exponent || (b.Exponent == a.Exponent && b.Significand > a.Significand))
            {
                (a, b) = (b, a);
            }

            int distance = a.Exponent - b.Exponent;

            UInt128 wide = (UInt128)a.Significand << 63;
            UInt128 addend = (UInt128)b.Significand << 63;
            bool sticky = false;

            if (distance >= 128)
            {
                sticky = true;
                addend = UInt128.Zero;
            }
            else if (distance > 0)
            {
                sticky = (addend & ((UInt128.One << distance) - 1)) != 0;
                addend >>= distance;
            }

            UInt128 sum;

            if (a.Sign == b.Sign)
            {
                sum = wide + addend;
            }
            else
            {
                sum = wide - addend;

                // The discarded tail made the subtrahend larger, so the true result sits just below.
                if (sticky) sum -= 1;

                if (sum == UInt128.Zero)
                {
                    return FloatResult.Exact(SoftFloat.Zero(mode == RoundNegative, format));
                }
            }

            int leading = HighestBit(sum);

            return Round(a.Sign, a.Exponent + 1 - (WorkingBit - leading),
                sum << (WorkingBit - leading), sticky, format, mode, flush);
        }

        // Two operand shapes are refused outright rather than computed with - see Mars_FpuMath.md §3.
        private static bool Refuses(in SoftFloat a, in SoftFloat b) => Refuses(a) || Refuses(b);

        private static bool Refuses(in SoftFloat value) =>
            value.Class is FloatClass.Subnormal or FloatClass.NanUnsupported;

        private static FloatResult Invalidated(in FloatFormat format) =>
            FloatResult.Raised(format.DefaultNan, Invalid);

        private static ulong Repack(in SoftFloat value, in FloatFormat format)
        {
            ulong mantissa = (value.Significand >> (63 - format.MantissaBits)) & format.MantissaMask;

            return (value.Sign ? format.SignBit : 0)
                | ((ulong)(value.Exponent + format.Bias) << format.MantissaBits)
                | mantissa;
        }

        // Conversions arrive here with a significand already normalised, and round by the same rules.
        public static FloatResult RoundInto(
            bool sign, int exponent, UInt128 significand, bool sticky, in FloatFormat format, uint mode, bool flush) =>
            Round(sign, exponent, significand, sticky, format, mode, flush);

        // The single place a rounding mode is consulted, and the single place a range is checked - see §4.
        private static FloatResult Round(
            bool sign, int exponent, UInt128 significand, bool sticky, in FloatFormat format, uint mode, bool flush)
        {
            int keep = format.MantissaBits + 1;
            int shift = 128 - keep;

            UInt128 head = significand >> shift;
            UInt128 tail = significand - (head << shift);
            UInt128 half = UInt128.One << (shift - 1);

            bool roundBit = tail >= half;
            bool below = (tail & (half - 1)) != 0 || sticky;
            bool inexact = roundBit || below;

            bool increment = mode switch
            {
                RoundNearest => roundBit && (below || (head & 1) != 0),
                RoundZero => false,
                RoundPositive => inexact && !sign,
                _ => inexact && sign,
            };

            if (increment) head += 1;

            if ((head >> keep) != 0)
            {
                head >>= 1;
                exponent += 1;
            }

            int biased = exponent + format.Bias;

            if (biased >= format.MaxBiasedExponent) return Overflowed(sign, format, mode);

            // Below the smallest normal this part has no answer of its own unless told to flush - see §4.1.
            if (biased <= 0)
            {
                if (!flush) return FloatResult.Refused();

                // The directed modes are taken literally, so a tiny value can land on the smallest normal.
                bool away = mode == RoundPositive ? !sign : mode == RoundNegative && sign;

                ulong flushed = away ? SoftFloat.Smallest(sign, format) : SoftFloat.Zero(sign, format);

                return FloatResult.Raised(flushed, Underflow | Inexact);
            }

            ulong bits = (sign ? format.SignBit : 0)
                | ((ulong)biased << format.MantissaBits)
                | ((ulong)head & format.MantissaMask);

            return FloatResult.Raised(bits, inexact ? Inexact : 0);
        }

        // Which way an overflow lands is the rounding mode's decision, not always infinity - see §4.1.
        private static FloatResult Overflowed(bool sign, in FloatFormat format, uint mode)
        {
            bool infinite = mode switch
            {
                RoundNearest => true,
                RoundZero => false,
                RoundPositive => !sign,
                _ => sign,
            };

            ulong bits = infinite ? SoftFloat.Infinite(sign, format) : SoftFloat.Largest(sign, format);

            return FloatResult.Raised(bits, Overflow | Inexact);
        }

        private static int HighestBit(UInt128 value)
        {
            ulong high = (ulong)(value >> 64);

            return high != 0
                ? 64 + (63 - System.Numerics.BitOperations.LeadingZeroCount(high))
                : 63 - System.Numerics.BitOperations.LeadingZeroCount((ulong)value);
        }
    }
}
