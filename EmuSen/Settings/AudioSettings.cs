namespace EmuSen.Audio
{
    // Central hub for audio-related settings - see
    // Man pages/EmuSen_Settings_Reference.md §2.
    public static class AudioSettings
    {
        public static int SampleRate = 32000;
        public static int AudioBufferMaxSamples = 64000;

        public static float MasterVolume = 1.0f;
        public static bool Muted = false;
        public static bool AudioEnabled = true;
    }
}
