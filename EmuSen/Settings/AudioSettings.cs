using System;

namespace EmuSen.Audio
{
    // Central hub for audio-related settings - see
    // EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Settings_Reference.md §2.
    public static class AudioSettings
    {
        public static int SampleRate = 32000;

        // Core-side safety valve only, not a resync - see EmuSen_Audio_Sync.md §4.
        public static int AudioBufferMaxSamples = 128000;

        // Where dynamic rate control steers the output queue - see EmuSen_Audio_Sync.md §3.
        public static int OutputTargetLatencyMs = 256;

        // Largest resample ratio departure from 1.0 - see EmuSen_Audio_Sync.md §3.
        public static double RateControlMaxDeviation = 0.005;

        public static float MasterVolume = 1.0f;
        public static bool Muted = false;
        public static bool AudioEnabled = true;

        public static int OutputTargetFrames => OutputTargetLatencyMs * SampleRate / 1000;

        // Applies etc/EmuSen/audio.json, seeding it on first run so there is
        // something to hand-edit - see EmuSen_Config_Reference.md §3.2. Values
        // are clamped because that file is meant to be edited by hand and a
        // zero sample rate divides by zero in OutputTargetFrames.
        public static void LoadFromDisk()
        {
            bool seed = !Galaxia.Models.AudioConfig.Exists;
            var config = Galaxia.Models.AudioConfig.Load();

            SampleRate = Math.Clamp(config.SampleRate, 8000, 192000);
            AudioBufferMaxSamples = Math.Max(config.AudioBufferMaxSamples, 1024);
            OutputTargetLatencyMs = Math.Clamp(config.OutputTargetLatencyMs, 16, 2000);
            RateControlMaxDeviation = Math.Clamp(config.RateControlMaxDeviation, 0.0, 0.5);
            MasterVolume = Math.Clamp(config.MasterVolume, 0.0f, 1.0f);
            Muted = config.Muted;
            AudioEnabled = config.AudioEnabled;

            if (seed) SaveToDisk();
        }

        public static bool SaveToDisk() => new Galaxia.Models.AudioConfig
        {
            SampleRate = SampleRate,
            AudioBufferMaxSamples = AudioBufferMaxSamples,
            OutputTargetLatencyMs = OutputTargetLatencyMs,
            RateControlMaxDeviation = RateControlMaxDeviation,
            MasterVolume = MasterVolume,
            Muted = Muted,
            AudioEnabled = AudioEnabled,
        }.Save();
    }
}
