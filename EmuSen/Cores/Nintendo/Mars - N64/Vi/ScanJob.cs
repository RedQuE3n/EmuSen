namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // One scan's walk, prepared by the interface and run later over the lines captured for it - see Mars_Video.md §2.7.
    public sealed class ScanJob
    {
        internal Vi.Picture Picture;
        internal uint Origin;
        internal int Width;
        internal bool Wide;
        internal bool Resample;
        internal bool Divot;
        internal int AntiAlias;
        internal bool Dither;
        internal bool Gamma;

        // The bytes the walk can reach, computed by Prepare - see §2.7.
        public uint From { get; internal set; }
        public int Count { get; internal set; }

        // The captured lines, where they start in RDRAM, how much was taken, and how large RDRAM is - see §2.7.
        internal byte[] Rdram = System.Array.Empty<byte>();
        internal byte[] Hidden = System.Array.Empty<byte>();
        internal uint Base;
        internal int Length;
        public bool Captured { get; internal set; }
    }
}
