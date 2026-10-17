using EmuSen.Galaxia.Native;

namespace EmuSen.Galaxia.Models
{
    // audio.json, which the frontends read for the audio device's and the cores' arguments - see EmuSen_Config_Reference.md §3.2.
    public class AudioConfig
    {
        public const int DefaultSampleRate = 32000;

        public int SampleRate { get; set; } = DefaultSampleRate;
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

        // Written with its defaults on the first run, so there is something to hand-edit.
        public static void SeedIfMissing()
        {
            if (!Exists) Load().Save();
        }

        // A copy with each number within the range the programs take; the file keeps what it holds.
        public AudioConfig Bounded()
        {
            var bounded = (AudioConfig)MemberwiseClone();
            bounded.SampleRate = System.Math.Clamp(SampleRate, 8000, 192000);
            bounded.AudioBufferMaxSamples = System.Math.Max(AudioBufferMaxSamples, 1024);
            bounded.OutputTargetLatencyMs = System.Math.Clamp(OutputTargetLatencyMs, 16, 2000);
            bounded.RateControlMaxDeviation = System.Math.Clamp(RateControlMaxDeviation, 0.0, 0.5);
            bounded.MasterVolume = System.Math.Clamp(MasterVolume, 0.0f, 1.0f);
            return bounded;
        }
    }
}
