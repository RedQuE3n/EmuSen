using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Models
{
    // On-disk mirror of EmuSen.Audio.AudioSettings, a static hub that cannot be serialized itself - see EmuSen_Config_Reference.md §3.2.
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

        public static AudioConfig Load() =>
            GalaxiaNative.Active ? File.LoadNative(upgrade: false) ?? ConfigFile<AudioConfig>.NewNative(upgrade: false) : File.Load(() => new AudioConfig());

        public static bool Exists => File.Exists;
    }
}
