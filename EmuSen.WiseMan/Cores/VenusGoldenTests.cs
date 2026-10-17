using System.Collections.Concurrent;
using System.Globalization;
using System.Security.Cryptography;
using System.Text.RegularExpressions;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;
using EmuSen.WiseMan.Fixtures.Snes;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // VenusRT's goldens, recorded from VenusRT itself and graded frame by frame, with the open replacements in place of firmware - see VenusRT_Native.md §68.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusGoldenTests : IDisposable
    {
        // A folder of SNES images, or "library" for AppSettings.RomDirectory; the games are found there by size and hash, and absent, their case passes unrun.
        public const string GamesVariable = "EMUSEN_VENUSRT_ROMS";

        public const int GameFrames = 3600;

        // The games, by the first sixteen digits of their SHA-256 and their size, with what each brings.
        public static readonly (string Sha, long Size, string Board)[] Games =
        {
            ("D70C9C7716AD12C6", 524800, "LoROM"),
            ("740646F3535BFB36", 4194304, "SA-1"),
            ("DEABC7D04688D00F", 2097664, "GSU-2"),
            ("6495D32E95774B1C", 524800, "DSP-1B"),
            ("A031B5AB828CDAD8", 1049088, "ST010"),
            ("A81FD8F3BC36A063", 2097664, "OBC1"),
        };

        private readonly ITestOutputHelper _output;

        public VenusGoldenTests(ITestOutputHelper output)
        {
            _output = output;
            FirmwareLibrary.Directory = VenusGolden.NoFirmware;
            CoreDiscovery.UseDirectories(null);
        }

        public void Dispose() => FirmwareLibrary.ResetDirectory();

        // The synthetic programs by frames: the display on LoROM and HiROM, its effects, the sound processor, each coprocessor, rendering skipped, and two resets.
        public static IEnumerable<VenusGoldenRun> FrameRuns()
        {
            yield return new("programs/display", VenusPrograms.Display, Input: "pads", Frames: 600);
            yield return new("programs/display-hirom", VenusPrograms.HiRom, Input: "pads", Frames: 600);
            yield return new("programs/effects", VenusPrograms.Effects, Input: "pads", Frames: 600);
            yield return new("programs/sound", VenusPrograms.Sound, Input: "pads", Frames: 600);
            yield return new("programs/sa1", VenusPrograms.Sa1, Input: "pads", Frames: 600);
            yield return new("programs/gsu", VenusPrograms.SuperFx, Frames: 600);
            yield return new("programs/dsp1", VenusPrograms.NecDsp, Input: "pads", Frames: 600);
            yield return new("programs/obc1", VenusPrograms.Obc1, Input: "pads", Frames: 600);
            yield return new("programs/effects-skipped", VenusPrograms.Effects, Input: "pads", SkipRendering: true, Frames: 300);
            yield return new("programs/sound-resets", VenusPrograms.Sound, Input: "pads", Frames: 300, Resets: new[] { 100, 200 });
        }

        // Random bytes as code on each board, eight seeds, 300 frames.
        public static IEnumerable<VenusGoldenRun> RandomRuns(string board) => Enumerable.Range(0, 8).Select(seed =>
            new VenusGoldenRun($"random/{board}-{seed}", () => VenusPrograms.Random(board, seed), Input: "pads", Frames: 300));

        // Every corpus ROM of the manifest to the frames the runner grades it at, its verdict by its suite's protocol.
        public static IEnumerable<VenusGoldenRun> CorpusRuns() => SnesTestRomCorpus.Root is not { } root ? Enumerable.Empty<VenusGoldenRun>()
            : SnesTestRomCorpus.Unique(root).Select(r => Path.Combine(root, r.Path)).Select(path => new VenusGoldenRun(
                "corpus/" + Path.GetRelativePath(root, path).Replace('\\', '/'), () => File.ReadAllBytes(path), Frames: SnesTestRomCorpus.Frames(path).Max(), Corpus: true, CorpusPath: path));

        private static string? GamesRoot => Environment.GetEnvironmentVariable(GamesVariable) switch
        {
            null or "" => null,
            "library" => AppSettings.Load().RomDirectory,
            var dir => dir,
        };

        // The catalogue's games found under the folder by size, then hash; named by hash alone.
        public static IEnumerable<VenusGoldenRun> GameRuns()
        {
            if (GamesRoot is not { } root || !Directory.Exists(root)) yield break;
            var files = Directory.EnumerateFiles(root, "*.s?c", SearchOption.AllDirectories).Select(f => new FileInfo(f)).ToList();
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
                foreach (var run in FrameRuns()) data.Add(run.Name);
                return data;
            }
        }

        public static TheoryData<string> BoardNames
        {
            get
            {
                var data = new TheoryData<string>();
                foreach (var (name, _) in VenusPrograms.Boards) data.Add(name);
                return data;
            }
        }

        [Theory]
        [MemberData(nameof(NamedRuns))]
        public void A_synthetic_program_runs_as_its_golden_recorded(string name)
        {
            if (VenusGolden.Discovered is null) return;
            _output.WriteLine(Check(FrameRuns().Single(r => r.Name == name)));
        }

        [Theory]
        [MemberData(nameof(BoardNames))]
        public void Random_programs_run_as_their_goldens_recorded(string board)
        {
            if (VenusGolden.Discovered is null) return;
            foreach (VenusGoldenRun run in RandomRuns(board)) _output.WriteLine(Check(run));
        }

        [Fact]
        public void The_corpus_runs_as_its_goldens_recorded()
        {
            var runs = CorpusRuns().ToList();
            if (runs.Count == 0 || VenusGolden.Discovered is null)
            {
                _output.WriteLine($"{SnesTestRomCorpus.Variable} unset, not run");
                return;
            }
            int threads = int.TryParse(Environment.GetEnvironmentVariable("EMUSEN_VENUSRT_THREADS"), out int t) ? t : Math.Max(1, Environment.ProcessorCount / 4);
            var failures = new ConcurrentBag<string>();
            Parallel.ForEach(runs, new ParallelOptions { MaxDegreeOfParallelism = threads }, run =>
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
            var runs = VenusGolden.Discovered is null ? new List<VenusGoldenRun>() : GameRuns().ToList();
            if (runs.Count == 0) _output.WriteLine($"{GamesVariable} unset or holding none of the games, not run");
            foreach (VenusGoldenRun run in runs) _output.WriteLine(Check(run));
        }

        // The verdicts the corpus traces record: every ROM C# Venus passed in the frozen baseline passes on VenusRT, and none falls below the C# core's verdict - see VenusRT_Native.md §68.3.
        [Fact]
        public void No_corpus_verdict_is_worse_than_the_csharp_baseline()
        {
            var baseline = VenusCorpusBaseline.Read();
            Assert.Equal(295, baseline.Count);
            var worse = new List<string>();
            foreach (var row in baseline)
            {
                string path = VenusGolden.TracePath("corpus/" + row.Rom);
                Assert.True(File.Exists(path), $"{row.Rom} is in the baseline and has no golden");
                string verdict = GoldenTrace.Parse(File.ReadAllText(path)).Transcript["verdict"].Split(' ')[0];
                if (VenusCorpusBaseline.Rank(verdict) < VenusCorpusBaseline.Rank(row.Csharp)) worse.Add($"{row.Rom}: C# {row.Csharp}, VenusRT {verdict}");
            }
            Assert.True(worse.Count == 0, string.Join("\n", worse));
            int csharp = baseline.Count(r => r.Csharp == "Passed");
            int ours = baseline.Count(r => GoldenTrace.Parse(File.ReadAllText(VenusGolden.TracePath("corpus/" + r.Rom))).Transcript["verdict"].StartsWith("Passed", StringComparison.Ordinal));
            Assert.True(ours >= csharp, $"VenusRT passes {ours}, C# Venus passed {csharp}");
            _output.WriteLine($"{baseline.Count} ROMs: {ours} pass on VenusRT, {csharp} passed on C# Venus, {baseline.Count(r => r.Mesen == "Passed")} on Mesen");
        }

        // Every trace in the tree is a run the catalogue names, so none is left ungraded.
        [Fact]
        public void Every_synthetic_trace_belongs_to_a_named_run()
        {
            var named = FrameRuns().Concat(VenusPrograms.Boards.SelectMany(b => RandomRuns(b.Name))).Select(r => r.Name).ToHashSet();
            var present = Traces("programs").Concat(Traces("random")).ToHashSet();
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
            if (VenusGolden.Discovered is null) return;
            VenusGoldenRun run = FrameRuns().Single(r => r.Name == "programs/sound");
            var golden = GoldenTrace.Parse(File.ReadAllText(VenusGolden.TracePath(run.Name)));
            string[] row = golden.Rows[299];
            row[column] = row[column][..15] + (row[column][15] == '0' ? '1' : '0');
            string? difference = GoldenTrace.Parse(golden.Text).FirstDifference(VenusGolden.Run(run));
            Assert.Equal($"frame 300: the {part} differs", difference?.Split(" (")[0]);
            _output.WriteLine(difference);
        }

        // The same for a corpus chain: one digit of a chained row changed is caught at its row.
        [Fact]
        public void A_deliberately_broken_corpus_chain_is_caught()
        {
            if (VenusGolden.Discovered is null || CorpusRuns().FirstOrDefault(r => r.Name.EndsWith("/CPUADC.sfc", StringComparison.Ordinal)) is not { } run) return;
            var golden = GoldenTrace.Parse(File.ReadAllText(VenusGolden.TracePath(run.Name)));
            string[] row = golden.Rows[10];
            row[1] = row[1][..15] + (row[1][15] == '0' ? '1' : '0');
            string? difference = GoldenTrace.Parse(golden.Text).FirstDifference(VenusGolden.Run(run));
            Assert.Equal($"frame {row[0]}: the chain differs", difference?.Split(" (")[0]);
        }

        private static IEnumerable<string> Traces(string under)
        {
            string dir = Path.Combine(VenusGolden.Directory(), under);
            return Directory.Exists(dir)
                ? Directory.GetFiles(dir, "*.trace", SearchOption.AllDirectories).Select(p => Path.GetRelativePath(VenusGolden.Directory(), p)[..^".trace".Length].Replace('\\', '/'))
                : Enumerable.Empty<string>();
        }

        // Records when asked, writing only a trace that changed; otherwise grades from power-on, then from a recorded interval state where this machine keeps one.
        private static string Check(VenusGoldenRun run)
        {
            string path = VenusGolden.TracePath(run.Name);
            if (VenusGolden.Recording)
            {
                string reason = VenusGolden.Reason;
                GoldenTrace trace = VenusGolden.Run(run, interval: run.Corpus ? null : (f, s) => VenusGolden.SaveState(run.Name, f, s));
                if (File.Exists(path) && GoldenTrace.Parse(File.ReadAllText(path)).FirstDifference(trace) is null) return $"{run.Name}: unchanged";
                trace.Reason = reason;
                Directory.CreateDirectory(Path.GetDirectoryName(path)!);
                File.WriteAllText(path, trace.Text);
                return $"{run.Name}: recorded, {trace.Rows.Count} rows";
            }

            Assert.True(File.Exists(path), $"{run.Name} has no golden; record one with {VenusGolden.RecordVariable}=1 and a reason in {VenusGolden.ReasonVariable}.");
            var golden = GoldenTrace.Parse(File.ReadAllText(path));
            GoldenTrace now = VenusGolden.Run(run);
            if (golden.FirstDifference(now) is { } difference) Assert.Fail($"{run.Name}: {difference}{Bisect(run, difference)}");
            if (run.Corpus) return $"{run.Name}: {now.Rows.Count} rows as recorded";

            int resumeAt = VenusGolden.IntervalFrames * Math.Max(1, run.Frames / 4 / VenusGolden.IntervalFrames);
            if (resumeAt >= run.Frames || VenusGolden.CachedState(run.Name, resumeAt) is not { } cached || cached.Frame != resumeAt)
                return $"{run.Name}: {now.Rows.Count} frames as recorded";
            GoldenTrace resumed = VenusGolden.Run(run, cached.State, resumeAt);
            var tail = GoldenTrace.Parse(golden.Text);
            tail.Rows.RemoveRange(0, resumeAt);
            if (tail.FirstDifference(resumed) is { } after) Assert.Fail($"{run.Name}, resumed from the recorded state of frame {resumeAt}: {after}");
            return $"{run.Name}: {now.Rows.Count} frames as recorded, and again from the recorded state of frame {resumeAt}";
        }

        // The first recorded interval state at or after a failing frame, and the field this run's state first differs from it in.
        private static string Bisect(VenusGoldenRun run, string difference)
        {
            Match m = Regex.Match(difference, @"^frame (\d+):");
            if (!m.Success || run.Corpus || VenusGolden.CachedState(run.Name, int.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture)) is not { } cached) return "";
            byte[]? mine = null;
            VenusGolden.Run(run with { Frames = cached.Frame }, interval: (f, s) => { if (f == cached.Frame) mine = s; });
            if (mine is null || mine.Length != cached.State.Length) return $"; at frame {cached.Frame} the states differ in length";
            int first = mine.AsSpan().CommonPrefixLength(cached.State);
            if (first == mine.Length) return $"; at frame {cached.Frame} the state equals the recorded one byte for byte, so the trace moved and not the engine";
            string field = VenusGolden.Layout(run.Rom()).Split('\n').Where(l => l.Length > 0).Select(l => l.Split(' '))
                .LastOrDefault(p => int.TryParse(p[0], CultureInfo.InvariantCulture, out int at) && at <= first) is { } line ? line[^1] : "?";
            return $"; at frame {cached.Frame} the state first differs from the recorded one at byte {first}, in {field}";
        }
    }
}
