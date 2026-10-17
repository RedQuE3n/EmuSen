using System;
using System.IO;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Galaxia
{
    // audio.json and graphics.json through their models, their ranges and the first run's seed; serial for the config override - see EmuSen_Config_Reference.md §3.2, §3.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class SettingsPersistenceTests : IDisposable
    {
        private readonly string _dir;

        public SettingsPersistenceTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenSettings_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        [Fact]
        public void Audio_settings_round_trip_through_the_file()
        {
            Assert.True(new AudioConfig { MasterVolume = 0.25f, Muted = true, OutputTargetLatencyMs = 128 }.Save());

            AudioConfig read = AudioConfig.Load();

            Assert.Equal(0.25f, read.MasterVolume);
            Assert.True(read.Muted);
            Assert.Equal(128, read.OutputTargetLatencyMs);
        }

        // A console's settings by key, in their text form, survive a save and a load beside the window fields - see EmuSen_Config_Reference.md §3.3.
        [Fact]
        public void A_consoles_settings_round_trip_through_the_graphics_file()
        {
            var written = new GraphicsConfig();
            written.SetValue("N64", "RdpWorkers", "3");
            written.SetValue("N64", "ThreadedRdp", "false");
            written.SetValue("SNES", "Anything", "text");
            Assert.True(written.Save());

            GraphicsConfig read = GraphicsConfig.Load();
            Assert.Equal("3", read.Value("N64", "RdpWorkers"));
            Assert.Equal("false", read.Value("N64", "ThreadedRdp"));
            Assert.Equal("text", read.Value("SNES", "Anything"));
            Assert.Null(read.Value("N64", "DeferredPresentation"));
            Assert.Null(read.Value("NES", "RdpWorkers"));

            read.Forget("N64");
            Assert.Null(read.Value("N64", "RdpWorkers"));
            Assert.Equal("text", read.Value("SNES", "Anything"));
        }

        [Fact]
        public void Graphics_settings_round_trip_through_the_file()
        {
            Assert.True(new GraphicsConfig { WindowWidth = 1600, BilinearFiltering = false, SyncToDisplay = false }.Save());

            GraphicsConfig read = GraphicsConfig.Load();

            Assert.Equal(1600, read.WindowWidth);
            Assert.False(read.BilinearFiltering);
            Assert.False(read.SyncToDisplay);
        }

        // A typed-in zero must not reach the audio device, whose target is latency times rate; the file keeps what was typed.
        [Fact]
        public void A_hand_edited_zero_sample_rate_is_clamped_rather_than_applied()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "audio.json"), """{"sampleRate":0,"masterVolume":9.0}""");

            AudioConfig bounded = AudioConfig.Load().Bounded();

            Assert.True(bounded.SampleRate >= 8000);
            Assert.Equal(1.0f, bounded.MasterVolume);
            Assert.True(bounded.OutputTargetLatencyMs * bounded.SampleRate / 1000 > 0);
            Assert.Contains("\"sampleRate\":0", File.ReadAllText(Path.Combine(_dir, "audio.json")));
        }

        [Fact]
        public void A_hand_edited_zero_window_size_is_clamped_rather_than_applied()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "graphics.json"), """{"windowWidth":0,"windowHeight":0,"targetFps":0}""");

            (int width, int height) = GraphicsConfig.Load().WindowSize();

            Assert.True(width >= 256);
            Assert.True(height >= 224);
        }

        // Seeded on first run so there is something to open and edit; a file
        // that already exists is never rewritten behind the player's back.
        [Fact]
        public void First_run_seeds_both_files()
        {
            Assert.False(AudioConfig.Exists);
            Assert.False(GraphicsConfig.Exists);

            AudioConfig.SeedIfMissing();
            GraphicsConfig.SeedIfMissing();

            Assert.True(AudioConfig.Exists);
            Assert.True(GraphicsConfig.Exists);
            Assert.Equal(new AudioConfig().OutputTargetLatencyMs, AudioConfig.Load().OutputTargetLatencyMs);
            Assert.True(GraphicsConfig.Load().SyncToDisplay);
        }

        [Fact]
        public void A_seed_leaves_an_existing_file_as_it_is()
        {
            Assert.True(new AudioConfig { OutputTargetLatencyMs = 99 }.Save());

            AudioConfig.SeedIfMissing();

            Assert.Equal(99, AudioConfig.Load().OutputTargetLatencyMs);
        }

        [Fact]
        public void An_unreadable_file_is_left_on_disk_untouched()
        {
            Directory.CreateDirectory(_dir);
            string path = Path.Combine(_dir, "audio.json");
            File.WriteAllText(path, "{ not json at all");

            AudioConfig.SeedIfMissing();
            AudioConfig.Load();

            Assert.Equal("{ not json at all", File.ReadAllText(path));
        }
    }
}
