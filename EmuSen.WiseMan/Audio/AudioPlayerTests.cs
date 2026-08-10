using System.Linq;
using EmuSen.Common;
using EmuSen.Endymion;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Audio
{
    // Exercises AudioPlayer's real SDL3 P/Invoke calls against SDL's "dummy"
    // audio driver - see EmuSen_Settings_Reference.md §4.10.
    [Collection(TestCollections.ProcessGlobals)]
    public class AudioPlayerTests
    {
        static AudioPlayerTests()
        {
            SDL.SetHint(SDL.Hints.AudioDriver, "dummy");
        }

        [Fact]
        public void Opens_successfully_against_the_dummy_driver()
        {
            using var player = new AudioPlayer();
            Assert.True(player.IsAvailable);
        }

        [Fact]
        public void Submitting_nothing_does_not_throw()
        {
            using var player = new AudioPlayer();
            var session = new EmulatorSession();

            Exception? ex = Record.Exception(() =>
                player.Submit(session.DequeueAudioSamples(int.MaxValue), session.AudioSampleRate));

            Assert.Null(ex);
        }

        // The sink takes data, never a core - the same contract Serenity has - see EmuSen_Audio_Sync.md §7.
        [Fact]
        public void Submit_names_no_core_type()
        {
            var parameters = typeof(AudioPlayer).GetMethod(nameof(AudioPlayer.Submit))!.GetParameters();

            Assert.Equal(new[] { typeof(short[]), typeof(int) }, parameters.Select(p => p.ParameterType));
        }

        // A core with a different rate must move the device, not play at the wrong speed - see §7.2.
        [Fact]
        public void Submitting_at_a_new_rate_reopens_the_device_at_that_rate()
        {
            using var player = new AudioPlayer();
            if (!player.IsAvailable) return; // no dummy device in this environment

            int original = player.SampleRate;
            int different = original == 44100 ? 32000 : 44100;

            player.Submit(new short[] { 0, 0, 0, 0 }, different);

            Assert.Equal(different, player.SampleRate);
            Assert.NotEqual(original, player.SampleRate);
        }

        // The latency target is a frame count, so it has to move with the rate.
        [Fact]
        public void Reopening_at_a_new_rate_rescales_the_latency_target()
        {
            using var player = new AudioPlayer();
            if (!player.IsAvailable) return;

            player.Submit(new short[] { 0, 0 }, 32000);
            int at32k = player.RateControl.TargetQueuedFrames;

            player.Submit(new short[] { 0, 0 }, 64000);

            Assert.Equal(at32k * 2, player.RateControl.TargetQueuedFrames);
        }

        [Fact]
        public void Submit_drains_queued_samples_through_to_sdl()
        {
            using var player = new AudioPlayer();
            var session = SyntheticRom.LoadSession(SyntheticRom.BuildBlank());
            var buffer = ((EmuSen.Cores.Nintendo.Venus.VenusCore)session.Core!).Bus!.Spc700.Dsp.AudioBuffer;
            for (int i = 0; i < 2000; i++)
            {
                buffer.Enqueue((short)i);
                buffer.Enqueue((short)-i);
            } // 2000 frames - under AudioPlayer's 4096-frame per-pump cap

            player.Submit(session.DequeueAudioSamples(int.MaxValue), session.AudioSampleRate);

            Assert.Empty(buffer); // fully drained in one call
        }

        [Fact]
        public void Dispose_does_not_throw()
        {
            var player = new AudioPlayer();
            Exception? ex = Record.Exception(player.Dispose);
            Assert.Null(ex);
        }
    }
}
