using System.Diagnostics;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Rewind on Sapphire's 41 MiB state: the chain lands on the state it held, and P14's costs measured on demand - see Sapphire_Native.md §38.7.
    [Collection(TestCollections.ProcessGlobals)]
    public class SapphireRewindTests : IDisposable
    {
        private readonly ITestOutputHelper _output;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenSapphireRewind_" + Guid.NewGuid().ToString("N"));

        public SapphireRewindTests(ITestOutputHelper output)
        {
            _output = output;
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(true);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private CoreEngine Open()
        {
            string path = Path.Combine(_root, "program.elf");
            File.WriteAllBytes(path, SyntheticPs2Elf.Build());
            var core = Assert.IsType<CoreEngine>(CoreFactory.Load(path).Core);
            Assert.Equal("sapphire", core.Info.Id);
            return core;
        }

        private static byte[] Save(ICore core)
        {
            using var s = new MemoryStream();
            core.SaveState(s);
            return s.ToArray();
        }

        [Fact]
        public void Rewind_at_the_default_interval_lands_on_the_state_it_held()
        {
            using var core = Open();
            var rewind = new RewindBuffer { Enabled = true };
            for (int f = 0; f < 48; f++)
            {
                core.RunFrame();
                core.DequeueAudioSamples(8192);
                rewind.OnFrameCompleted(core);
            }
            Assert.InRange(rewind.SnapshotBytes, 41 << 20, 42 << 20);
            var moments = rewind.Moments();
            long target = moments[moments.Count / 2].Frame;
            byte[] expected = rewind.StateAt(target)!;
            Assert.True(rewind.RewindTo(core, target));
            Assert.True(expected.AsSpan().SequenceEqual(Save(core)), "the state after the rewind is the one held");
        }

        // Opt-in by EMUSEN_SAPPHIRE_P14: the capture, encode and restore of the chain on this machine, and the codec on states of which a share differs.
        [Fact]
        public void P14_the_chains_costs_on_a_41_MiB_state()
        {
            if (Environment.GetEnvironmentVariable("EMUSEN_SAPPHIRE_P14") is null)
            {
                _output.WriteLine("skipped: EMUSEN_SAPPHIRE_P14 asks for the measurement");
                return;
            }
            using var core = Open();
            const int Frames = 240;
            var watch = Stopwatch.StartNew();
            for (int f = 0; f < Frames; f++) { core.RunFrame(); core.DequeueAudioSamples(8192); }
            double plain = watch.Elapsed.TotalMilliseconds / Frames;
            var rewind = new RewindBuffer { Enabled = true };
            var captures = new List<double>();
            watch.Restart();
            for (int f = 0; f < Frames; f++)
            {
                core.RunFrame();
                core.DequeueAudioSamples(8192);
                var t = Stopwatch.GetTimestamp();
                if (rewind.OnFrameCompleted(core)) captures.Add(Stopwatch.GetElapsedTime(t).TotalMilliseconds);
            }
            double with = watch.Elapsed.TotalMilliseconds / Frames;
            var saves = Enumerable.Range(0, 8).Select(_ => { var t = Stopwatch.GetTimestamp(); Save(core); return Stopwatch.GetElapsedTime(t).TotalMilliseconds; }).ToList();
            byte[] a = Save(core);
            core.RunFrame();
            byte[] b = Save(core);
            var encodes = Enumerable.Range(0, 8).Select(_ => { var t = Stopwatch.GetTimestamp(); XorDeltaCodec.Encode(a, b); return Stopwatch.GetElapsedTime(t).TotalMilliseconds; }).ToList();
            var restores = new List<double>();
            for (int k = 0; k < 16 && rewind.Depth > 1; k++)
            {
                var t = Stopwatch.GetTimestamp();
                Assert.True(rewind.Rewind(core));
                restores.Add(Stopwatch.GetElapsedTime(t).TotalMilliseconds);
            }
            _output.WriteLine($"state {a.Length:N0} bytes; frame {plain:F2} ms without the chain, {with:F2} ms with it at {RewindBuffer.DefaultIntervalFrames} frames");
            _output.WriteLine($"capture on the emulation thread: median {Median(captures):F2} ms, max {captures.Max():F2} ms of {captures.Count}; SaveState alone median {Median(saves):F2} ms");
            _output.WriteLine($"encode of one frame apart (idle board): median {Median(encodes):F2} ms, {XorDeltaCodec.Encode(a, b).Length:N0} bytes; restore (Apply and LoadState): median {Median(restores):F2} ms, max {restores.Max():F2} ms of {restores.Count}");
            var rng = new Random(30);
            foreach (double share in new[] { 0.001, 0.01, 0.1, 0.5, 1.0 })
            {
                byte[] c = (byte[])a.Clone();
                int run = 64, step = (int)(run / share);
                for (int at = rng.Next(step); at + run <= c.Length; at += step)
                    for (int k = 0; k < run; k++) c[at + k] ^= 0x5A;
                var times = Enumerable.Range(0, 4).Select(_ => { var t = Stopwatch.GetTimestamp(); XorDeltaCodec.Encode(a, c); return Stopwatch.GetElapsedTime(t).TotalMilliseconds; }).ToList();
                byte[] delta = XorDeltaCodec.Encode(a, c);
                var t2 = Stopwatch.GetTimestamp();
                XorDeltaCodec.Apply(c, delta);
                double apply = Stopwatch.GetElapsedTime(t2).TotalMilliseconds;
                Assert.True(c.AsSpan().SequenceEqual(a));
                double reach = RewindBuffer.DefaultBudgetBytes / (double)Math.Max(1, delta.Length) * RewindBuffer.DefaultIntervalFrames / 59.94;
                _output.WriteLine($"{share:P1} of the state differing in runs of {run}: encode median {Median(times):F2} ms, {delta.Length:N0} bytes, apply {apply:F2} ms; {RewindBuffer.DefaultBudgetBytes >> 20} MiB holds {reach:N1} s");
            }
        }

        private static double Median(List<double> v)
        {
            var s = v.OrderBy(x => x).ToList();
            return s.Count == 0 ? 0 : s[s.Count / 2];
        }
    }
}
