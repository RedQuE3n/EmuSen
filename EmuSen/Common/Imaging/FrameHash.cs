namespace EmuSen.Common.Imaging
{
    // 64-bit FNV-1a over raw RGBA bytes - used by --autoshot to decide
    // whether the current frame differs from the last one it saved, without
    // needing a full pixel-by-pixel comparison. Not cryptographically
    // anything; a frame-to-frame "did this change at all" check has no
    // adversarial input to worry about, and FNV-1a's avalanche behavior is
    // more than enough to make two visually different SNES frames collide
    // by chance a non-concern in practice.
    public static class FrameHash
    {
        public static ulong Compute(byte[] data)
        {
            const ulong FnvOffsetBasis = 14695981039346656037;
            const ulong FnvPrime = 1099511628211;

            ulong hash = FnvOffsetBasis;
            foreach (byte b in data)
            {
                hash ^= b;
                hash *= FnvPrime;
            }
            return hash;
        }
    }
}
