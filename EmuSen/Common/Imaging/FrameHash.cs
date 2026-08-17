namespace EmuSen.Common.Imaging
{
    // 64-bit FNV-1a over raw RGBA bytes - used by --autoshot to decide whether the current frame differs.
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
