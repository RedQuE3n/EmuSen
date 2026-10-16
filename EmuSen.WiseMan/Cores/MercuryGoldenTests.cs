using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text.RegularExpressions;
using System.Threading.Tasks;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // Stage 9's goldens: every run recorded from C# Mercury, each engine graded against the recording frame by frame - see Mercury_Native.md §14.4.
    public class MercuryGoldenTests
    {
        // A directory of commercial Game Boy images, the goldens of which hold hashes only; absent, those cases pass without running.
        public const string GamesVariable = MercuryRtStateTests.RomsVariable;

        public const int GameFrames = 3600;

        private readonly ITestOutputHelper _output;

        public MercuryGoldenTests(ITestOutputHelper output)
        {
            _output = output;
            CoreOptions.BatteryRamDisabled = true;
        }

        // The synthetic programs, run by frames: busy and interrupts on every board, HDMA halting and running, four channels, and the small lock-step programs.
        public static IEnumerable<MercuryGoldenRun> FrameRuns()
        {
            foreach (var (k, r, c) in MercuryPrograms.Boards)
            {
                yield return new($"programs/busy-{k:X2}-{r:X2}-{c:X2}", () => SyntheticGbRom.Build(romBanks: 4, cartridgeType: k, ramSizeCode: r, cgbFlag: c, patches: (0, MercuryPrograms.Busy)), Frames: 600);
                yield return new($"programs/interrupts-{k:X2}-{r:X2}-{c:X2}", () => MercuryPrograms.InterruptsRom(k, r, c), Frames: 600);
            }
            yield return new("programs/interrupts-03-02-00-on-gbc", () => MercuryPrograms.InterruptsRom(0x03, 0x02, 0x00), GbModel.GameBoyColor, Frames: 600);
            yield return new("programs/interrupts-03-03-80-on-gb", () => MercuryPrograms.InterruptsRom(0x03, 0x03, 0x80), GbModel.GameBoy, Frames: 600);
            yield return new("programs/interrupts-03-02-80-skipped", () => MercuryPrograms.InterruptsRom(0x03, 0x02, 0x80), SkipRendering: true, Frames: 300);
            yield return new("programs/colour", () => MercuryPrograms.ColourRom(), Frames: 600);
            yield return new("programs/colour-running", () => MercuryPrograms.ColourRom(halt: false), Frames: 600);
            yield return new("programs/colour-skipped", () => MercuryPrograms.ColourRom(halt: false), SkipRendering: true, Frames: 300);
            yield return new("programs/four-channels", () => SyntheticGbRom.Build(patches: (0, MercuryPrograms.FourChannels)), Frames: 600);
            yield return new("programs/colour-priority", MercuryPrograms.ColourPriorityRom, Frames: 30);
            yield return new("programs/sweep-overflow", MercuryPrograms.SweepOverflowRom, Frames: 10);
            foreach (byte kind in new byte[] { 0x01, 0x0F, 0x19, 0x05 })
                yield return new($"programs/bank-zero-{kind:X2}", () => MercuryPrograms.BankZeroRom(kind), Frames: 2);
            yield return new("programs/mbc3-latch", MercuryPrograms.Mbc3LatchRom, Frames: 2);
        }

        // The programs compared instruction by instruction.
        public static IEnumerable<MercuryGoldenRun> StepRuns()
        {
            yield return new("steps/halt-bug", MercuryPrograms.HaltBugRom, Steps: 40);
            yield return new("steps/window", () => MercuryPrograms.InterruptsRom(0x03, 0x02, 0x00), SkipRendering: true, Steps: 20000);
            yield return new("steps/oam-dma", MercuryPrograms.OamDmaRom, Steps: 3000);
        }

        public static IEnumerable<MercuryGoldenRun> RandomRuns(byte kind, byte ram, byte cgb) =>
            Enumerable.Range(0, 16).Select(seed => new MercuryGoldenRun($"steps/random-{kind:X2}-{ram:X2}-{cgb:X2}-{seed:D2}", () => MercuryPrograms.RandomRom(kind, ram, cgb, seed), Steps: 5000));

        // Every corpus ROM on the console its header asks for, to its verdict or the baseline's cap, rendering skipped as the baseline runs it.
        public static IEnumerable<MercuryGoldenRun> CorpusRuns() => HardwareTestRomLibrary.Find().Select(path =>
            new MercuryGoldenRun("corpus/" + Path.GetRelativePath(HardwareTestRomLibrary.Root, path).Replace('\\', '/'), () => File.ReadAllBytes(path),
                SkipRendering: true, Frames: MercuryHardwareTestRomTests.FrameCap, Corpus: true));

        // §3.3's games from boot with the bench's input, named by their hash alone.
        public static IEnumerable<MercuryGoldenRun> GameRuns()
        {
            string? folder = Environment.GetEnvironmentVariable(GamesVariable);
            if (folder is null || !Directory.Exists(folder)) yield break;
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                yield return new($"games/{Convert.ToHexString(SHA256.HashData(rom))[..16]}", () => rom, Input: "bench", Frames: GameFrames);
            }
        }

        private static TheoryData<string, MercuryEngine> OnEngines(IEnumerable<string> names)
        {
            var data = new TheoryData<string, MercuryEngine>();
            foreach (string name in names)
                foreach (MercuryEngine engine in Enum.GetValues<MercuryEngine>()) data.Add(name, engine);
            return data;
        }

        public static TheoryData<string, MercuryEngine> FrameRunNames => OnEngines(FrameRuns().Concat(StepRuns()).Select(r => r.Name));

        public static TheoryData<byte, byte, byte, MercuryEngine> RandomBoards
        {
            get
            {
                var data = new TheoryData<byte, byte, byte, MercuryEngine>();
                foreach (var (k, r, c) in MercuryPrograms.Boards)
                    foreach (MercuryEngine engine in Enum.GetValues<MercuryEngine>()) data.Add(k, r, c, engine);
                return data;
            }
        }

        public static TheoryData<MercuryEngine> Engines => new() { MercuryEngine.Mercury, MercuryEngine.MercuryRt };

        [Theory]
        [MemberData(nameof(FrameRunNames))]
        public void A_synthetic_program_runs_as_its_golden_recorded(string name, MercuryEngine engine)
        {
            MercuryGoldenRun run = FrameRuns().Concat(StepRuns()).Single(r => r.Name == name);
            _output.WriteLine(Check(run, engine));
        }

        [Theory]
        [MemberData(nameof(RandomBoards))]
        public void Random_programs_step_as_their_goldens_recorded(byte kind, byte ram, byte cgb, MercuryEngine engine)
        {
            foreach (MercuryGoldenRun run in RandomRuns(kind, ram, cgb)) _output.WriteLine(Check(run, engine));
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void The_corpus_runs_as_its_goldens_recorded(MercuryEngine engine)
        {
            var runs = CorpusRuns().ToList();
            if (runs.Count == 0)
            {
                _output.WriteLine($"No corpus under {HardwareTestRomLibrary.Root}; not run.");
                return;
            }
            var failures = new ConcurrentBag<string>();
            Parallel.ForEach(runs, new ParallelOptions { MaxDegreeOfParallelism = Math.Max(1, Environment.ProcessorCount / 4) }, run =>
            {
                try { Check(run, engine); }
                catch (Exception e) { failures.Add($"{run.Name}: {e.Message}"); }
            });
            var recorded = Traces("corpus").Except(runs.Select(r => r.Name)).ToList();
            foreach (string orphan in recorded) failures.Add($"{orphan}: recorded, not in this corpus");
            Assert.True(failures.IsEmpty, $"{failures.Count} of {runs.Count} corpus runs differ from their goldens:\n" + string.Join("\n", failures.Order(StringComparer.Ordinal)));
            _output.WriteLine($"{runs.Count} corpus ROMs as recorded on {engine}");
        }

        [Theory]
        [MemberData(nameof(Engines))]
        public void The_games_run_as_their_goldens_recorded(MercuryEngine engine)
        {
            var runs = GameRuns().ToList();
            if (runs.Count == 0) _output.WriteLine($"{GamesVariable} unset, not run");
            foreach (MercuryGoldenRun run in runs) _output.WriteLine(Check(run, engine));
        }

        // Every trace in the tree is a run the catalogue names, so none is left ungraded; corpus and game traces are checked where their ROMs are.
        [Fact]
        public void Every_synthetic_trace_belongs_to_a_named_run()
        {
            var named = FrameRuns().Concat(StepRuns()).Concat(MercuryPrograms.Boards.SelectMany(b => RandomRuns(b.Kind, b.RamCode, b.Cgb))).Select(r => r.Name).ToHashSet();
            var present = Traces("programs").Concat(Traces("steps")).ToHashSet();
            Assert.Empty(present.Except(named));
            Assert.Empty(named.Except(present));
        }

        // A trace with one hash changed in its 300th frame fails, naming the frame and the part - the check that grading compares anything.
        [Theory]
        [InlineData("state", 1)]
        [InlineData("picture", 2)]
        [InlineData("sound", 3)]
        public void A_deliberately_broken_trace_is_caught(string part, int column)
        {
            MercuryGoldenRun run = FrameRuns().Single(r => r.Name == "programs/interrupts-03-03-80");
            var golden = MercuryGoldenTrace.Parse(File.ReadAllText(MercuryGolden.TracePath(run.Name)));
            string[] row = golden.Rows[299];
            row[column] = row[column][..15] + (row[column][15] == '0' ? '1' : '0');
            var broken = MercuryGoldenTrace.Parse(golden.Text);
            string? difference = broken.FirstDifference(MercuryGolden.Run(MercuryEngine.MercuryRt, run));
            Assert.Equal($"frame 300: the {part} differs", difference?.Split(" (")[0]);
            _output.WriteLine(difference);
        }

        private static IEnumerable<string> Traces(string under)
        {
            string dir = Path.Combine(MercuryGolden.Directory(), under);
            return Directory.Exists(dir)
                ? Directory.GetFiles(dir, "*.trace", SearchOption.AllDirectories).Select(p => Path.GetRelativePath(MercuryGolden.Directory(), p)[..^".trace".Length].Replace('\\', '/'))
                : Enumerable.Empty<string>();
        }

        // Records when asked and this is the recording engine; otherwise grades from boot, then from a recorded interval state where this machine keeps one.
        private string Check(MercuryGoldenRun run, MercuryEngine engine)
        {
            string path = MercuryGolden.TracePath(run.Name);
            if (MercuryGolden.Recording is { } recorder)
            {
                if (engine != recorder) return $"{run.Name}: recording from {recorder}, {engine} not graded";
                MercuryGoldenTrace trace = MercuryGolden.Run(engine, run, interval: (f, s) => MercuryGolden.SaveState(run.Name, f, s));
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllText(path, trace.Text);
                return $"{run.Name}: recorded from {engine}, {trace.Rows.Count} rows";
            }

            Assert.True(File.Exists(path), $"{run.Name} has no golden; record one with {MercuryGolden.RecordVariable}=Mercury.");
            var golden = MercuryGoldenTrace.Parse(File.ReadAllText(path));
            MercuryGoldenTrace now = MercuryGolden.Run(engine, run);
            if (golden.FirstDifference(now) is { } difference) Assert.Fail($"{run.Name} on {engine}: {difference}{Bisect(run, engine, difference)}");
            if (run.Steps > 0 || run.Corpus) return $"{run.Name}: {now.Rows.Count} rows as recorded";

            int resumeAt = MercuryGolden.IntervalFrames * Math.Max(1, run.Frames / 4 / MercuryGolden.IntervalFrames);
            if (resumeAt >= run.Frames || MercuryGolden.CachedState(run.Name, resumeAt) is not { } cached || cached.Frame != resumeAt)
                return $"{run.Name}: {now.Rows.Count} frames as recorded";
            MercuryGoldenTrace resumed = MercuryGolden.Run(engine, run, cached.State, resumeAt);
            var tail = MercuryGoldenTrace.Parse(golden.Text);
            tail.Rows.RemoveRange(0, resumeAt);
            if (tail.FirstDifference(resumed) is { } after) Assert.Fail($"{run.Name} on {engine}, resumed from the recorded state of frame {resumeAt}: {after}");
            return $"{run.Name}: {now.Rows.Count} frames as recorded, and again from the recorded state of frame {resumeAt}";
        }

        // The first recorded interval state at or after a failing frame, and the fields this engine's state differs from it in.
        private static string Bisect(MercuryGoldenRun run, MercuryEngine engine, string difference)
        {
            Match m = Regex.Match(difference, @"^frame (\d+):");
            if (!m.Success || MercuryGolden.CachedState(run.Name, int.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture)) is not { } cached) return "";
            byte[]? mine = null;
            MercuryGolden.Run(engine, run with { Frames = cached.Frame }, interval: (f, s) => { if (f == cached.Frame) mine = s; });
            if (mine is null || mine.Length != cached.State.Length) return $"; at frame {cached.Frame} the states differ in length";
            MercuryStateLayout layout;
            try { layout = MercuryStateLayout.For(run.Rom(), cached.State); }
            catch (InvalidDataException e) { return $"; at frame {cached.Frame} the states differ, and {e.Message}"; }
            var fields = layout.Differences(cached.State, mine).Take(12).ToList();
            return fields.Count == 0
                ? $"; at frame {cached.Frame} the state equals the recorded one field for field, so the trace moved and not the engine"
                : $"; at frame {cached.Frame}, against the recorded state: " + string.Join(", ", fields);
        }
    }
}
