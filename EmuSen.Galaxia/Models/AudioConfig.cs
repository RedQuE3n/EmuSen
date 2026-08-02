namespace EmuSen.Galaxia.Models
{
    // On-disk mirror of EmuSen.Audio.AudioSettings - see EmuSen_Config_Reference.md §3.2.
    // A separate type rather than the hub itself because the hub is a static
    // class, which System.Text.Json cannot serialize; AudioSettings owns the
    // mapping in both directions.
    public class AudioConfig
    {
        public int SampleRate { get; set; } = 32000;
        public int AudioBufferMaxSamples { get; set; } = 128000;
        public int OutputTargetLatencyMs { get; set; } = 256;
        public double RateControlMaxDeviation { get; set; } = 0.005;
        public float MasterVolume { get; set; } = 1.0f;
        public bool Muted { get; set; } = false;
        public bool AudioEnabled { get; set; } = true;

        private static readonly ConfigFile<AudioConfig> File = new("audio.json");

        public bool Save() => File.Save(this);

        public static AudioConfig Load() => File.Load(() => new AudioConfig());

        public static bool Exists => File.Exists;
    }
}
