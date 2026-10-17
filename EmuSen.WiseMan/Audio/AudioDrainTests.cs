using System;
using EmuSen.Cores;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Audio
{
    // ICore.AudioSampleRate and DequeueAudioSamples on the SNES's engine, and EmulatorSession's pass-through - see ICore.cs.
    public class AudioDrainTests
    {
        // VenusRT gives the S-DSP's own 32 kHz, and the host resamples - see VenusRT_Native.md §22.
        [Fact]
        public void AudioSampleRate_is_the_engines_own()
        {
            using var core = (IDisposable)SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            Assert.Equal(32000, ((ICore)core).AudioSampleRate);
        }

        [Fact]
        public void Dequeue_with_nothing_queued_returns_empty()
        {
            using var core = (IDisposable)SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            Assert.Empty(((ICore)core).DequeueAudioSamples(100));
        }

        [Fact]
        public void Dequeue_respects_its_cap_in_whole_frames_and_leaves_the_rest()
        {
            ICore core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            core.RunFrame();

            Assert.Equal(8, core.DequeueAudioSamples(4).Length); // 4 frames = 8 shorts
            short[] rest = core.DequeueAudioSamples(int.MaxValue);
            Assert.NotEmpty(rest);
            Assert.Equal(0, rest.Length % 2);
            Assert.Empty(core.DequeueAudioSamples(int.MaxValue));
            (core as IDisposable)?.Dispose();
        }

        [Fact]
        public void EmulatorSession_before_LoadRom_falls_back_instead_of_throwing()
        {
            var session = new EmuSen.Common.EmulatorSession();
            Assert.Equal(EmuSen.Galaxia.Models.AudioConfig.DefaultSampleRate, session.AudioSampleRate);
            Assert.Empty(session.DequeueAudioSamples(10));
        }

        [Fact]
        public void EmulatorSession_drains_through_to_the_real_core()
        {
            var session = SyntheticRom.LoadSession(SyntheticRom.BuildBlank());
            session.RunFrame();

            Assert.NotEmpty(session.DequeueAudioSamples(int.MaxValue));
            Assert.Empty(session.DequeueAudioSamples(int.MaxValue));
        }
    }
}
