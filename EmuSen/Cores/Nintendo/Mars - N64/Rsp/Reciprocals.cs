using System.Numerics;

namespace EmuSen.Cores.Nintendo.Mars.Rsp
{
    // The reciprocal and reciprocal-square-root tables, and the one lookup both share - see Mars_RspVector.md §10.
    internal static class Reciprocals
    {
        private const int TableSize = 512;

        private static readonly ushort[] Reciprocal = BuildReciprocals();
        private static readonly ushort[] RootReciprocal = BuildRootReciprocals();

        public static uint Of(uint value) => Evaluate(value, root: false);

        public static uint RootOf(uint value) => Evaluate(value, root: true);

        // Negative inputs past 0xFFFF8000 are taken one lower before inverting, which the corpus's model does without explaining - see §10.1.
        private static uint Evaluate(uint value, bool root)
        {
            if (value == 0) return 0x7FFF_FFFF;
            if (value == 0xFFFF_8000) return 0xFFFF_0000;

            uint adjusted = value > 0xFFFF_8000 ? value - 1 : value;
            bool negative = (int)adjusted < 0;
            uint magnitude = negative ? ~adjusted : adjusted;

            int shift = BitOperations.LeadingZeroCount(magnitude) + 1;

            // A shift of thirty-two is taken modulo thirty-two, in C# as in the corpus's model of the part - see §10.1.
            uint normalised = magnitude << shift;

            uint result = root
                ? (0x4000_0000u | ((uint)RootReciprocal[(normalised >> 24) | ((uint)(shift & 1) << 8)] << 14)) >> ((32 - shift) >> 1)
                : (0x4000_0000u | ((uint)Reciprocal[normalised >> 23] << 14)) >> (32 - shift);

            return negative ? ~result : result;
        }

        private static ushort[] BuildReciprocals()
        {
            var table = new ushort[TableSize];
            table[0] = 0xFFFF;

            for (int i = 1; i < TableSize; i++) table[i] = (ushort)((((1UL << 34) / (ulong)(i + 512)) + 1) >> 8);

            return table;
        }

        // Each entry is the largest value whose square, times a factor taken from its index, stays under 2^44 - see §10.
        private static ushort[] BuildRootReciprocals()
        {
            var table = new ushort[TableSize];

            for (int i = 0; i < TableSize; i++)
            {
                ulong scale = i < 256 ? (ulong)(i + 256) : ((ulong)(i - 256) << 1) + 512;
                ulong root = 1UL << 17;

                for (ulong step = 512; step != 0; step >>= 1)
                {
                    while (scale * (root + step) * (root + step) < (1UL << 44)) root += step;
                }

                table[i] = (ushort)(root >> 1);
            }

            return table;
        }
    }
}
