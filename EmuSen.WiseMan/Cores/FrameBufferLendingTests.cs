using EmuSen.Cores;

namespace EmuSen.WiseMan.Cores
{
    // The lending a core's IFrameBufferPool rests on: an array goes back once, and only one it lent at the size it lends - see EmuSen_Multicore.md §16.
    public class FrameBufferLendingTests
    {
        private const int Length = 64;

        [Fact]
        public void A_returned_array_is_lent_again_and_a_held_one_never_is()
        {
            var lending = new FrameBufferLending();
            byte[] a = lending.Lend(Length), b = lending.Lend(Length);
            Assert.NotSame(a, b);
            lending.Return(a);
            byte[] again = lending.Lend(Length);
            Assert.Same(a, again);
            Assert.NotSame(b, lending.Lend(Length));
            Assert.Equal((3L, 1L), (lending.Made, lending.Reused));
        }

        [Fact]
        public void Returning_an_array_twice_does_not_lend_it_twice()
        {
            var lending = new FrameBufferLending();
            byte[] a = lending.Lend(Length);
            lending.Return(a);
            lending.Return(a);
            Assert.Equal(1, lending.Free);
            Assert.Equal(1, lending.Dropped);
            byte[] first = lending.Lend(Length), second = lending.Lend(Length);
            Assert.Same(a, first);
            Assert.NotSame(a, second);
        }

        [Fact]
        public void A_foreign_array_or_one_lent_at_another_size_is_dropped()
        {
            var lending = new FrameBufferLending();
            lending.Return(new byte[Length]);
            Assert.Equal(0, lending.Free);

            byte[] small = lending.Lend(Length);
            byte[] large = lending.Lend(Length * 4);
            lending.Return(small);
            Assert.Equal(0, lending.Free);
            Assert.NotSame(small, lending.Lend(Length * 4));
            Assert.NotSame(small, lending.Lend(Length));

            // A foreign array of the very size lent is still not taken.
            byte[] kept = lending.Lend(Length);
            byte[] foreign = new byte[Length];
            lending.Return(foreign);
            Assert.Equal(0, lending.Free);

            // A free array of the old size is not lent at the new one.
            lending.Return(kept);
            Assert.Equal(Length * 4, lending.Lend(Length * 4).Length);
            Assert.Equal(3, lending.Dropped);
            Assert.NotSame(foreign, lending.Lend(Length));
            GC.KeepAlive(large);
        }

        [Fact]
        public void The_lending_stays_bounded_whatever_is_returned()
        {
            var lending = new FrameBufferLending();
            var lent = new byte[FrameBufferLending.LentLimit + 4][];
            for (int i = 0; i < lent.Length; i++) lent[i] = lending.Lend(Length);

            // The four oldest were forgotten, so they are the caller's for good, and the free list holds at most its limit.
            foreach (byte[] buffer in lent) lending.Return(buffer);
            Assert.Equal(FrameBufferLending.FreeLimit, lending.Free);
            Assert.Equal(lent.Length - FrameBufferLending.FreeLimit, lending.Dropped);
            for (int i = 0; i < FrameBufferLending.FreeLimit; i++) Assert.Contains(lending.Lend(Length), lent[^FrameBufferLending.LentLimit..]);
        }

        [Fact]
        public void A_closed_lending_takes_nothing_back()
        {
            var lending = new FrameBufferLending();
            byte[] a = lending.Lend(Length), b = lending.Lend(Length);
            lending.Return(a);
            lending.Close();
            lending.Return(b);
            Assert.Equal(0, lending.Free);
            Assert.NotSame(a, lending.Lend(Length));
        }
    }
}
