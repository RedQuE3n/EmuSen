using System.Collections.Concurrent;
using System.Globalization;
using System.Security.Cryptography;
using System.Text.RegularExpressions;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT's goldens, recorded from MoonRT itself and graded frame by frame and instruction by instruction - see Moon_Native.md §9.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class MoonGoldenTests
    {
        // The corpus directory (christopherpow/nes-test-roms); absent, its case passes unrun.
        public const string CorpusVariable = "EMUSEN_MOONRT_CORPUS";

        // A folder of NES images, or "library" for AppSettings.RomDirectory; the games are found there by hash, and absent, their case passes unrun.
        public const string GamesVariable = "EMUSEN_MOONRT_ROMS";

        public const int GameFrames = 3600;

        // The games, by the first sixteen digits of their SHA-256 and their size, with the board each brings.
        public static readonly (string Sha, long Size, string Board)[] Games =
        {
            ("F61548FDF1670CFF", 40976, "NROM"),
            ("8F72DC2E98572EB4", 131088, "MMC1"),
            ("27B5A635DF33ED57", 262160, "MMC1"),
            ("55BCAB0D9EFC33A9", 131088, "UxROM"),
            ("6EA0777CA520BA7A", 393232, "MMC3"),
            ("102309C351308638", 786448, "MMC3"),
            ("0435F6D9BCFEB474", 262160, "MMC2"),
        };

        private readonly ITestOutputHelper _output;

        public MoonGoldenTests(ITestOutputHelper output)
        {
            _output = output;

        }

        private static IEnumerable<(int Mapper, int Chr)> BoardList => MoonPrograms.Boards.Select(row => ((int)row[0], (int)row[1]));

        // The synthetic programs by frames: the busy loop and the board program on every board, RAMBO-1's cycle IRQ, the mask cycle, rendering skipped, and two resets.
        public static IEnumerable<MoonGoldenRun> FrameRuns()
        {
            foreach (var (m, c) in BoardList)
            {
                yield return new($"programs/busy-{m:D2}-{c}", () => MoonPrograms.Rom(m, c, (0, MoonPrograms.Busy)), Frames: 600);
                yield return new($"programs/board-{m:D2}-{c}", () => BoardPrograms.Build(m, c, cycleIrq: false), Input: "pads", Frames: 600);
            }
            yield return new("programs/rambo1-cycle-irq", () => BoardPrograms.Build(64, 8, cycleIrq: true), Input: "pads", Frames: 600);
            yield return new("programs/mask-cycle", () => BoardPrograms.Build(4, 8, cycleIrq: false, maskCycle: true), Input: "pads", Frames: 600);
            yield return new("programs/board-04-8-skipped", () => BoardPrograms.Build(4, 8, cycleIrq: false), Input: "pads", SkipRendering: true, Frames: 300);
            yield return new("programs/resets-01-0", () => BoardPrograms.Build(1, 0, cycleIrq: false), Input: "pads", Frames: 300, Resets: new[] { 100, 200 });
        }

        // The board program instruction by instruction, so where each IRQ, NMI and DMA lands inside a frame is graded.
        public static IEnumerable<MoonGoldenRun> StepRuns() =>
            new[] { 4, 9, 64, 69 }.Select(m => new MoonGoldenRun($"steps/board-{m:D2}", () => BoardPrograms.Build(m, 8, cycleIrq: false), Steps: 60000));

        // Random bytes as code on a board, eight seeds; from the fifth seed on the JAMs are NOPs, so the programs run longer.
        public static IEnumerable<MoonGoldenRun> RandomRuns(int mapper, int chr) => Enumerable.Range(0, 8).Select(seed => new MoonGoldenRun(
            $"steps/random-{mapper:D2}-{chr}-{seed}", () => RandomRom(mapper, chr, seed), Steps: 4000));

        public static byte[] RandomRom(int mapper, int chr, int seed)
        {
            var noise = new Random(seed * 131 + mapper * 7 + chr);
            byte[] rom = MoonPrograms.Rom(mapper, chr);
            noise.NextBytes(rom.AsSpan(16, rom.Length - 16));
            if (seed >= 4)
                for (int i = 16; i < 16 + 8 * SyntheticNesRom.PrgBankSize; i++)
                    if ((rom[i] & 0x0F) == 0x02 && (rom[i] >> 4) is not (8 or 0xA or 0xC or 0xE)) rom[i] = 0xEA;
            return rom;
        }

        public static string? CorpusRoot => Environment.GetEnvironmentVariable(CorpusVariable) is { Length: > 0 } dir && Directory.Exists(dir) ? dir : null;

        // Every corpus ROM by blargg's protocol to its verdict or 2,400 frames, rendering skipped as the runner skips it.
        public static IEnumerable<MoonGoldenRun> CorpusRuns() => CorpusRoot is not { } root ? Enumerable.Empty<MoonGoldenRun>()
            : Directory.GetFiles(root, "*.nes", SearchOption.AllDirectories).Order(StringComparer.Ordinal).Select(path =>
                new MoonGoldenRun("corpus/" + Path.GetRelativePath(root, path).Replace('\\', '/'), () => File.ReadAllBytes(path), SkipRendering: true, Frames: MoonGolden.CorpusFrames, Corpus: true));

        private static string? GamesRoot => Environment.GetEnvironmentVariable(GamesVariable) switch
        {
            null or "" => null,
            "library" => AppSettings.Load().RomDirectory,
            var dir => dir,
        };

        // One of the catalogue's games by its hash prefix, or null where the folder lacks it.
        public static byte[]? Game(string sha)
        {
            if (GamesRoot is not { } root || !Directory.Exists(root) || Games.FirstOrDefault(g => g.Sha == sha) is not { Size: > 0 } game) return null;
            return Directory.EnumerateFiles(root, "*.nes", SearchOption.AllDirectories).Where(f => new FileInfo(f).Length == game.Size).Select(File.ReadAllBytes)
                .FirstOrDefault(b => Convert.ToHexString(SHA256.HashData(b)).StartsWith(sha, StringComparison.Ordinal));
        }

        // The catalogue's games found under the folder by size, then hash; named by hash alone.
        public static IEnumerable<MoonGoldenRun> GameRuns()
        {
            if (GamesRoot is not { } root || !Directory.Exists(root)) yield break;
            var files = Directory.EnumerateFiles(root, "*.nes", SearchOption.AllDirectories).Select(f => new FileInfo(f)).ToList();
            foreach (var (sha, size, _) in Games)
            {
                byte[]? rom = files.Where(f => f.Length == size).Select(f => File.ReadAllBytes(f.FullName))
                    .FirstOrDefault(b => Convert.ToHexString(SHA256.HashData(b)).StartsWith(sha, StringComparison.Ordinal));
                if (rom is not null) yield return new($"games/{sha}", () => rom, Input: "bench", Frames: GameFrames);
            }
        }

        public static TheoryData<string> NamedRuns
        {
            get
            {
                var data = new TheoryData<string>();
                foreach (var run in FrameRuns().Concat(StepRuns())) data.Add(run.Name);
                return data;
            }
        }

        public static TheoryData<int, int> Boards => MoonPrograms.Boards;

        [Theory]
        [MemberData(nameof(NamedRuns))]
        public void A_synthetic_program_runs_as_its_golden_recorded(string name)
        {
            MoonGoldenRun run = FrameRuns().Concat(StepRuns()).Single(r => r.Name == name);
            _output.WriteLine(Check(run));
        }

        [Theory]
        [MemberData(nameof(Boards))]
        public void Random_programs_step_as_their_goldens_recorded(int mapper, int chr)
        {
            foreach (MoonGoldenRun run in RandomRuns(mapper, chr)) _output.WriteLine(Check(run));
        }

        [Fact]
        public void The_corpus_runs_as_its_goldens_recorded()
        {
            var runs = CorpusRuns().ToList();
            if (runs.Count == 0)
            {
                _output.WriteLine($"{CorpusVariable} unset, not run");
                return;
            }
            var failures = new ConcurrentBag<string>();
            Parallel.ForEach(runs, new ParallelOptions { MaxDegreeOfParallelism = Math.Max(1, Environment.ProcessorCount / 4) }, run =>
            {
                try { Check(run); }
                catch (Exception e) { failures.Add($"{run.Name}: {e.Message}"); }
            });
            foreach (string orphan in Traces("corpus").Except(runs.Select(r => r.Name))) failures.Add($"{orphan}: recorded, not in this corpus");
            Assert.True(failures.IsEmpty, $"{failures.Count} of {runs.Count} corpus runs differ from their goldens:\n" + string.Join("\n", failures.Order(StringComparer.Ordinal)));
            _output.WriteLine($"{runs.Count} corpus ROMs as recorded");
        }

        [Fact]
        public void The_games_run_as_their_goldens_recorded()
        {
            var runs = GameRuns().ToList();
            if (runs.Count == 0) _output.WriteLine($"{GamesVariable} unset or holding none of the games, not run");
            foreach (MoonGoldenRun run in runs) _output.WriteLine(Check(run));
        }

        // The corpus's verdicts, read from the committed traces, are never worse than the C# core's last recorded baseline - see Moon_Native.md §9.3.
        [Fact]
        public void No_corpus_verdict_is_worse_than_the_csharp_baseline()
        {
            var baseline = MoonCorpusBaseline.Read();
            Assert.NotEmpty(baseline);
            var worse = new List<string>();
            foreach (var row in baseline)
            {
                string path = MoonGolden.TracePath("corpus/" + row.Rom);
                Assert.True(File.Exists(path), $"{row.Rom} is in the baseline and has no golden");
                var trace = GoldenTrace.Parse(File.ReadAllText(path));
                string verdict = trace.Transcript["verdict"].Split(' ')[0];
                Assert.Equal(row.MoonRt, verdict);
                if (MoonCorpusBaseline.Rank(verdict) < MoonCorpusBaseline.Rank(row.Csharp)) worse.Add($"{row.Rom}: C# {row.Csharp}, MoonRT {verdict}");
            }
            Assert.True(worse.Count == 0, string.Join("\n", worse));
            _output.WriteLine($"{baseline.Count} ROMs: {baseline.Count(r => r.MoonRt == "Passed")} pass on MoonRT, {baseline.Count(r => r.Csharp == "Passed")} passed on the C# core");
        }

        // Every trace in the tree is a run the catalogue names, so none is left ungraded.
        [Fact]
        public void Every_synthetic_trace_belongs_to_a_named_run()
        {
            var named = FrameRuns().Concat(StepRuns()).Concat(BoardList.SelectMany(b => RandomRuns(b.Mapper, b.Chr))).Select(r => r.Name).ToHashSet();
            var present = Traces("programs").Concat(Traces("steps")).ToHashSet();
            Assert.Empty(present.Except(named));
            Assert.Empty(named.Except(present));
            Assert.Empty(Traces("games").Except(Games.Select(g => "games/" + g.Sha)));
        }

        // One hash changed in frame 300 of a committed trace, in each column, fails naming the frame and the part - the check that grading compares anything.
        [Theory]
        [InlineData("state", 1)]
        [InlineData("picture", 2)]
        [InlineData("sound", 3)]
        public void A_deliberately_broken_trace_is_caught(string part, int column)
        {
            MoonGoldenRun run = FrameRuns().Single(r => r.Name == "programs/board-04-8");
            var golden = GoldenTrace.Parse(File.ReadAllText(MoonGolden.TracePath(run.Name)));
            string[] row = golden.Rows[299];
            row[column] = row[column][..15] + (row[column][15] == '0' ? '1' : '0');
            var broken = GoldenTrace.Parse(golden.Text);
            string? difference = broken.FirstDifference(MoonGolden.Run(run));
            Assert.Equal($"frame 300: the {part} differs", difference?.Split(" (")[0]);
            _output.WriteLine(difference);
        }

        // The same for a chained trace: one digit of a step chain changed is caught at its row.
        [Fact]
        public void A_deliberately_broken_step_chain_is_caught()
        {
            MoonGoldenRun run = StepRuns().First();
            var golden = GoldenTrace.Parse(File.ReadAllText(MoonGolden.TracePath(run.Name)));
            string[] row = golden.Rows[50];
            row[1] = row[1][..15] + (row[1][15] == '0' ? '1' : '0');
            string? difference = GoldenTrace.Parse(golden.Text).FirstDifference(MoonGolden.Run(run));
            Assert.Equal($"step {row[0]}: the chain differs", difference?.Split(" (")[0]);
        }

        private static IEnumerable<string> Traces(string under)
        {
            string dir = Path.Combine(MoonGolden.Directory(), under);
            return Directory.Exists(dir)
                ? Directory.GetFiles(dir, "*.trace", SearchOption.AllDirectories).Select(p => Path.GetRelativePath(MoonGolden.Directory(), p)[..^".trace".Length].Replace('\\', '/'))
                : Enumerable.Empty<string>();
        }

        // Records when asked, writing only a trace that changed; otherwise grades from boot, then from a recorded interval state where this machine keeps one.
        private string Check(MoonGoldenRun run)
        {
            string path = MoonGolden.TracePath(run.Name);
            if (MoonGolden.Recording)
            {
                string reason = MoonGolden.Reason;
                GoldenTrace trace = MoonGolden.Run(run, interval: (f, s) => MoonGolden.SaveState(run.Name, f, s));
                if (File.Exists(path) && GoldenTrace.Parse(File.ReadAllText(path)).FirstDifference(trace) is null) return $"{run.Name}: unchanged";
                trace.Reason = reason;
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllText(path, trace.Text);
                return $"{run.Name}: recorded, {trace.Rows.Count} rows";
            }

            Assert.True(File.Exists(path), $"{run.Name} has no golden; record one with {MoonGolden.RecordVariable}=1 and a reason in {MoonGolden.ReasonVariable}.");
            var golden = GoldenTrace.Parse(File.ReadAllText(path));
            GoldenTrace now = MoonGolden.Run(run);
            if (golden.FirstDifference(now) is { } difference) Assert.Fail($"{run.Name}: {difference}{Bisect(run, difference)}");
            if (run.Steps > 0 || run.Corpus) return $"{run.Name}: {now.Rows.Count} rows as recorded";

            int resumeAt = MoonGolden.IntervalFrames * Math.Max(1, run.Frames / 4 / MoonGolden.IntervalFrames);
            if (resumeAt >= run.Frames || MoonGolden.CachedState(run.Name, resumeAt) is not { } cached || cached.Frame != resumeAt)
                return $"{run.Name}: {now.Rows.Count} frames as recorded";
            GoldenTrace resumed = MoonGolden.Run(run, cached.State, resumeAt);
            var tail = GoldenTrace.Parse(golden.Text);
            tail.Rows.RemoveRange(0, resumeAt);
            if (tail.FirstDifference(resumed) is { } after) Assert.Fail($"{run.Name}, resumed from the recorded state of frame {resumeAt}: {after}");
            return $"{run.Name}: {now.Rows.Count} frames as recorded, and again from the recorded state of frame {resumeAt}";
        }

        // The first recorded interval state at or after a failing frame, and the first field this run's state differs from it in.
        private static string Bisect(MoonGoldenRun run, string difference)
        {
            Match m = Regex.Match(difference, @"^frame (\d+):");
            if (!m.Success || MoonGolden.CachedState(run.Name, int.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture)) is not { } cached) return "";
            byte[]? mine = null;
            MoonGolden.Run(run with { Frames = cached.Frame }, interval: (f, s) => { if (f == cached.Frame) mine = s; });
            if (mine is null || mine.Length != cached.State.Length) return $"; at frame {cached.Frame} the states differ in length";
            int first = mine.AsSpan().CommonPrefixLength(cached.State);
            if (first == mine.Length) return $"; at frame {cached.Frame} the state equals the recorded one byte for byte, so the trace moved and not the engine";
            using var machine = new MoonMachine(run.Rom());
            string field = machine.Layout().Split('\n').Where(l => l.Length > 0).Select(l => l.Split(' '))
                .LastOrDefault(p => int.Parse(p[0], CultureInfo.InvariantCulture) <= first) is { } line ? line[^1] : "?";
            return $"; at frame {cached.Frame} the state first differs from the recorded one at byte {first}, in {field}";
        }
    }
}
