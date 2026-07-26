using EmuSen.Common;
using EmuSen.TestingStudio.Audio;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Audio
{
    // Exercises AudioPlayer's actual SDL P/Invoke calls (InitSubSystem,
    // OpenAudioDevice, QueueAudio, GetQueuedAudioSize, CloseAudioDevice,
    // QuitSubSystem) against SDL's "dummy" audio driver - a real,
    // functioning SDL audio backend built for exactly this (headless CI/
    // testing with no real sound hardware), not a fake or a mock. Forced
    // via SDL_AUDIODRIVER before any SDL call in this process, since SDL
    // reads that once at init time - see the static constructor below.
    //
    // This is what actually caught whether Silk.NET.SDL's exact method/
    // struct shapes (OpenAudioDevice's parameter order, AudioSpec's field
    // layout, the AudioS16Sys/InitAudio constant names) were right -
    // confirming this compiles is not the same as confirming it runs
    // without a marshaling crash, which is what these checks are for.
    public class AudioPlayerTests
    {
        // NativeEnvironment.Set, not Environment.SetEnvironmentVariable -
        // see that class's own comment for why (verified directly on this
        // runtime: SDL kept trying ALSA, absent in this sandbox, instead
        // of seeing the requested "dummy" driver at all) and for the
        // per-platform reasoning behind how it's actually fixed.
        static AudioPlayerTests()
        {
            NativeEnvironment.Set("SDL_AUDIODRIVER", "dummy");
        }

        [Fact]
        public void Opens_successfully_against_the_dummy_driver()
        {
            using var player = new AudioPlayer();
            Assert.True(player.IsAvailable);
        }

        [Fact]
        public void Pump_with_no_rom_loaded_does_not_throw()
        {
            using var player = new AudioPlayer();
            var session = new EmulatorSession();

            Exception? ex = Record.Exception(() => player.Pump(session));

            Assert.Null(ex);
        }

        [Fact]
        public void Pump_drains_queued_samples_through_to_sdl()
        {
            using var player = new AudioPlayer();
            var session = SyntheticRom.LoadSession(SyntheticRom.BuildBlank());
            var buffer = session.Bus.Spc700.Dsp.AudioBuffer;
            for (int i = 0; i < 2000; i++)
            {
                buffer.Enqueue((short)i);
                buffer.Enqueue((short)-i);
            } // 2000 frames - under AudioPlayer's 4096-frame per-pump cap

            player.Pump(session);

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
