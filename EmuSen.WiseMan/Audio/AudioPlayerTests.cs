using EmuSen.Common;
using EmuSen.Nehellania.Audio;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Audio
{
    // Exercises AudioPlayer's real SDL3 P/Invoke calls against SDL's "dummy"
    // audio driver - see EmuSen_Settings_Reference.md §4.10.
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
