namespace EmuSen.Audio
{
    // Central hub for audio-related settings. SDsp.cs now generates real
    // audio (BRR-decoded voices, ADSR/GAIN envelopes, MVOL-scaled mixing -
    // see Dsp/DspVoice.cs), though echo/noise/pitch-modulation aren't
    // implemented yet. Nothing plays AudioBuffer's contents to an actual
    // output device yet - that's the remaining piece for audible sound.
    public static class AudioSettings
    {
        // DSP output rate in Hz - real hardware fact, not really a "preference",
        // but it's a magic number worth naming rather than leaving as a bare 32000
        // (or the derived "32 cycles per sample" at the SPC700's ~1.024MHz clock)
        // scattered in SDsp.cs.
        public static int SampleRate = 32000;

        // Caps AudioBuffer's size (in samples, not sample-pairs) so it can't grow
        // unbounded if a future output device consumer falls behind. 64000 =
        // ~1 second of 32kHz stereo audio.
        public static int AudioBufferMaxSamples = 64000;

        // --- Now wired into SDsp.GenerateSample ---

        // Overall gain applied to the final mixed stereo sample.
        public static float MasterVolume = 1.0f;

        // Silences output without needing to touch MasterVolume. Voice
        // playback/envelope state still advances while muted, so audio
        // doesn't jump ahead the moment this is turned back off.
        public static bool Muted = false;

        // Master on/off for audio generation entirely.
        public static bool AudioEnabled = true;
    }
}
