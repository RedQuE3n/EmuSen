using EmuSen.Common.Imaging;

namespace EmuSen.WiseMan.Imaging
{
    // FrameHash.Compute (FNV-1a) - --autoshot's whole "did this frame
    // actually change" mechanism rests on this being deterministic and
    // sensitive to any single-byte change, so those are exactly what's
    // worth locking down here.
    public class FrameHashTests
    {
        [Fact]
        public void Same_input_produces_the_same_hash()
        {
            byte[] data = { 1, 2, 3, 4, 5, 6, 7, 8 };
            Assert.Equal(FrameHash.Compute(data), FrameHash.Compute((byte[])data.Clone()));
        }

        [Fact]
        public void A_single_changed_byte_produces_a_different_hash()
        {
            byte[] a = { 1, 2, 3, 4, 5, 6, 7, 8 };
            byte[] b = (byte[])a.Clone();
            b[3] = (byte)(b[3] + 1);

            Assert.NotEqual(FrameHash.Compute(a), FrameHash.Compute(b));
        }

        [Fact]
        public void Empty_input_is_the_FNV_offset_basis()
        {
            Assert.Equal(14695981039346656037UL, FrameHash.Compute(Array.Empty<byte>()));
        }
    }
}
