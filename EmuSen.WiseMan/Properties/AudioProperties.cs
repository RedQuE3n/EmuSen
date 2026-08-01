using System;
using System.Linq;
using CsCheck;
using EmuSen.Audio;

namespace EmuSen.WiseMan.Properties
{
    // Resampler/rate-control invariants - see EmuSen_Debugging_Tools_Reference_v5.md §3.18b.
    public class AudioProperties
    {
        // Ratios well beyond the fraction of a percent rate control actually uses.
        private static readonly Gen<double> Ratio = Gen.Int[1, 40000].Select(n => n / 10000.0);

        [Fact]
        public void Resampled_output_is_whole_interleaved_stereo_frames() =>
            Gen.Select(Gen.Short.Array, Ratio).Sample((input, ratio) =>
                new LinearResampler().Resample(input, ratio).Length % 2 == 0, iter: 2000);

        // Linear interpolation between two samples cannot leave the range they span.
        [Fact]
        public void Resampled_output_never_exceeds_the_inputs_own_range() =>
            Gen.Select(Gen.Short.Array[2, 400], Ratio).Sample((input, ratio) =>
            {
                short[] output = new LinearResampler().Resample(input, ratio);
                short lo = input.Min(), hi = input.Max();
                return output.All(s => s >= lo && s <= hi);
            }, iter: 2000);

        [Fact]
        public void A_non_positive_ratio_is_rejected() =>
            Gen.Select(Gen.Short.Array, Gen.Double[-1000.0, 0.0]).Sample((input, ratio) =>
            {
                try { new LinearResampler().Resample(input, ratio); return false; }
                catch (ArgumentOutOfRangeException) { return true; }
            }, iter: 500);

        [Fact]
        public void Fewer_than_two_samples_is_not_a_whole_frame() =>
            Ratio.Sample(ratio =>
                new LinearResampler().Resample(Array.Empty<short>(), ratio).Length == 0
                && new LinearResampler().Resample(new short[] { 1 }, ratio).Length == 0, iter: 500);

        [Fact]
        public void The_ratio_never_departs_further_than_MaxDeviation_allows() =>
            Gen.Select(Gen.Int[1, 10000], Gen.Int[0, 100000]).Sample((target, queued) =>
            {
                var drc = new DynamicRateControl(target);
                double ratio = drc.ComputeRatio(queued);
                return ratio >= 1.0 - drc.MaxDeviation - 1e-12 && ratio <= 1.0 + drc.MaxDeviation + 1e-12;
            }, iter: 5000);

        [Fact]
        public void A_queue_on_target_asks_for_no_correction() =>
            Gen.Int[1, 10000].Sample(target => new DynamicRateControl(target).ComputeRatio(target) == 1.0, iter: 2000);

        [Fact]
        public void No_target_means_no_rate_control() =>
            Gen.Select(Gen.Int[-1000, 0], Gen.Int[0, 100000]).Sample((target, queued) =>
                new DynamicRateControl(target).ComputeRatio(queued) == 1.0, iter: 2000);

        // A fuller queue must never ask to speed up - the whole steering direction.
        [Fact]
        public void A_fuller_queue_never_raises_the_ratio() =>
            Gen.Select(Gen.Int[1, 10000], Gen.Int[0, 100000], Gen.Int[0, 100000]).Sample((target, a, delta) =>
            {
                var drc = new DynamicRateControl(target);
                return drc.ComputeRatio(a) >= drc.ComputeRatio(a + delta);
            }, iter: 5000);
    }
}
