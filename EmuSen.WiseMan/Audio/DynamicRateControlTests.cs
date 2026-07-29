using System;
using EmuSen.Audio;

namespace EmuSen.WiseMan.Audio
{
    // The control loop that replaced the 150ms drop - see EmuSen_Audio_Sync.md §3.
    public class DynamicRateControlTests
    {
        private const int Target = 8192;

        private static short[] Silence(int frames) => new short[frames * 2];

        [Fact]
        public void At_target_the_ratio_is_exactly_one()
        {
            Assert.Equal(1.0, new DynamicRateControl(Target).ComputeRatio(Target), precision: 12);
        }

        // Too much queued means emit fewer frames per frame consumed - §3.
        [Fact]
        public void An_overfull_queue_asks_for_a_ratio_below_one()
        {
            Assert.True(new DynamicRateControl(Target).ComputeRatio(Target * 2) < 1.0);
        }

        [Fact]
        public void A_draining_queue_asks_for_a_ratio_above_one()
        {
            Assert.True(new DynamicRateControl(Target).ComputeRatio(Target / 4) > 1.0);
        }

        // The pitch shift has to stay under perception however far off we are - §3.
        [Fact]
        public void The_ratio_never_leaves_the_configured_deviation_band()
        {
            var d = new DynamicRateControl(Target) { MaxDeviation = 0.005 };
            foreach (int queued in new[] { 0, 1, Target / 2, Target, Target * 4, Target * 100 })
            {
                double ratio = d.ComputeRatio(queued);
                Assert.InRange(ratio, 1.0 - 0.005, 1.0 + 0.005);
            }
        }

        [Fact]
        public void An_empty_queue_asks_for_the_maximum_stretch()
        {
            var d = new DynamicRateControl(Target) { MaxDeviation = 0.005 };
            Assert.Equal(1.005, d.ComputeRatio(0), precision: 9);
        }

        // Steady state is where output exactly matches drain, i.e.
        // ratio == drain/in. Solving the control law for that gives the
        // queue depth the loop must settle at - see EmuSen_Audio_Sync.md §3.
        private static double PredictedEquilibrium(int inFrames, int drainFrames, double maxDeviation = 0.005)
        {
            double ratio = drainFrames / (double)inFrames;
            return Target * (1.0 + (1.0 - ratio) / maxDeviation);
        }

        private static (int Queued, int Shedding) Settle(int inFrames, int drainFrames, int frames = 20000)
        {
            var d = new DynamicRateControl(Target);
            int queued = Target;
            for (int i = 0; i < frames; i++)
            {
                queued += d.Process(Silence(inFrames), queued).Length / 2;
                queued = Math.Max(0, queued - drainFrames);
            }
            return (queued, d.SheddingEvents);
        }

        // A core producing slightly fast must settle, not drift away - §3.
        [Fact]
        public void A_persistently_fast_producer_settles_where_the_control_law_predicts()
        {
            var (queued, shedding) = Settle(inFrames: 534, drainFrames: 533);
            Assert.Equal(PredictedEquilibrium(534, 533), queued, tolerance: 40);
            Assert.Equal(0, shedding);
        }

        [Fact]
        public void A_persistently_slow_producer_settles_where_the_control_law_predicts()
        {
            var (queued, shedding) = Settle(inFrames: 531, drainFrames: 533);
            Assert.Equal(PredictedEquilibrium(531, 533), queued, tolerance: 40);
            Assert.Equal(0, shedding);
        }

        // Whichever way it is off, it must stop moving rather than run to an
        // end stop - the old mechanism's failure was unbounded accumulation.
        [Fact]
        public void The_queue_stops_moving_once_settled()
        {
            foreach (int inFrames in new[] { 531, 532, 533, 534, 535 })
            {
                int shortRun = Settle(inFrames, 533, frames: 20000).Queued;
                int longRun = Settle(inFrames, 533, frames: 40000).Queued;
                Assert.True(Math.Abs(longRun - shortRun) < 60,
                    $"in={inFrames}: still drifting, {shortRun} -> {longRun}");
            }
        }

        // The old mechanism's failure case: steady over-production reached a
        // 150ms discard every ~30s, forever - §1.
        [Fact]
        public void Ten_minutes_of_steady_over_production_never_sheds_anything()
        {
            var (_, shedding) = Settle(inFrames: 534, drainFrames: 533, frames: 60 * 60 * 10);
            Assert.Equal(0, shedding);
        }

        // Drift larger than the deviation band cannot be corrected - worth
        // knowing, and why the shedding path exists at all - §3.1.
        [Fact]
        public void Drift_beyond_the_deviation_band_is_not_correctable()
        {
            var (queued, _) = Settle(inFrames: 500, drainFrames: 533);
            Assert.Equal(0, queued); // starves however hard it stretches
        }

        [Fact]
        public void A_gross_backlog_engages_shedding_and_then_recovers()
        {
            var d = new DynamicRateControl(Target);

            Assert.Empty(d.Process(Silence(533), Target * 4));
            Assert.True(d.IsShedding);
            Assert.Equal(1, d.SheddingEvents);

            Assert.Empty(d.Process(Silence(533), (int)(Target * 2.5)));
            Assert.True(d.IsShedding);

            Assert.NotEmpty(d.Process(Silence(533), Target));
            Assert.False(d.IsShedding);
        }

        [Fact]
        public void Reset_clears_shedding_state()
        {
            var d = new DynamicRateControl(Target);
            d.Process(Silence(533), Target * 4);
            Assert.True(d.IsShedding);

            d.Reset();
            Assert.False(d.IsShedding);
            Assert.Equal(1.0, d.LastRatio);
        }

        [Fact]
        public void A_zero_target_degrades_to_passthrough_rather_than_dividing_by_zero()
        {
            Assert.Equal(1.0, new DynamicRateControl(0).ComputeRatio(1234));
        }
    }
}
