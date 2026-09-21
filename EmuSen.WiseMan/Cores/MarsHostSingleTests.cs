using System;
using EmuSen.Cores.Nintendo.Mars.Cpu.Fpu;

namespace EmuSen.WiseMan.Cores
{
    // Wherever the host answers a single-precision add, subtract or multiply, its bits and its flags are the software unit's - see Mars_FpuMath.md §11.
    public class MarsHostSingleTests
    {
        private static readonly uint[] Mantissas = { 0x000000, 0x000001, 0x7FFFFF, 0x7FFFFE, 0x400000, 0x3FFFFF, 0x400001, 0x555555, 0x2AAAAA };

        [Theory]
        [InlineData(0u)]
        [InlineData(1u)]
        [InlineData(2u)]
        public void The_hosts_answer_is_the_software_units_wherever_the_host_answers(uint function)
        {
            var seed = new Random(1234 + (int)function);
            long answered = 0, inexact = 0, declined = 0;
            string? first = null;

            void Try(uint left, uint right)
            {
                if (!HostSingle.TryCompute(function, left, right, out ulong bits, out uint flags)) { declined++; return; }

                answered++;
                if (flags != 0) inexact++;
                var (expectedBits, expectedFlags, unimplemented) = HostSingle.Reference(function, left, right);
                if (first is null && (unimplemented || bits != expectedBits || flags != expectedFlags))
                    first = $"function {function}: {left:X8} and {right:X8} gave {bits:X8} flags {flags:X}, the software unit {expectedBits:X8} flags {expectedFlags:X}{(unimplemented ? " and refused" : "")}";
            }

            uint Make(int sign, int exponent, uint mantissa) => ((uint)sign << 31) | ((uint)exponent << 23) | (mantissa & 0x7FFFFF);

            // Whatever bits come, most of them normal numbers far apart.
            for (int i = 0; i < 1_500_000; i++) Try((uint)seed.NextInt64(0, 1L << 32), (uint)seed.NextInt64(0, 1L << 32));

            // Every exponent against exponents near it, where sums cancel, carry and round, with the mantissas that sit on a rounding boundary.
            for (int exponent = 1; exponent <= 254; exponent++)
            {
                for (int gap = -32; gap <= 32; gap++)
                {
                    int other = exponent + gap;
                    if (other < 0 || other > 255) continue;

                    foreach (uint m in Mantissas)
                    {
                        foreach (uint n in Mantissas)
                        {
                            for (int signs = 0; signs < 4; signs++) Try(Make(signs & 1, exponent, m), Make(signs >> 1, other, n));
                        }
                    }

                    for (int i = 0; i < 12; i++) Try(Make(seed.Next(2), exponent, (uint)seed.Next(1 << 23)), Make(seed.Next(2), other, (uint)seed.Next(1 << 23)));
                }
            }

            // Products whose exponents add to either end of the range, where the host must decline rather than guess.
            for (int i = 0; i < 400_000; i++)
            {
                int exponent = seed.Next(1, 255);
                int other = seed.Next(2) == 0 ? Math.Clamp(127 + 127 - exponent + seed.Next(-3, 4), 1, 254) : Math.Clamp(127 - 126 - exponent + 127 + seed.Next(-3, 4), 1, 254);
                Try(Make(seed.Next(2), exponent, (uint)seed.Next(1 << 23)), Make(seed.Next(2), other, (uint)seed.Next(1 << 23)));
            }

            Assert.Null(first);
            Assert.True(answered > 1_000_000, $"the host answered only {answered} cases, so the test compared too little");
            Assert.True(inexact > 100_000 && declined > 100_000, $"inexact {inexact}, declined {declined}: a side of the test was not exercised");
        }
    }
}
