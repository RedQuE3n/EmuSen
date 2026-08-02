using System;
using System.IO;
using EmuSen.Audio;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Graphics;

namespace EmuSen.WiseMan.Galaxia
{
    // The static settings hubs against their on-disk mirrors - see
    // EmuSen_Config_Reference.md §3.2 and §3.3. Both hubs are process-global
    // mutable state, so every value touched here is restored on the way out.
    public class SettingsPersistenceTests : IDisposable
    {
        private readonly string _dir;
        private readonly AudioConfig _audioBefore;
        private readonly GraphicsConfig _graphicsBefore;

        public SettingsPersistenceTests()
        {
            _audioBefore = CaptureAudio();
            _graphicsBefore = CaptureGraphics();
            _dir = Path.Combine(Path.GetTempPath(), "EmuSenSettings_" + Guid.NewGuid().ToString("N"));
            ConfigStore.OverrideDirectory = _dir;
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            RestoreAudio(_audioBefore);
            RestoreGraphics(_graphicsBefore);
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static AudioConfig CaptureAudio() => new()
        {
            SampleRate = AudioSettings.SampleRate,
            AudioBufferMaxSamples = AudioSettings.AudioBufferMaxSamples,
            OutputTargetLatencyMs = AudioSettings.OutputTargetLatencyMs,
            RateControlMaxDeviation = AudioSettings.RateControlMaxDeviation,
            MasterVolume = AudioSettings.MasterVolume,
            Muted = AudioSettings.Muted,
            AudioEnabled = AudioSettings.AudioEnabled,
        };

        private static void RestoreAudio(AudioConfig c)
        {
            AudioSettings.SampleRate = c.SampleRate;
            AudioSettings.AudioBufferMaxSamples = c.AudioBufferMaxSamples;
            AudioSettings.OutputTargetLatencyMs = c.OutputTargetLatencyMs;
            AudioSettings.RateControlMaxDeviation = c.RateControlMaxDeviation;
            AudioSettings.MasterVolume = c.MasterVolume;
            AudioSettings.Muted = c.Muted;
            AudioSettings.AudioEnabled = c.AudioEnabled;
        }

        private static GraphicsConfig CaptureGraphics() => new()
        {
            WindowWidth = GraphicsSettings.WindowWidth,
            WindowHeight = GraphicsSettings.WindowHeight,
            TargetFps = GraphicsSettings.TargetFps,
            VSyncEnabled = GraphicsSettings.VSyncEnabled,
            WindowResizable = GraphicsSettings.WindowResizable,
            BilinearFiltering = GraphicsSettings.BilinearFiltering,
        };

        private static void RestoreGraphics(GraphicsConfig c)
        {
            GraphicsSettings.WindowWidth = c.WindowWidth;
            GraphicsSettings.WindowHeight = c.WindowHeight;
            GraphicsSettings.TargetFps = c.TargetFps;
            GraphicsSettings.VSyncEnabled = c.VSyncEnabled;
            GraphicsSettings.WindowResizable = c.WindowResizable;
            GraphicsSettings.BilinearFiltering = c.BilinearFiltering;
        }

        [Fact]
        public void Audio_settings_round_trip_through_the_hub()
        {
            AudioSettings.MasterVolume = 0.25f;
            AudioSettings.Muted = true;
            AudioSettings.OutputTargetLatencyMs = 128;
            Assert.True(AudioSettings.SaveToDisk());

            AudioSettings.MasterVolume = 1.0f;
            AudioSettings.Muted = false;
            AudioSettings.OutputTargetLatencyMs = 256;
            AudioSettings.LoadFromDisk();

            Assert.Equal(0.25f, AudioSettings.MasterVolume);
            Assert.True(AudioSettings.Muted);
            Assert.Equal(128, AudioSettings.OutputTargetLatencyMs);
        }

        [Fact]
        public void Graphics_settings_round_trip_through_the_hub()
        {
            GraphicsSettings.WindowWidth = 1600;
            GraphicsSettings.BilinearFiltering = false;
            Assert.True(GraphicsSettings.SaveToDisk());

            GraphicsSettings.WindowWidth = 1060;
            GraphicsSettings.BilinearFiltering = true;
            GraphicsSettings.LoadFromDisk();

            Assert.Equal(1600, GraphicsSettings.WindowWidth);
            Assert.False(GraphicsSettings.BilinearFiltering);
        }

        // The file exists to be hand-edited, so a typed-in zero must not reach
        // OutputTargetFrames, which divides by SampleRate.
        [Fact]
        public void A_hand_edited_zero_sample_rate_is_clamped_rather_than_applied()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "audio.json"), """{"sampleRate":0,"masterVolume":9.0}""");

            AudioSettings.LoadFromDisk();

            Assert.True(AudioSettings.SampleRate >= 8000);
            Assert.Equal(1.0f, AudioSettings.MasterVolume);
            Assert.True(AudioSettings.OutputTargetFrames > 0);
        }

        [Fact]
        public void A_hand_edited_zero_window_size_is_clamped_rather_than_applied()
        {
            Directory.CreateDirectory(_dir);
            File.WriteAllText(Path.Combine(_dir, "graphics.json"), """{"windowWidth":0,"windowHeight":0,"targetFps":0}""");

            GraphicsSettings.LoadFromDisk();

            Assert.True(GraphicsSettings.WindowWidth >= 256);
            Assert.True(GraphicsSettings.WindowHeight >= 224);
            Assert.True(GraphicsSettings.TargetFps >= 1);
        }

        // Seeded on first run so there is something to open and edit; a file
        // that already exists is never rewritten behind the user's back.
        [Fact]
        public void First_run_seeds_both_files()
        {
            Assert.False(AudioConfig.Exists);
            Assert.False(GraphicsConfig.Exists);

            AudioSettings.LoadFromDisk();
            GraphicsSettings.LoadFromDisk();

            Assert.True(AudioConfig.Exists);
            Assert.True(GraphicsConfig.Exists);
        }

        [Fact]
        public void An_unreadable_file_is_left_on_disk_untouched()
        {
            Directory.CreateDirectory(_dir);
            string path = Path.Combine(_dir, "audio.json");
            File.WriteAllText(path, "{ not json at all");

            AudioSettings.LoadFromDisk();

            Assert.Equal("{ not json at all", File.ReadAllText(path));
        }
    }
}
