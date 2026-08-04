using System;
using System.Linq;
using NesApu = EmuSen.Cores.Nintendo.Moon.Apu.Apu;

namespace EmuSen.WiseMan.Cores
{
    // The 2A03 synthesis path - see Moon_APU.md.
    public class MoonApuTests
    {
        private static NesApu NewApu()
        {
            var apu = new NesApu();
            apu.Reset();
            apu.SetSampleRate(44100);
            return apu;
        }

        // A440-ish square: period 253 is around 440 Hz at the NTSC rate.
        private static NesApu PlayingPulse()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4015, 0x01);   // enable pulse 1
            apu.WriteRegister(0x4000, 0xBF);   // 50% duty, constant volume 15, length halted
            apu.WriteRegister(0x4002, 0xFD);
            apu.WriteRegister(0x4003, 0x08);   // period high + length load
            return apu;
        }

        [Fact]
        public void A_playing_pulse_produces_a_non_silent_buffer()
        {
            NesApu apu = PlayingPulse();
            apu.Step(60000);

            short[] samples = apu.Drain(int.MaxValue);

            Assert.NotEmpty(samples);
            Assert.Contains(samples, s => s != 0);
        }

        [Fact]
        public void Silence_stays_silent()
        {
            NesApu apu = NewApu();
            apu.Step(60000);

            Assert.All(apu.Drain(int.MaxValue), s => Assert.Equal(0, s));
        }

        // One output frame is two shorts, so a drain must never split a pair.
        [Fact]
        public void Drain_returns_whole_stereo_frames()
        {
            NesApu apu = PlayingPulse();
            apu.Step(60000);

            Assert.Equal(0, apu.Drain(int.MaxValue).Length % 2);
        }

        [Fact]
        public void Both_output_channels_carry_the_same_mono_sample()
        {
            NesApu apu = PlayingPulse();
            apu.Step(60000);

            short[] s = apu.Drain(int.MaxValue);
            for (int i = 0; i + 1 < s.Length; i += 2) Assert.Equal(s[i], s[i + 1]);
        }

        // One second of CPU cycles should yield about one second of audio.
        [Fact]
        public void Sample_output_tracks_the_requested_rate()
        {
            NesApu apu = PlayingPulse();
            apu.Step((int)NesApu.CpuClockHz);

            int frames = apu.Drain(int.MaxValue).Length / 2;

            Assert.InRange(frames, 43000, 45200);
        }

        [Fact]
        public void The_buffer_is_capped_rather_than_growing_without_bound()
        {
            NesApu apu = PlayingPulse();
            apu.MaxBufferedSamples = 2000;

            apu.Step((int)NesApu.CpuClockHz);

            Assert.True(apu.BufferedSamples <= 2000, $"buffer held {apu.BufferedSamples}");
        }

        // The output decays through the high-pass rather than snapping to zero - see Moon_APU.md §4.1.
        [Fact]
        public void Disabling_a_channel_through_4015_silences_it()
        {
            NesApu apu = PlayingPulse();
            apu.Step(20000);
            int playingPeak = apu.Drain(int.MaxValue).Select(s => Math.Abs((int)s)).DefaultIfEmpty(0).Max();

            apu.WriteRegister(0x4015, 0x00);
            apu.Step(40000);
            short[] after = apu.Drain(int.MaxValue);

            // The tail is what the filter is still settling; the last of it must be effectively silent.
            int tailPeak = after.TakeLast(200).Select(s => Math.Abs((int)s)).DefaultIfEmpty(0).Max();

            Assert.True(playingPeak > 1000, $"the channel should have been audible, peaked at {playingPeak}");
            Assert.True(tailPeak < playingPeak / 50, $"tail peaked at {tailPeak} against {playingPeak} playing");
        }

        [Fact]
        public void Status_reports_a_channel_with_a_live_length_counter()
        {
            NesApu apu = PlayingPulse();

            Assert.Equal(0x01, apu.ReadStatus() & 0x01);

            apu.WriteRegister(0x4015, 0x00);
            Assert.Equal(0, apu.ReadStatus() & 0x01);
        }

        // The four-step sequencer raises the frame IRQ once per period unless inhibited.
        [Fact]
        public void The_four_step_sequencer_raises_a_frame_irq()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4017, 0x00);

            apu.Step(30000);

            Assert.True(apu.FrameIrqPending);
        }

        [Fact]
        public void The_inhibit_bit_suppresses_the_frame_irq()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4017, 0x40);

            apu.Step(60000);

            Assert.False(apu.FrameIrqPending);
        }

        [Fact]
        public void Five_step_mode_never_raises_the_frame_irq()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4017, 0x80);

            apu.Step(120000);

            Assert.False(apu.FrameIrqPending);
        }

        // A period under 8 is muted by the sweep unit, not by the envelope.
        [Fact]
        public void A_pulse_period_below_eight_is_muted()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4015, 0x01);
            apu.WriteRegister(0x4000, 0xBF);
            apu.WriteRegister(0x4002, 0x04);
            apu.WriteRegister(0x4003, 0x08);

            apu.Step(60000);

            Assert.All(apu.Drain(int.MaxValue), s => Assert.Equal(0, s));
        }

        [Fact]
        public void The_triangle_runs_without_any_volume_control()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4015, 0x04);   // enable triangle
            apu.WriteRegister(0x4008, 0xFF);   // control set, linear counter loaded
            apu.WriteRegister(0x400A, 0x40);
            apu.WriteRegister(0x400B, 0x08);

            apu.Step(60000);

            Assert.Contains(apu.Drain(int.MaxValue), s => s != 0);
        }

        [Fact]
        public void The_noise_channel_produces_output()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4015, 0x08);   // enable noise
            apu.WriteRegister(0x400C, 0x3F);   // constant volume 15, halted
            apu.WriteRegister(0x400E, 0x04);
            apu.WriteRegister(0x400F, 0x08);

            apu.Step(60000);

            Assert.Contains(apu.Drain(int.MaxValue), s => s != 0);
        }

        // $4011 is a direct DAC write, so it moves the output with no sample playing.
        [Fact]
        public void Writing_the_dmc_level_directly_moves_the_output()
        {
            NesApu apu = NewApu();
            apu.WriteRegister(0x4011, 0x7F);

            apu.Step(4000);

            Assert.Contains(apu.Drain(int.MaxValue), s => s != 0);
        }

        // Two channels together must not simply sum - the DAC is non-linear.
        [Fact]
        public void The_mixer_is_non_linear_across_two_pulses()
        {
            static double PeakOf(Action<NesApu> setup)
            {
                NesApu apu = NewApu();
                setup(apu);
                apu.Step(60000);
                return apu.Drain(int.MaxValue).Select(s => (double)Math.Abs(s)).DefaultIfEmpty(0).Max();
            }

            double one = PeakOf(a =>
            {
                a.WriteRegister(0x4015, 0x01);
                a.WriteRegister(0x4000, 0xBF); a.WriteRegister(0x4002, 0xFD); a.WriteRegister(0x4003, 0x08);
            });

            double two = PeakOf(a =>
            {
                a.WriteRegister(0x4015, 0x03);
                a.WriteRegister(0x4000, 0xBF); a.WriteRegister(0x4002, 0xFD); a.WriteRegister(0x4003, 0x08);
                a.WriteRegister(0x4004, 0xBF); a.WriteRegister(0x4006, 0xFD); a.WriteRegister(0x4007, 0x08);
            });

            Assert.True(two > one, "two pulses should be louder than one");
            Assert.True(two < one * 2, $"the DAC is non-linear, so {two} should be under {one * 2}");
        }
    }
}
