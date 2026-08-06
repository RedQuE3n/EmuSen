namespace EmuSen.Cores.Nintendo.Venus.Video
{
    // The compositor's pixel; internal on purpose, so it cannot leave the core - see Venus_PPU.md §15.
    internal readonly struct Rgba32
    {
        public readonly byte R;
        public readonly byte G;
        public readonly byte B;
        public readonly byte A;

        public Rgba32(byte r, byte g, byte b, byte a)
        {
            R = r;
            G = g;
            B = b;
            A = a;
        }
    }
}
