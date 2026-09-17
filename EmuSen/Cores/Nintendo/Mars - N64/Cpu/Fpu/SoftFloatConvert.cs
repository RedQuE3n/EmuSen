using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Fpu
{
    // Between the two float formats, and between floats and integers - see Mars_FpuMath.md §6.
    internal static class SoftFloatConvert
    {
        private const long WidestSource = 1L << 55;

        // The 64-bit conversions stop at the double's mantissa, not at the long's range - see §6.3.
        private const int WidestWholeExponent = 53;

        public static FloatResult Between(
            ulong bits, in FloatFormat from, in FloatFormat to, uint mode, bool flush)
        {
            var value = SoftFloat.Unpack(bits, from);

            if (value.Class is FloatClass.Subnormal or FloatClass.NanUnsupported) return FloatResult.Refused();
            if (value.Class == FloatClass.Nan) return FloatResult.Raised(to.DefaultNan, SoftFloatMath.Invalid);
            if (value.Class == FloatClass.Infinity) return FloatResult.Exact(SoftFloat.Infinite(value.Sign, to));
            if (value.Class == FloatClass.Zero) return FloatResult.Exact(SoftFloat.Zero(value.Sign, to));

            return SoftFloatMath.RoundInto(value.Sign, value.Exponent, (UInt128)value.Significand << 64,
                sticky: false, to, mode, flush);
        }

        // Out of range, infinite or not a number, this family refuses rather than saturating - see §6.1.
        public static FloatResult ToInteger(
            ulong bits, in FloatFormat from, bool wide, uint mode)
        {
            var value = SoftFloat.Unpack(bits, from);

            if (value.Class is FloatClass.Subnormal or FloatClass.NanUnsupported) return FloatResult.Refused();
            if (value.Class is FloatClass.Nan or FloatClass.Infinity) return FloatResult.Refused();
            if (value.Class == FloatClass.Zero) return FloatResult.Exact(0);

            if (value.Exponent >= (wide ? WidestWholeExponent : 32)) return FloatResult.Refused();

            UInt128 whole;
            bool roundBit;
            bool below;

            int shift = 63 - value.Exponent;

            if (shift <= 0)
            {
                whole = (UInt128)value.Significand << -shift;
                roundBit = false;
                below = false;
            }
            else if (shift >= 128)
            {
                whole = UInt128.Zero;
                roundBit = false;
                below = true;
            }
            else
            {
                whole = (UInt128)value.Significand >> shift;
                UInt128 tail = (UInt128)value.Significand - (whole << shift);
                UInt128 half = UInt128.One << (shift - 1);

                roundBit = tail >= half;
                below = (tail & (half - 1)) != 0;
            }

            bool inexact = roundBit || below;

            bool increment = mode switch
            {
                SoftFloatMath.RoundNearest => roundBit && (below || (whole & 1) != 0),
                SoftFloatMath.RoundZero => false,
                SoftFloatMath.RoundPositive => inexact && !value.Sign,
                _ => inexact && value.Sign,
            };

            if (increment) whole += 1;

            UInt128 limit = value.Sign
                ? UInt128.One << (wide ? 63 : 31)
                : (UInt128.One << (wide ? 63 : 31)) - 1;

            if (whole > limit) return FloatResult.Refused();

            ulong magnitude = (ulong)whole;
            ulong result = value.Sign ? unchecked(0UL - magnitude) : magnitude;

            if (!wide) result &= 0xFFFF_FFFF;

            return FloatResult.Raised(result, inexact ? SoftFloatMath.Inexact : 0);
        }

        public static FloatResult FromInteger(ulong bits, bool wide, in FloatFormat to, uint mode, bool flush)
        {
            long value = wide ? (long)bits : (int)(uint)bits;

            // A 64-bit source outside this range is refused rather than rounded - see Mars_FpuMath.md §6.2.
            if (wide && (value >= WidestSource || value < -WidestSource)) return FloatResult.Refused();

            if (value == 0) return FloatResult.Exact(0);

            bool sign = value < 0;
            ulong magnitude = sign ? unchecked((ulong)-value) : (ulong)value;

            int leading = 63 - System.Numerics.BitOperations.LeadingZeroCount(magnitude);

            return SoftFloatMath.RoundInto(sign, leading, (UInt128)(magnitude << (63 - leading)) << 64,
                sticky: false, to, mode, flush);
        }
    }
}
