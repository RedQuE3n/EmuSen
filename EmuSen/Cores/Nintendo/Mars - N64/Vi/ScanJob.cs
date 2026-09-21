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

        // The geometry the last walk ran over; its bytes are still in the capture above, which is written only when a walk follows - see Mars_Video.md §2.8.
        internal int LastCount = -1;
        internal (Vi.Picture Picture, uint Origin, int Width, bool Wide, bool Resample, bool Divot, int AntiAlias, bool Dither, bool Gamma, uint From, int Count, int Scale) LastShape;

        // True when this scan's geometry and bytes are the last walk's, so its walk would write the raster already there - see §2.8.
        public bool Repeats { get; internal set; }

        // The multiple the walk reads at, and the scaled memory's lines captured for it, when a picture at the multiple exists - see Mars_Video.md §2.9.
        public int Scale { get; internal set; } = 1;
        internal Vi.Picture ScaledPicture;
        internal uint ScaledFrom;
        internal int ScaledCount;
        internal uint ScaledBase;
        internal int ScaledLength;
        internal byte[] ScaledRdram = System.Array.Empty<byte>();
        internal byte[] ScaledHidden = System.Array.Empty<byte>();
        internal byte[] LiveScaledRdram = System.Array.Empty<byte>();
        internal byte[] LiveScaledHidden = System.Array.Empty<byte>();

        // The picture as the compute device walked it, one word a pixel, when the device holds the memory at the multiple - see Mars_Gpu.md §13.
        internal uint[] DevicePicture = System.Array.Empty<uint>();
        internal bool DeviceScanned;

        // Forgets the last capture, for a loaded state, whose raster the walk that follows must write - see §2.8.
        public void Forget() => LastCount = -1;
    }
}
