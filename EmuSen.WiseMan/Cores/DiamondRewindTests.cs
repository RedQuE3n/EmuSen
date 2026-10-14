using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Rewind on Diamond with every enhancement on at 2x, as Mistress keeps it: its reach under the default budget, and a rewind landing on the state it held - see Diamond_Enhancements.md §52.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiamondRewindTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiamondRewind_" + Guid.NewGuid().ToString("N"));

        public DiamondRewindTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // Mistress's rate, two snapshots a second (MainWindow.RewindSnapshotsPerSecond), at the PlayStation's 60 Hz.
        private const int Interval = 30;

        // The bench's press scripts, as Diamond's disc gates read them: (frame, buttons, frames held).
        private static List<(int At, PadButton[] Buttons, int Held)> Presses(string script)
        {
            string path = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "Reference", "analysis", "psxplay", script);
            string[] lines = File.ReadAllLines(path);
            int anchor = int.Parse(lines.First(l => l.StartsWith("# T: frame ")).Substring(11).Split(' ')[0]);
            var presses = new List<(int, PadButton[], int)>();
            foreach (string raw in lines)
            {
                string line = raw.Split('#')[0].Trim();
                if (line.Length == 0) continue;
                string[] f = line.Split(' ', StringSplitOptions.RemoveEmptyEntries);
                int at = f[0][0] == '+' ? anchor + int.Parse(f[0][1..]) : int.Parse(f[0][1..]);
                var buttons = f[1].Split('+').Where(b => !b.StartsWith("stick:")).Select(b => b switch
                {
                    "Cross" => PadButton.B, "Circle" => PadButton.A, "Triangle" => PadButton.X, "Square" => PadButton.Y, "L1" => PadButton.L, "R1" => PadButton.R,
                    _ => Enum.Parse<PadButton>(b),
                }).ToArray();
                presses.Add((at, buttons, int.Parse(f[2])));
            }
            return presses;
        }

        // Opt-in by EMUSEN_DIAMOND_DISCS, the folder of the bench's discs; the reach is printed, before and after the seeded state measured by it.
        [Theory]
        [InlineData("Spyro the Dragon (USA).bin", "spyro-the-dragon.txt", 4000, 7000)]
        [InlineData("Ape Escape (USA).cue", "ape-escape.txt", 3000, 6000)]
        [InlineData("Crash Bandicoot (USA).bin", "crash-bandicoot.txt", 3000, 6000)]
        public void Rewind_with_every_enhancement_on_at_2x_reaches_back_and_lands_on_its_state(string disc, string script, int from, int to)
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_DIAMOND_DISCS") is not { } folder || !File.Exists(Path.Combine(folder, disc)))
            {
                _output.WriteLine("skipped: EMUSEN_DIAMOND_DISCS names the folder of the bench's discs");
                return;
            }
            using var core = (CoreEngine)CoreFactory.Load(Path.Combine(folder, disc)).Core;
            Assert.Equal("diamond", core.Info.Id);
            foreach (var (key, value) in new[] { ("precision", "true"), ("resolution", "2"), ("colour", "true"), ("filtering", "all") }) core.Set(key, value);
            var presses = Presses(script);
            var rewind = new RewindBuffer { Enabled = true, IntervalFrames = Interval };
            var held = new HashSet<PadButton>();
            for (int f = 0; f < to; f++)
            {
                var now = presses.Where(p => f >= p.At && f < p.At + p.Held).SelectMany(p => p.Buttons).ToHashSet();
                foreach (var b in held.Except(now)) core.SetButton(0, b, false);
                foreach (var b in now.Except(held)) core.SetButton(0, b, true);
                held = now;
                core.RunFrame();
                core.DequeueAudioSamples(8192);
                if (f == from) rewind.CaptureNow(core);
                else if (f > from) rewind.OnFrameCompleted(core);
            }
            var moments = rewind.Moments();
            int depth = rewind.Depth;
            long deltas = rewind.BufferedBytes - rewind.SnapshotBytes;
            double perSnapshot = (double)deltas / depth;
            double reach = rewind.BudgetBytes / perSnapshot * Interval / 60.0;
            _output.WriteLine($"{disc}: state {rewind.SnapshotBytes:N0} bytes; {depth} deltas of {from}-{to}, {perSnapshot:N0} bytes each; reach under {rewind.BudgetBytes >> 20} MiB {reach:N1} s, held {rewind.BufferedSeconds(60):N1} s");
            Assert.True(depth > 0);

            long target = moments[moments.Count / 2].Frame;
            byte[] expected = rewind.StateAt(target)!;
            Assert.True(rewind.RewindTo(core, target));
            using var s = new MemoryStream();
            core.SaveState(s);
            Assert.True(expected.AsSpan().SequenceEqual(s.ToArray()), "the state after the rewind is the one held");
        }
    }
}
