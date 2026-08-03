using EmuSen.Crystal;

namespace EmuSen.WiseMan.Crystal
{
    // Rate conversion, the shape of the bug in Venus_APU.md §2.8 - see EmuSen_Crystal_Scheduler.md §5.
    public class ClockRatioTests
    {
        // The real SNES numbers the SPC700 runs at, which is where §2.8 went wrong.
        private const long MasterHz = 21477272;
        private const long Spc700Hz = 1024000;

        [Fact]
        public void A_ratio_is_reduced_on_construction()
        {
            var r = new ClockRatio(1000, 4000);

            Assert.Equal(1, r.Numerator);
            Assert.Equal(4, r.Denominator);
        }

        [Fact]
        public void A_non_positive_rate_is_rejected()
        {
            Assert.Throws<ArgumentOutOfRangeException>(() => new ClockRatio(0, 10));
            Assert.Throws<ArgumentOutOfRangeException>(() => new ClockRatio(10, 0));
            Assert.Throws<ArgumentOutOfRangeException>(() => new ClockRatio(-1, 10));
        }

        [Fact]
        public void Identity_converts_unchanged()
        {
            var r = new ClockRatio(7, 7);

            Assert.True(r.IsIdentity);
            Assert.Equal(1234, r.ToDevice(1234));
            Assert.Equal(1234, r.ToMaster(1234));
        }

        [Fact]
        public void Master_to_device_and_back_is_stable_on_exact_multiples()
        {
            var r = new ClockRatio(1, 21);

            Assert.Equal(100, r.ToDevice(2100));
            Assert.Equal(2100, r.ToMaster(100));
        }

        // A deadline must never land short of the ticks asked for.
        [Fact]
        public void MasterTicksFor_rounds_up()
        {
            var r = new ClockRatio(2, 3);

            Assert.Equal(2, r.MasterTicksFor(1));
            Assert.Equal(3, r.MasterTicksFor(2));
            Assert.Equal(5, r.MasterTicksFor(3));
        }

        [Fact]
        public void The_spc700_runs_at_roughly_a_twenty_first_of_the_master_clock()
        {
            var r = ClockRatio.FromHz(Spc700Hz, MasterHz);

            long ticks = r.ToDevice(MasterHz);

            Assert.InRange(ticks, Spc700Hz - 2, Spc700Hz);
        }

        // Truncating each call loses the remainder; this is why ClockAccumulator exists.
        [Fact]
        public void Repeated_truncating_conversion_drifts_but_the_accumulator_does_not()
        {
            var r = ClockRatio.FromHz(Spc700Hz, MasterHz);
            const long step = 1364;
            const int steps = 15734;

            long truncated = 0;
            for (int i = 0; i < steps; i++) truncated += r.ToDevice(step);

            var acc = new ClockAccumulator();
            long accumulated = 0;
            for (int i = 0; i < steps; i++) accumulated += acc.Advance(step, r);

            long exact = r.ToDevice(step * steps);

            Assert.Equal(exact, accumulated);
            Assert.True(truncated < exact, $"expected drift below {exact}, got {truncated}");
        }

        [Fact]
        public void The_accumulator_carries_its_remainder_across_calls()
        {
            var r = new ClockRatio(1, 3);
            var acc = new ClockAccumulator();

            Assert.Equal(0, acc.Advance(1, r));
            Assert.Equal(0, acc.Advance(1, r));
            Assert.Equal(1, acc.Advance(1, r));
        }

        [Fact]
        public void The_accumulator_resets_its_remainder()
        {
            var r = new ClockRatio(1, 3);
            var acc = new ClockAccumulator();
            acc.Advance(2, r);

            acc.Reset();

            Assert.Equal(0, acc.Remainder);
            Assert.Equal(0, acc.Advance(2, r));
        }

        [Fact]
        public void A_backwards_advance_is_rejected()
        {
            var r = new ClockRatio(1, 3);
            var acc = new ClockAccumulator();

            Assert.Throws<ArgumentOutOfRangeException>(() => acc.Advance(-1, r));
        }
    }
}
