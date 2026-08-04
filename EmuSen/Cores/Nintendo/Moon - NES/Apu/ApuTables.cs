namespace EmuSen.Cores.Nintendo.Moon.Apu
{
    // The 2A03's fixed lookup tables - see Moon_APU.md §3.
    internal static class ApuTables
    {
        // $4003/$4007/$400B/$400F bits 3-7 index this to load a length counter.
        public static readonly byte[] LengthCounter =
        {
            10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
            12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
        };

        // Four duty cycles, eight steps each; sequence 3 is 25% inverted, not 75%.
        public static readonly byte[][] PulseDuty =
        {
            new byte[] { 0, 1, 0, 0, 0, 0, 0, 0 },
            new byte[] { 0, 1, 1, 0, 0, 0, 0, 0 },
            new byte[] { 0, 1, 1, 1, 1, 0, 0, 0 },
            new byte[] { 1, 0, 0, 1, 1, 1, 1, 1 },
        };

        // 32 steps down then back up, so the triangle never leaves a DC step behind.
        public static readonly byte[] TriangleSequence =
        {
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        };

        // NTSC noise periods in APU cycles - see Moon_APU.md §3.4.
        public static readonly int[] NoisePeriod =
        {
            4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
        };

        // NTSC DMC rates in CPU cycles - see Moon_APU.md §3.5.
        public static readonly int[] DmcRate =
        {
            428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
        };
    }
}
