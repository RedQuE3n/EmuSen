using System;
using System.Linq;
using EmuSen.Endymion;

namespace EmuSen.WiseMan.Audio
{
    // The resampler dynamic rate control drives - see EmuSen_Audio_Sync.md §2.
    public class LinearResamplerTests
    {
        // Interleaved stereo where R is always L negated, so a channel swap shows up.
        private static short[] Ramp(int frames, int start = 0)
        {
            var a = new short[frames * 2];
            for (int i = 0; i < frames; i++)
            {
                a[i * 2] = (short)(start + i);
                a[i * 2 + 1] = (short)-(start + i);
            }
            return a;
        }

        private static short[] LeftChannel(short[] interleaved)
        {
            var l = new short[interleaved.Length / 2];
            for (int i = 0; i < l.Length; i++) l[i] = interleaved[i * 2];
            return l;
        }

        [Fact]
        public void Ratio_of_one_passes_samples_through_unchanged()
        {
            var r = new LinearResampler();
            short[] outp = r.Resample(Ramp(100), 1.0);
            Assert.Equal(Enumerable.Range(0, outp.Length / 2).Select(i => (short)i), LeftChannel(outp));
        }

        // The property that makes chunked pumping safe - §2.
        [Fact]
        public void Ratio_of_one_stays_continuous_across_chunk_boundaries()
        {
            var r = new LinearResampler();
            var all = new System.Collections.Generic.List<short>();
            for (int chunk = 0; chunk < 5; chunk++) all.AddRange(r.Resample(Ramp(50, chunk * 50), 1.0));

            short[] left = LeftChannel(all.ToArray());
            for (int i = 0; i < left.Length; i++) Assert.Equal((short)i, left[i]);
        }

        [Fact]
        public void Right_channel_never_leaks_into_left()
        {
            var r = new LinearResampler();
            var all = new System.Collections.Generic.List<short>();
            for (int chunk = 0; chunk < 5; chunk++) all.AddRange(r.Resample(Ramp(37, chunk * 37), 0.997));

            short[] outp = all.ToArray();
            for (int i = 0; i < outp.Length / 2; i++)
            {
                Assert.Equal(outp[i * 2], (short)-outp[i * 2 + 1]);
            }
        }

        [Fact]
        public void Slowing_down_produces_more_frames_than_it_consumed()
        {
            var r = new LinearResampler();
            int produced = 0;
            for (int chunk = 0; chunk < 20; chunk++) produced += r.Resample(Ramp(500, chunk * 500), 1.005).Length / 2;
            Assert.InRange(produced, 10000, 10100); // 20*500 input frames, ~+0.5%
        }

        [Fact]
        public void Speeding_up_produces_fewer_frames_than_it_consumed()
        {
            var r = new LinearResampler();
            int produced = 0;
            for (int chunk = 0; chunk < 20; chunk++) produced += r.Resample(Ramp(500, chunk * 500), 0.995).Length / 2;
            Assert.InRange(produced, 9900, 10000);
        }

        // Drift must not accumulate over a long session - §2.
        [Fact]
        public void Output_frame_count_tracks_the_ratio_over_a_long_run()
        {
            var r = new LinearResampler();
            int produced = 0;
            const int chunks = 2000, frames = 533; // ~35s of real playback
            for (int c = 0; c < chunks; c++) produced += r.Resample(Ramp(frames, c * frames), 1.002).Length / 2;

            double expected = chunks * frames * 1.002;
            Assert.True(Math.Abs(produced - expected) < 5, $"expected ~{expected:F0} frames, got {produced}");
        }

        [Fact]
        public void A_reset_restarts_cleanly_rather_than_carrying_stale_phase()
        {
            var r = new LinearResampler();
            r.Resample(Ramp(100), 0.99);
            r.Reset();
            short[] outp = r.Resample(Ramp(100), 1.0);
            Assert.Equal((short)0, outp[0]);
        }

        [Fact]
        public void An_empty_or_single_sample_chunk_is_ignored()
        {
            var r = new LinearResampler();
            Assert.Empty(r.Resample(Array.Empty<short>(), 1.0));
            Assert.Empty(r.Resample(new short[] { 5 }, 1.0));
        }

        [Fact]
        public void A_non_positive_ratio_throws_rather_than_looping_forever()
        {
            var r = new LinearResampler();
            Assert.Throws<ArgumentOutOfRangeException>(() => r.Resample(Ramp(10), 0.0));
            Assert.Throws<ArgumentOutOfRangeException>(() => r.Resample(Ramp(10), -1.0));
        }

        // A near-1.0 ratio must stay close to the source, since that is the
        // whole reason linear interpolation is good enough here - §2.
        [Fact]
        public void Interpolation_error_stays_tiny_at_rate_control_ratios()
        {
            var r = new LinearResampler();
            short[] outp = r.Resample(Ramp(1000), 1.005);
            short[] left = LeftChannel(outp);
            for (int i = 1; i < left.Length; i++)
            {
                Assert.InRange(left[i] - left[i - 1], 0, 1); // monotone ramp preserved
            }
        }
    }
}
