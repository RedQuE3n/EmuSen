using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Audio
{
    // ICore.AudioSampleRate / DequeueAudioSamples (VenusCore's
    // implementation) and EmulatorSession's pass-through - the audio
    // counterpart to GetFrameBufferRgba(), added so a frontend can drain
    // synthesized audio without reaching into Spc700.Dsp.AudioBuffer (a
    // real SNES-specific type) directly. See ICore.cs's own comment.
    public class AudioDrainTests
    {
        [Fact]
        public void AudioSampleRate_matches_AudioSettings()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            Assert.Equal(EmuSen.Audio.AudioSettings.SampleRate, core.AudioSampleRate);
        }

        [Fact]
        public void Dequeue_with_nothing_queued_returns_empty()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            Assert.Empty(core.DequeueAudioSamples(100));
        }

        [Fact]
        public void Dequeue_respects_cap_and_drains_in_fifo_order()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var buffer = core.Spc700!.Dsp.AudioBuffer;
            for (int i = 0; i < 10; i++)
            {
                buffer.Enqueue((short)(1000 + i)); // L
                buffer.Enqueue((short)(-1000 - i)); // R
            }

            short[] capped = core.DequeueAudioSamples(4);

            Assert.Equal(8, capped.Length); // 4 frames = 8 shorts
            Assert.Equal(new short[] { 1000, -1000, 1001, -1001, 1002, -1002, 1003, -1003 }, capped);
            Assert.Equal(12, buffer.Count); // 6 frames left
        }

        [Fact]
        public void Dequeue_asking_for_more_than_available_returns_only_whats_queued()
        {
            var core = SyntheticRom.LoadCore(SyntheticRom.BuildBlank());
            var buffer = core.Spc700!.Dsp.AudioBuffer;
            buffer.Enqueue(1);
            buffer.Enqueue(2);
            buffer.Enqueue(3);
            buffer.Enqueue(4); // 2 frames

            short[] result = core.DequeueAudioSamples(1000);

            Assert.Equal(new short[] { 1, 2, 3, 4 }, result);
            Assert.Empty(buffer);
        }

        [Fact]
        public void EmulatorSession_before_LoadRom_falls_back_instead_of_throwing()
        {
            var session = new EmuSen.Common.EmulatorSession();
            Assert.Equal(EmuSen.Audio.AudioSettings.SampleRate, session.AudioSampleRate);
            Assert.Empty(session.DequeueAudioSamples(10));
        }

        [Fact]
        public void EmulatorSession_drains_through_to_the_real_core()
        {
            var session = SyntheticRom.LoadSession(SyntheticRom.BuildBlank());
            ((EmuSen.Cores.Nintendo.Venus.VenusCore)session.Core!).Bus!.Spc700.Dsp.AudioBuffer.Enqueue(42);
            ((EmuSen.Cores.Nintendo.Venus.VenusCore)session.Core!).Bus!.Spc700.Dsp.AudioBuffer.Enqueue(-42);

            short[] result = session.DequeueAudioSamples(10);

            Assert.Equal(new short[] { 42, -42 }, result);
        }
    }
}
