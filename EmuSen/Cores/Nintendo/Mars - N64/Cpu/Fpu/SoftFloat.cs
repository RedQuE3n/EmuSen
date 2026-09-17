using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Fpu
{
    // What a bit pattern turns out to be, named for what this part does with it - see Mars_FpuMath.md §3.
    internal enum FloatClass
    {
        Zero,
        Subnormal,
        Normal,
        Infinity,
        Nan,
        NanUnsupported,
    }

    // The two formats, as the handful of numbers every operation needs - see Mars_FpuMath.md §2.
    internal readonly struct FloatFormat
    {
        public static readonly FloatFormat Single = new(23, 8, 127, 0x7FBF_FFFF);
        public static readonly FloatFormat Double = new(52, 11, 1023, 0x7FF7_FFFF_FFFF_FFFF);

        public readonly int MantissaBits;
        public readonly int ExponentBits;
        public readonly int Bias;

        // What this part answers with when an operation has no numeric result - see Mars_FpuMath.md §3.1.
        public readonly ulong DefaultNan;

        private FloatFormat(int mantissaBits, int exponentBits, int bias, ulong defaultNan)
        {
            MantissaBits = mantissaBits;
            ExponentBits = exponentBits;
            Bias = bias;
            DefaultNan = defaultNan;
        }

        public int MaxBiasedExponent => (1 << ExponentBits) - 1;

        public ulong MantissaMask => (1UL << MantissaBits) - 1;

        // The bit that decides which of the two NaN behaviours a pattern gets - see Mars_FpuMath.md §3.
        public ulong NanSelectBit => 1UL << (MantissaBits - 1);

        public ulong SignBit => 1UL << (MantissaBits + ExponentBits);

        public ulong LargestFinite => ((ulong)(MaxBiasedExponent - 1) << MantissaBits) | MantissaMask;

        public ulong Infinity => (ulong)MaxBiasedExponent << MantissaBits;
    }

    // A number pulled apart into the pieces arithmetic and rounding need - see Mars_FpuMath.md §2.
    internal struct SoftFloat
    {
        public bool Sign;

        // Unbiased, and always the exponent of the leading one, which sits at bit 63 of Significand.
        public int Exponent;

        public ulong Significand;

        public FloatClass Class;

        public bool IsNan => Class is FloatClass.Nan or FloatClass.NanUnsupported;

        public static SoftFloat Unpack(ulong bits, in FloatFormat format)
        {
            var value = default(SoftFloat);

            value.Sign = (bits & format.SignBit) != 0;

            int biased = (int)((bits >> format.MantissaBits) & (ulong)format.MaxBiasedExponent);
            ulong mantissa = bits & format.MantissaMask;

            if (biased == 0)
            {
                value.Class = mantissa == 0 ? FloatClass.Zero : FloatClass.Subnormal;

                // Arithmetic refuses these, but a compare reads them by value - see Mars_FpuMath.md §7.
                if (value.Class == FloatClass.Subnormal)
                {
                    int highest = 63 - System.Numerics.BitOperations.LeadingZeroCount(mantissa);

                    value.Exponent = 1 - format.Bias - format.MantissaBits + highest;
                    value.Significand = mantissa << (63 - highest);
                }

                return value;
            }

            if (biased == format.MaxBiasedExponent)
            {
                value.Class = mantissa == 0
                    ? FloatClass.Infinity
                    : (mantissa & format.NanSelectBit) != 0 ? FloatClass.Nan : FloatClass.NanUnsupported;
                return value;
            }

            value.Class = FloatClass.Normal;
            value.Exponent = biased - format.Bias;
            value.Significand = (mantissa | (1UL << format.MantissaBits)) << (63 - format.MantissaBits);

            return value;
        }

        // Ordering, with the two zeros equal and neither infinity a special case - see Mars_FpuMath.md §7.
        public static int Compare(in SoftFloat a, in SoftFloat b)
        {
            if (a.Class == FloatClass.Zero && b.Class == FloatClass.Zero) return 0;
            if (a.Sign != b.Sign) return a.Sign ? -1 : 1;

            int magnitude = CompareMagnitude(a, b);
            return a.Sign ? -magnitude : magnitude;
        }

        private static int CompareMagnitude(in SoftFloat a, in SoftFloat b)
        {
            if (a.Class == FloatClass.Infinity) return b.Class == FloatClass.Infinity ? 0 : 1;
            if (b.Class == FloatClass.Infinity) return -1;
            if (a.Class == FloatClass.Zero) return b.Class == FloatClass.Zero ? 0 : -1;
            if (b.Class == FloatClass.Zero) return 1;

            if (a.Exponent != b.Exponent) return a.Exponent < b.Exponent ? -1 : 1;
            if (a.Significand == b.Significand) return 0;

            return a.Significand < b.Significand ? -1 : 1;
        }

        public static ulong Zero(bool sign, in FloatFormat format) => sign ? format.SignBit : 0;

        public static ulong Infinite(bool sign, in FloatFormat format) =>
            format.Infinity | (sign ? format.SignBit : 0);

        public static ulong Smallest(bool sign, in FloatFormat format) =>
            (1UL << format.MantissaBits) | (sign ? format.SignBit : 0);

        public static ulong Largest(bool sign, in FloatFormat format) =>
            format.LargestFinite | (sign ? format.SignBit : 0);
    }
}
