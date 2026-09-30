using System.Text;
using System.Text.RegularExpressions;

namespace EmuSen.WiseMan.Fixtures.Snes
{
    public enum SnesOutcome
    {
        NoDump,
        Passed,
        Failed,
        // Still running, or never reached its verdict.
        Incomplete,
        // Prints values and "Done": its numbers are graded against another engine, not by itself.
        Done,
        // No self-grading protocol: graded by picture or by values against Mesen.
        Visual,
    }

    // What a test ROM said about itself, by its own suite's protocol - see VenusRT_Plan.md §3.1 and VenusRT_Native.md §3.3.
    public sealed record SnesVerdict(SnesOutcome Outcome, string Protocol, string Detail)
    {
        public bool SelfGraded => Outcome is SnesOutcome.Passed or SnesOutcome.Failed or SnesOutcome.Incomplete;

        public override string ToString() => $"{Outcome} {Protocol} {Detail}".TrimEnd();
    }

    public static class SnesTestRomGrader
    {
        // The ROM's verdict at the run's last frame; the frame at half of it, when there, tells a steady screen from a running one.
        public static SnesVerdict Grade(string romPath, SnesRun run)
        {
            var last = run.Snapshots.Count > 0 ? run.Snapshots[^1] : null;
            if (last is null || !last.Spaces.TryGetValue("vram", out byte[]? vram)) return new(SnesOutcome.NoDump, "", "");
            string name = Path.GetFileName(romPath);
            byte[] low = vram.Where((_, i) => i % 2 == 0).ToArray();

            // gilyon: the word at $0032 begins "Success" or "Failed", the word at $006E is the test number.
            if (romPath.Contains("gilyon") || name is "cputest-full.sfc" or "cputest-basic.sfc" or "spctest.sfc")
            {
                string text = TextAt(vram, 0x32, 7), number = TextAt(vram, 0x6E, 4);
                if (text.StartsWith("Success", StringComparison.Ordinal)) return new(SnesOutcome.Passed, "gilyon", $"last={number}");
                if (text.StartsWith("Failed", StringComparison.Ordinal)) return new(SnesOutcome.Failed, "gilyon", $"test={number}");
                return new(SnesOutcome.Incomplete, "gilyon", $"test={number}");
            }

            // blargg's SPC tests: the backdrop, CGRAM entry 0, turns blue for a pass and red for a failure.
            if (romPath.Contains("blargg-spc-6"))
            {
                byte[] cg = last.Spaces.GetValueOrDefault("cgram") ?? new byte[2];
                int colour = cg[0] | cg[1] << 8, r = colour & 31, g = colour >> 5 & 31, b = colour >> 10 & 31;
                var o = b > r && b > g ? SnesOutcome.Passed : r > b && r > g ? SnesOutcome.Failed : SnesOutcome.Incomplete;
                return new(o, "blargg-backdrop", $"rgb={r},{g},{b}");
            }

            // PeterLemon: PASS and FAIL in the tilemap's low bytes; a failing test loops on its FAIL, a passing one stops.
            string? asm = SourceOf(romPath);
            if (asm is not null)
            {
                int passes = Count(low, "PASS"), fails = Count(low, "FAIL");
                var half = run.At(last.Frame / 2);
                bool steady = half is not null && half.Spaces.TryGetValue("vram", out byte[]? before) && before.AsSpan().SequenceEqual(vram);
                var o = fails > 0 ? SnesOutcome.Failed : passes > 0 && steady ? SnesOutcome.Passed : SnesOutcome.Incomplete;
                return new(o, "peterlemon", $"shown={passes} fail={fails} steady={(steady ? 1 : 0)}");
            }

            // The rest: printable runs of the tilemap's low bytes; the last verdict word, if any, is the ROM's.
            var runs = Regex.Matches(Encoding.Latin1.GetString(low), "[ -~]{4,}").Select(m => m.Value).Where(s => s.Distinct().Count() > 2).ToList();
            var words = Regex.Matches(string.Join(" ", runs), @"\b(Passed|PASSED|Failed|FAILED|FAIL|OK|Done|Success)\b").Select(m => m.Value).ToList();
            string detail = string.Join(" | ", runs.Select(s => s.Trim()));
            if (detail.Length > 400) detail = detail[..400];
            if (words.Count == 0) return new(SnesOutcome.Visual, "text", detail);
            string word = words[^1];
            var outcome = word == "Done" ? SnesOutcome.Done : word is "Passed" or "PASSED" or "OK" or "Success" ? SnesOutcome.Passed : SnesOutcome.Failed;
            return new(outcome, "text", detail);
        }

        private static string TextAt(byte[] vram, int word, int n) =>
            new(Enumerable.Range(0, n).Select(i => (char)vram[(word + i) * 2]).ToArray());

        private static int Count(byte[] hay, string needle) => Regex.Matches(Encoding.Latin1.GetString(hay), Regex.Escape(needle)).Count;

        // The ROM's own source when it counts its passes with PrintText(Pass...: beside it by name, else any .asm in its folder.
        private static string? SourceOf(string rom)
        {
            string own = Path.ChangeExtension(rom, ".asm");
            string? dir = Path.GetDirectoryName(rom);
            var candidates = new[] { own }.Concat(dir is null || !Directory.Exists(dir) ? Array.Empty<string>() : Directory.GetFiles(dir, "*.asm").Order(StringComparer.Ordinal));
            string? first = candidates.FirstOrDefault(File.Exists);
            return first is not null && File.ReadAllText(first, Encoding.Latin1).Contains("PrintText(Pass", StringComparison.Ordinal) ? first : null;
        }
    }

    // The fetched corpus: ~/.cache/emusen/probe/venusrt, never committed - see VenusRT_Plan.md §3.1 and its PROVENANCE.txt.
    public static class SnesTestRomCorpus
    {
        public const string Variable = "EMUSEN_VENUSRT_CORPUS";

        public static string? Root => Environment.GetEnvironmentVariable(Variable) is { } r && Directory.Exists(r) ? r : null;

        // The corpus's own manifest: one ROM per MD5, as unique-roms.txt names them.
        public static IReadOnlyList<(string Md5, string Path)> Unique(string root) =>
            File.ReadLines(System.IO.Path.Combine(root, "unique-roms.txt"))
                .Select(l => l.Split(' ', 3))
                .Select(p => (p[0], p[2]))
                .ToList();

        // Frames each ROM is run to: 1800 and 3600, and 18,000 for blargg's DSP test, which Mesen finishes near 10,800.
        public static int[] Frames(string path) => path.EndsWith("spc_dsp6.sfc", StringComparison.Ordinal) ? new[] { 9000, 18000 } : new[] { 1800, 3600 };

        // The row of VenusRT_Plan.md §3.5's table a ROM belongs to.
        public static string Suite(string path)
        {
            string n = System.IO.Path.GetFileName(path);
            if (path.Contains("gilyon") && n.StartsWith("cputest")) return "gilyon cputest";
            if (n == "spctest.sfc") return "gilyon spctest";
            if (path.Contains("CPUTest/CPU/") || path.Contains("SNES-CPUTest-CPU/")) return "PeterLemon CPU";
            if (path.Contains("CPUTest/SPC700/") || path.Contains("SNES-CPUTest-SPC700/")) return "PeterLemon SPC700";
            if (path.Contains("GSUTest")) return "PeterLemon GSU";
            if (path.Contains("blargg-spc-6")) return n.StartsWith("spc_dsp6") ? "blargg spc_dsp6" : "blargg SPC";
            if (path.Contains("blargg_2010")) return "blargg 2010";
            if (Regex.IsMatch(path, "(?i)adc|sbc")) return "ADC/SBC";
            if (Regex.IsMatch(path, "(?i)mul|div")) return "multiply/divide";
            if (path.Contains("absindx")) return "absindx SA-1";
            if (Regex.IsMatch(path, "(?i)cx4")) return "Cx4";
            if (path.Contains("undisbeliever")) return "undisbeliever";
            if (path.Contains("240pSuite")) return "240p suite";
            if (path.Contains("PeterLemon")) return "PeterLemon other";
            return "higan collection, other";
        }
    }
}
