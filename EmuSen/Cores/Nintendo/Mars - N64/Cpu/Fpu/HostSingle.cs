using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Fpu
{
    // Single-precision add, subtract and multiply on the host, only where its answer is provably the software unit's - see Mars_FpuMath.md §11.
    public static class HostSingle
    {
        // Off only to measure what it saves, or to show it changes nothing - see §11.
        public static bool Enabled = Environment.GetEnvironmentVariable("EMUSEN_MARS_NOHOSTFLOAT") != "1";

        private const double SmallestNormal = 1.17549435082228750797e-38;

        // False leaves the operation to the software unit: an operand that is not a normal number, a sum the double cannot hold exactly, or a result at or past either end of the normal range - see §11.
        public static bool TryCompute(uint function, uint left, uint right, out ulong bits, out uint flags)
        {
            bits = 0;
            flags = 0;

            uint leftExponent = (left >> 23) & 0xFF, rightExponent = (right >> 23) & 0xFF;
            if (leftExponent - 1 >= 254 || rightExponent - 1 >= 254) return false;

            double a = BitConverter.UInt32BitsToSingle(left), b = BitConverter.UInt32BitsToSingle(right), exact;

            if (function == 2)
            {
                // Twenty-four bits by twenty-four is forty-eight, which a double holds whole.
                exact = a * b;
            }
            else
            {
                // A sum is whole in a double while the exponents are within its spare twenty-nine bits.
                if (Math.Abs((int)leftExponent - (int)rightExponent) > 28) return false;
                exact = function == 0 ? a + b : a - b;
            }

            // Tininess and overflow are the software unit's, with their flags and their refusals; a zero sum has a sign rule of its own.
            double size = Math.Abs(exact);
            if (!(size >= SmallestNormal)) return false;

            float rounded = (float)exact;
            if (float.IsInfinity(rounded)) return false;

            bits = BitConverter.SingleToUInt32Bits(rounded);
            flags = (double)rounded != exact ? SoftFloatMath.Inexact : 0;
            return true;
        }

        // The software unit's answer to the same question, for the test that compares them - see §11.
        public static (ulong Bits, uint Flags, bool Unimplemented) Reference(uint function, uint left, uint right)
        {
            FloatFormat format = FloatFormat.Single;
            FloatResult result = function switch
            {
                0 => SoftFloatMath.Add(left, right, format, SoftFloatMath.RoundNearest, false),
                1 => SoftFloatMath.Subtract(left, right, format, SoftFloatMath.RoundNearest, false),
                _ => SoftFloatMath.Multiply(left, right, format, SoftFloatMath.RoundNearest, false),
            };
            return (result.Bits, result.Flags, result.Unimplemented);
        }
    }
}
