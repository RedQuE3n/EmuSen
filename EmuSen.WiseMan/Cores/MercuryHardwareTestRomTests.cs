using EmuSen.Cores.Native;
using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Security.Cryptography;
using System.Threading.Tasks;
using EmuSen.Cores;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // blargg and mooneye judged by their own reports, each ROM's verdict on each console held to a recorded baseline in either direction - see Mercury_Native.md §12.2.
    public class MercuryHardwareTestRomTests
    {
        // blargg's combined cpu_instrs is the longest of these at roughly half a minute of emulated time.
        public const int FrameCap = 3600;

        // Set to 1 to write this run's verdicts into the committed baseline instead of comparing them.
        public const string RecordVariable = "EMUSEN_MERCURY_CORPUS_RECORD";

        public const string BaselineName = "MercuryCorpusBaseline.tsv";

        // The baseline's columns, one per console; the engine's name stays in the header, as the file has always named its columns.
        private static readonly GbModel[] Columns = Enum.GetValues<GbModel>();

        private readonly ITestOutputHelper _output;

        public MercuryHardwareTestRomTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<GbModel> Runs
        {
            get
            {
                var data = new TheoryData<GbModel>();
                foreach (GbModel model in Columns) data.Add(model);
                return data;
            }
        }

        [Theory]
        [MemberData(nameof(Runs))]
        public void Each_hardware_test_rom_says_what_the_baseline_recorded(GbModel model)
        {
            IReadOnlyList<string> roms = HardwareTestRomLibrary.Find();
            if (roms.Count == 0)
            {
                _output.WriteLine($"No corpus under {HardwareTestRomLibrary.Root} ({HardwareTestRomLibrary.Variable} unset or empty); not run.");
                return;
            }

            var verdicts = new ConcurrentDictionary<string, (string Rom, string Sha, string Verdict, int Frame)>();
            Parallel.ForEach(roms, new ParallelOptions { MaxDegreeOfParallelism = Math.Max(1, Environment.ProcessorCount / 4) }, path =>
            {
                string name = Path.GetRelativePath(HardwareTestRomLibrary.Root, path).Replace('\\', '/');
                string sha = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path)))[..16];
                var (verdict, frame) = Run(model, path);
                verdicts[name] = (name, sha, verdict, frame);
            });

            string column = Column(model);
            var counts = verdicts.Values.GroupBy(v => v.Verdict.Split('(')[0]).OrderBy(g => g.Key, StringComparer.Ordinal).Select(g => $"{g.Key} {g.Count()}");
            _output.WriteLine($"{column}: {roms.Count} ROMs, {string.Join(", ", counts)}");

            string source = SourceBaseline();
            if (Environment.GetEnvironmentVariable(RecordVariable) == "1")
            {
                Record(source, column, verdicts.Values);
                _output.WriteLine($"recorded into {source}");
                return;
            }

            string baseline = File.Exists(source) ? source : Path.Combine(AppContext.BaseDirectory, "Cores", BaselineName);
            Assert.True(File.Exists(baseline), $"No corpus baseline; record one with {RecordVariable}=1.");
            var table = Read(baseline);
            int at = Array.IndexOf(table.Header, column);
            Assert.True(at >= 0, $"The baseline has no column {column}; record it with {RecordVariable}=1.");

            var differences = new List<string>();
            foreach (var v in verdicts.Values.OrderBy(v => v.Rom, StringComparer.Ordinal))
            {
                if (!table.Rows.TryGetValue(v.Rom, out string[]? row)) differences.Add($"{v.Rom}: not in the baseline, now {v.Verdict}");
                else if (row[1] != v.Sha) differences.Add($"{v.Rom}: a different ROM from the one recorded ({row[1]}, now {v.Sha})");
                else if (row[at] != v.Verdict) differences.Add($"{v.Rom}: recorded {row[at]}, now {v.Verdict} at frame {v.Frame}");
            }
            foreach (string rom in table.Rows.Keys.Where(r => !verdicts.ContainsKey(r)).Order(StringComparer.Ordinal)) differences.Add($"{rom}: recorded, not in this corpus");

            Assert.True(differences.Count == 0,
                $"{column}: {differences.Count} verdicts moved from the baseline. A regression is a defect; an improvement is re-recorded deliberately with {RecordVariable}=1 and the file committed.\n" + string.Join("\n", differences));
        }

        // Runs to a verdict or the cap; serial verdicts are Passed, Failed or NoVerdict, cart-RAM ones Passed or Failed(status).
        public static (string Verdict, int Frame) Run(GbModel model, string path)
        {
            using MercuryRig core = MercuryRig.Open(path, model);
            core.SkipRendering = true;
            for (int frame = 1; frame <= FrameCap; frame++)
            {
                try { core.RunFrame(); }
                catch (NotSupportedException) { return ("IllegalOpcode", frame); }

                TestRomVerdict serial = HardwareTestRomLibrary.ReadVerdict(core.SerialLog);
                if (serial != TestRomVerdict.NoVerdict) return (serial.ToString(), frame);

                if (HardwareTestRomLibrary.ReadCartRamStatus(i => core.ReadSpace(MercuryEngines.SpaceCartRam, i)) is int status)
                    return (status == 0 ? "Passed" : $"Failed({status})", frame);
            }
            return (TestRomVerdict.NoVerdict.ToString(), FrameCap);
        }

        public static string Column(GbModel model) => $"MercuryRt/{model}";

        private static string SourceBaseline([CallerFilePath] string here = "") => Path.Combine(Path.GetDirectoryName(here)!, BaselineName);

        private static (string[] Header, Dictionary<string, string[]> Rows) Read(string path)
        {
            string[] lines = File.ReadAllLines(path).Where(l => l.Length > 0 && !l.StartsWith('#')).ToArray();
            string[] header = lines[0].Split('\t');
            return (header, lines.Skip(1).Select(l => l.Split('\t')).ToDictionary(c => c[0], StringComparer.Ordinal));
        }

        // One column replaced, the others kept; a ROM new to the corpus gets "-" in the columns not yet recorded.
        private static void Record(string path, string column, IEnumerable<(string Rom, string Sha, string Verdict, int Frame)> verdicts)
        {
            string[] header = new[] { "rom", "sha256" }.Concat(Columns.Select(Column)).ToArray();
            var rows = new SortedDictionary<string, string[]>(StringComparer.Ordinal);
            if (File.Exists(path))
            {
                var old = Read(path);
                foreach (var (rom, cells) in old.Rows)
                    rows[rom] = header.Select(h => Array.IndexOf(old.Header, h) is int i and >= 0 && i < cells.Length ? cells[i] : "-").ToArray();
            }
            int at = Array.IndexOf(header, column);
            foreach (var v in verdicts)
            {
                if (!rows.TryGetValue(v.Rom, out string[]? row) || row[1] != v.Sha) rows[v.Rom] = row = header.Select(_ => "-").ToArray();
                row[0] = v.Rom;
                row[1] = v.Sha;
                row[at] = v.Verdict;
            }
            var text = new List<string>
            {
                "# blargg and mooneye verdicts on MercuryRT per console, the first 3,600 frames; re-recorded only with " + RecordVariable + "=1 - see Mercury_Native.md §12.2.",
                string.Join('\t', header),
            };
            text.AddRange(rows.Values.Select(r => string.Join('\t', r)));
            File.WriteAllText(path, string.Join('\n', text) + "\n");
        }
    }
}
