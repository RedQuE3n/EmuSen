using System;
using System.Collections.Concurrent;
using System.Collections.Generic;

namespace EmuSen.Mistress.BigPicture
{
    // Mistress's own navigation sounds, synthesised, for any of the seven a theme does not provide - see EmuSen_BigPicture.md §29.
    public static class NavigationSounds
    {
        public const string Prefix = "emusen-builtin:";
        public const int Rate = 48000;

        // One tone of a sound: its start and end pitch, when it starts and how long it lasts, in seconds.
        private readonly record struct Tone(double From, double To, double Start, double Length);

        private static readonly IReadOnlyDictionary<string, Tone[]> Designs = new Dictionary<string, Tone[]>(StringComparer.Ordinal)
        {
            ["scroll"] = [new(1500, 1500, 0, 0.03)],
            ["systembrowse"] = [new(520, 780, 0, 0.07)],
            ["quicksysselect"] = [new(660, 660, 0, 0.035), new(990, 990, 0.04, 0.035)],
            ["select"] = [new(700, 1400, 0, 0.09)],
            ["back"] = [new(1400, 700, 0, 0.09)],
            ["favorite"] = [new(880, 880, 0, 0.05), new(1109, 1109, 0.05, 0.05), new(1319, 1319, 0.1, 0.08)],
            ["launch"] = [new(330, 1320, 0, 0.25)],
        };

        public static IEnumerable<string> Names => Designs.Keys;

        public static string Key(string name) => Prefix + name;

        public static bool IsBuiltIn(string path) => path.StartsWith(Prefix, StringComparison.Ordinal);

        private static readonly ConcurrentDictionary<string, byte[]> Made = new(StringComparer.Ordinal);

        // A sound as 48 kHz stereo 32-bit float, UiSoundPlayer's format; empty for a name that is not one of the seven.
        public static byte[] Samples(string keyOrName)
        {
            string name = IsBuiltIn(keyOrName) ? keyOrName[Prefix.Length..] : keyOrName;
            return Designs.TryGetValue(name, out Tone[]? tones) ? Made.GetOrAdd(name, _ => Render(tones)) : [];
        }

        // Each tone a sine with a quarter of its octave, a 3 ms rise and an exponential fall, the pitch glided linearly.
        private static byte[] Render(Tone[] tones)
        {
            double end = 0;
            foreach (Tone t in tones) end = Math.Max(end, t.Start + t.Length);
            int frames = (int)Math.Ceiling(end * Rate);
            var mix = new float[frames];
            foreach (Tone t in tones)
            {
                int first = (int)(t.Start * Rate), count = (int)(t.Length * Rate);
                double phase = 0;
                for (int i = 0; i < count && first + i < frames; i++)
                {
                    double at = (double)i / Rate, hz = t.From + (t.To - t.From) * i / count;
                    phase += 2 * Math.PI * hz / Rate;
                    double envelope = Math.Min(1, at / 0.003) * Math.Exp(-4 * at / t.Length);
                    mix[first + i] += (float)(0.3 * envelope * (Math.Sin(phase) + 0.25 * Math.Sin(2 * phase)));
                }
            }

            var bytes = new byte[frames * 8];
            for (int i = 0; i < frames; i++)
            {
                BitConverter.TryWriteBytes(bytes.AsSpan(i * 8), mix[i]);
                BitConverter.TryWriteBytes(bytes.AsSpan(i * 8 + 4), mix[i]);
            }
            return bytes;
        }
    }
}
