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
    }
}
