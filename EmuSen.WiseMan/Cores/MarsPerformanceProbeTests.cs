using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // How fast each game runs through a frontend's bundle, and whether it still produces what it did - see Mars_Performance.md §1.
    [Collection(TestCollections.ProcessGlobals)]
    public class MarsPerformanceProbeTests : IDisposable
    {
        public const string DirectoryVariable = "EMUSEN_MARS_PERF";
        public const string RecordVariable = "EMUSEN_MARS_PERF_RECORD";
        public const string ThreadedVariable = "EMUSEN_MARS_PERF_THREADED";
        public const string WorkersVariable = "EMUSEN_MARS_PERF_WORKERS";
        public const int Frames = 600;

        // Two games are the baseline, whatever else the library holds; the variable names others, or "all" - see Mars_Performance.md §40.
        public const string GamesVariable = "EMUSEN_MARS_PERF_GAMES";
        private static readonly string[] Baseline = { "Super Mario 64", "Ocarina of Time" };

        private static bool Graded(string rom)
        {
            string? asked = Environment.GetEnvironmentVariable(GamesVariable);
            if (string.Equals(asked, "all", StringComparison.OrdinalIgnoreCase)) return true;
            string[] wanted = string.IsNullOrWhiteSpace(asked) ? Baseline : asked.Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries);
            string name = Path.GetFileNameWithoutExtension(rom);
            return wanted.Any(w => name.Contains(w, StringComparison.OrdinalIgnoreCase));
        }

        // Derived from commercial games, so kept beside the other probe output rather than in the repository.
        public static string BaselineDirectory =>
            Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".cache", "emusen", "mars-golden");

        private readonly string _home = Path.Combine(Path.GetTempPath(), "EmuSenMarsPerf_" + Guid.NewGuid().ToString("N"));

        public MarsPerformanceProbeTests() => DataStore.OverrideDirectory = _home;

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            if (Directory.Exists(_home)) Directory.Delete(_home, recursive: true);
        }

        [Fact]
        public void Each_game_produces_what_it_did_and_the_report_says_how_fast()
        {
            string? directory = Environment.GetEnvironmentVariable(DirectoryVariable);
            if (string.IsNullOrWhiteSpace(directory)) return;

            bool record = Environment.GetEnvironmentVariable(RecordVariable) == "1";
            Directory.CreateDirectory(directory);
            Directory.CreateDirectory(BaselineDirectory);

            var report = new List<string>();
            var failures = new List<string>();

            foreach (string rom in N64TestRomLibrary.Find().Where(p => !p.EndsWith(N64TestRomLibrary.SystemTestRomName)).Where(Graded))
            {
                string game = Path.GetFileNameWithoutExtension(rom);
                (double seconds, List<string> hashes) = Run(rom);

                report.Add($"{game}: {Frames} frames in {seconds:F2}s of RunFrame = {Frames / seconds:F2} fps, {1000 * seconds / Frames:F2} ms a frame");

                string baseline = Path.Combine(BaselineDirectory, game + ".txt");
                if (record || !File.Exists(baseline))
                {
                    File.WriteAllLines(baseline, hashes);
                    report.Add($"  baseline {(record ? "recorded" : "created")}: {baseline}");
                    continue;
                }

                string[] expected = File.ReadAllLines(baseline);
                int first = Enumerable.Range(0, Math.Min(expected.Length, hashes.Count)).FirstOrDefault(i => expected[i] != hashes[i], -1);
                if (first >= 0) failures.Add($"{game}: frame {first} differs: expected [{expected[first]}] got [{hashes[first]}]");
                report.Add(first < 0 ? $"  identical to the baseline for all {Frames} frames" : $"  DIFFERS from frame {first}");
            }

            File.WriteAllLines(Path.Combine(directory, "mars-performance.txt"), report);
            Assert.Empty(failures);
        }

        // RunFrame alone is timed; the hashing after each frame is not.
        private static (double Seconds, List<string> Hashes) Run(string rom)
        {
            var core = (MarsCore)CoreFactory.Load(rom, headless: true).Core;
            core.ThreadedRdp = Environment.GetEnvironmentVariable(ThreadedVariable) == "1";
            if (int.TryParse(Environment.GetEnvironmentVariable(WorkersVariable), out int workers)) core.RdpWorkers = workers;
            var hashes = new List<string>(Frames);
            var clock = new Stopwatch();

            for (int frame = 0; frame < Frames; frame++)
            {
                clock.Start();
                core.RunFrame();
                clock.Stop();

                short[] audio = core.DequeueAudioSamples(int.MaxValue);
                byte[] sound = new byte[audio.Length * 2];
                Buffer.BlockCopy(audio, 0, sound, 0, sound.Length);

                // The whole machine as a state carries it, so a divergence shows before it reaches RDRAM - see Mars_Performance.md §1.
                using var state = new MemoryStream();
                core.SaveState(state);

                hashes.Add($"{frame} {Hash(core.GetFrameBufferRgba())} {Hash(core.Bus!.Rdram)} {Hash(sound)} {core.Bus.Cycles} {Hash(state.ToArray())}");
            }

            return (clock.Elapsed.TotalSeconds, hashes);
        }

        private static string Hash(byte[] bytes) => Convert.ToHexString(SHA256.HashData(bytes).AsSpan(0, 8));
    }
}
