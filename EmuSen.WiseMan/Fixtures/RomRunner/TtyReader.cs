using System.Globalization;
using System.Text;
using System.Text.RegularExpressions;

namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // A program's TTY as text, from the `tty` space a core keeps it in, up to its first NUL - see Diamond_Native.md §11.12.
    public static class TtyReader
    {
        public const string Space = "tty";

        public static string? Text(RomSnapshot snapshot)
        {
            if (!snapshot.Spaces.TryGetValue(Space, out var bytes)) return null;
            int end = Array.IndexOf(bytes, (byte)0);
            return Encoding.ASCII.GetString(bytes, 0, end < 0 ? bytes.Length : end).Replace("\r", "");
        }

        public static IReadOnlyList<string> Lines(string text) => text.Split('\n').Select(l => l.TrimEnd()).ToArray();

        // The text from a program's heading line on, so that a kernel's own boot messages before it are left out.
        public static string From(string text, string heading)
        {
            int at = text.IndexOf(heading, StringComparison.Ordinal);
            return at < 0 ? text : text[at..];
        }
    }

    // How a test program's TTY is graded, as data: the program's path suffix, its heading line and the rule.
    public enum TtyRule { PassFail, TableAgainstLog, LinesAgainstLog, TimersAgainstLog, BandwidthAgainstLog, Expect }

    public sealed record TtyProgram(string Path, string Heading, TtyRule Rule, double Tolerance = 0, string[]? Disputed = null, string Settings = "", int Frame = 900, string[]? Expected = null);

    // ps1-tests' programs the runner grades, and the rules - see Diamond_Native.md §11.12.
    public static class Ps1Tests
    {
        public static readonly TtyProgram[] Programs =
        {
            new("cpu/cop/cop.exe", "cpu/cop", TtyRule.PassFail),
            new("cpu/code-in-io/code-in-io.exe", "cpu/code-in-io", TtyRule.PassFail),
            new("dma/otc-test/otc-test.exe", "dma/otc-test", TtyRule.PassFail),
            new("dma/dpcr/dpcr.exe", "dma/dpcr", TtyRule.PassFail),
            // Its table against the console's in cycles a load; the 32-bit SPUCNT row times an address error, Diamond_Disputes.md D-4.
            new("cpu/access-time/access-time.exe", "cpu/access-time", TtyRule.TableAgainstLog, 0.5, new[] { "SPUCNT/32" }),
            // The booleans exactly, the cycles within 10%, Diamond_Disputes.md D-5.
            new("dma/chain-looping/chain-looping.exe", "dma/chain-looping", TtyRule.LinesAgainstLog, 0.10),
            // The console's log is from a PAL-clocked console forced into NTSC mode, Diamond_Disputes.md D-0; its rows within their tolerances.
            new("timers/timers.exe", "timers test", TtyRule.TimersAgainstLog, Settings: "region=psx.eu"),
            new("gpu/gp0-e1/gp0-e1.exe", "gpu/gp0-e1", TtyRule.PassFail),
            new("gpu/mask-bit/mask-bit.exe", "gpu/mask-bit", TtyRule.PassFail),
            // Each row's hblanks within 2% of the console's - see Diamond_GPU.md §8.
            new("gpu/bandwidth/bandwidth.exe", "gpu/bandwidth", TtyRule.BandwidthAgainstLog, 0.02, Frame: 1500),
            // The default console has the 208-pin chip, version 2 by psx-spx's detection.
            new("gpu/version-detect/version-detect.exe", "gpu/version-detect", TtyRule.Expect, Expected: new[] { "* GPU version 2 [New 208pin GPU (LATE-PU-8 and up)]" }),
        };

        public static RomProtocol Protocol(TtyProgram p, Func<string, string?> consoleLog) =>
            new($"tty:{p.Path}", path => path.Replace('\\', '/').EndsWith(p.Path, StringComparison.Ordinal), run => Grade(p, run, consoleLog));

        public static RomVerdict Grade(TtyProgram p, RomRun run, Func<string, string?> consoleLog)
        {
            string name = $"tty:{p.Path}";
            if (run.Snapshots.Count == 0 || TtyReader.Text(run.Snapshots[^1]) is not { } raw) return new RomVerdict(RomOutcome.NoDump, name, "no TTY space");
            if (!raw.Contains(p.Heading, StringComparison.Ordinal)) return new RomVerdict(RomOutcome.Incomplete, name, "the program printed nothing");
            var lines = TtyReader.Lines(TtyReader.From(raw, p.Heading));
            if (p.Rule == TtyRule.PassFail) return PassFail(name, lines);
            if (p.Rule == TtyRule.Expect) return Expect(name, lines, p.Expected ?? Array.Empty<string>());
            if (consoleLog(p.Path) is not { } log) return new RomVerdict(RomOutcome.Incomplete, name, "no console log to compare with");
            var console = TtyReader.Lines(TtyReader.From(log.Replace("\r", ""), p.Heading));
            return p.Rule switch
            {
                TtyRule.TableAgainstLog => Table(name, lines, console, p.Tolerance, p.Disputed ?? Array.Empty<string>()),
                TtyRule.LinesAgainstLog => Numbers(name, lines, console, p.Tolerance),
                TtyRule.BandwidthAgainstLog => Bandwidth(name, lines, console, p.Tolerance),
                _ => Timers(name, lines, console),
            };
        }

        private static RomVerdict PassFail(string name, IReadOnlyList<string> lines)
        {
            var fails = lines.Where(l => l.StartsWith("fail", StringComparison.Ordinal)).ToArray();
            int passes = lines.Count(l => l.StartsWith("pass", StringComparison.Ordinal));
            if (fails.Length > 0) return new RomVerdict(RomOutcome.Failed, name, string.Join("; ", fails));
            bool done = lines.Any(l => l == "Done.");
            return new RomVerdict(done && passes > 0 ? RomOutcome.Passed : RomOutcome.Incomplete, name, $"{passes} pass");
        }

        private static RomVerdict Expect(string name, IReadOnlyList<string> lines, string[] wanted)
        {
            var missing = wanted.Where(w => !lines.Contains(w)).ToArray();
            return new RomVerdict(missing.Length == 0 ? RomOutcome.Passed : RomOutcome.Failed, name, missing.Length == 0 ? $"{wanted.Length} lines" : "missing: " + string.Join("; ", missing));
        }

        private static readonly Regex BandwidthRow = new(@"^(.+?)\s+dT:.*\(hblanks:\s*(\d+)\)");

        // Each named row's hblanks within the fraction of the console's.
        private static RomVerdict Bandwidth(string name, IReadOnlyList<string> lines, IReadOnlyList<string> console, double fraction)
        {
            Dictionary<string, double> Rows(IEnumerable<string> ls) => ls.Select(l => BandwidthRow.Match(l)).Where(m => m.Success).ToDictionary(m => m.Groups[1].Value.Trim(), m => double.Parse(m.Groups[2].Value, CultureInfo.InvariantCulture));
            var ours = Rows(lines);
            var off = new List<string>();
            var theirs = Rows(console);
            foreach (var (row, want) in theirs)
            {
                if (!ours.TryGetValue(row, out double have)) return new RomVerdict(RomOutcome.Incomplete, name, $"no row {row}");
                if (Math.Abs(have - want) > want * fraction) off.Add($"{row}: {have} against {want}");
            }
            return new RomVerdict(off.Count == 0 && theirs.Count > 0 ? RomOutcome.Passed : RomOutcome.Failed, name, off.Count == 0 ? $"{theirs.Count} rows within {fraction:P0}" : string.Join("; ", off));
        }

        private static readonly Regex Row = new(@"^(\S+)\s+\(\s*0x[0-9a-fA-F]+\)\s+(.*)$");

        // A "SEGMENT (ADDRESS) a.b c.d e.f" row's values, each printed as whole cycles and hundredths.
        private static Dictionary<string, double[]> TableRows(IEnumerable<string> lines) =>
            lines.Select(l => Row.Match(l)).Where(m => m.Success)
                .ToDictionary(m => m.Groups[1].Value, m => m.Groups[2].Value.Split(' ', StringSplitOptions.RemoveEmptyEntries).Select(Hundredths).ToArray());

        // NaN when the text is not a number, so that the cell fails rather than the reading.
        private static double Hundredths(string v)
        {
            var parts = v.Split('.');
            if (!int.TryParse(parts[0], CultureInfo.InvariantCulture, out int whole)) return double.NaN;
            if (parts.Length < 2) return whole;
            return int.TryParse(parts[1], CultureInfo.InvariantCulture, out int cents) ? whole + cents / 100.0 : double.NaN;
        }

        private static RomVerdict Table(string name, IReadOnlyList<string> lines, IReadOnlyList<string> console, double tolerance, string[] disputed)
        {
            var ours = TableRows(lines);
            var theirs = TableRows(console);
            var widths = new[] { "8", "16", "32" };
            var off = new List<string>();
            foreach (var (segment, values) in theirs)
            {
                if (!ours.TryGetValue(segment, out var mine)) return new RomVerdict(RomOutcome.Incomplete, name, $"no {segment} row");
                for (int i = 0; i < Math.Min(values.Length, mine.Length); i++)
                {
                    string cell = $"{segment}/{widths[i]}";
                    if (!(Math.Abs(values[i] - mine[i]) <= tolerance) && !disputed.Contains(cell)) off.Add($"{cell} {mine[i]:0.00} against {values[i]:0.00}");
                }
            }
            return new RomVerdict(off.Count == 0 ? RomOutcome.Passed : RomOutcome.Failed, name, off.Count == 0 ? $"{theirs.Count} rows within {tolerance}" : string.Join("; ", off));
        }

        private static readonly Regex Number = new(@"-?\d+");

        // Line by line: the words equal, each number within the fraction of the console's.
        private static RomVerdict Numbers(string name, IReadOnlyList<string> lines, IReadOnlyList<string> console, double fraction)
        {
            var want = console.Where(l => l.Length > 0).ToArray();
            var have = lines.Where(l => l.Length > 0).Take(want.Length).ToArray();
            if (have.Length < want.Length) return new RomVerdict(RomOutcome.Incomplete, name, $"{have.Length} of {want.Length} lines");
            var off = new List<string>();
            for (int i = 0; i < want.Length; i++)
            {
                if (Number.Replace(want[i], "#") != Number.Replace(have[i], "#")) { off.Add($"'{have[i]}' against '{want[i]}'"); continue; }
                var a = Number.Matches(want[i]).Select(m => double.Parse(m.Value, CultureInfo.InvariantCulture)).ToArray();
                var b = Number.Matches(have[i]).Select(m => double.Parse(m.Value, CultureInfo.InvariantCulture)).ToArray();
                for (int k = 0; k < a.Length; k++)
                    if (Math.Abs(a[k] - b[k]) > Math.Abs(a[k]) * fraction) off.Add($"{b[k]} against {a[k]} in '{want[i]}'");
            }
            return new RomVerdict(off.Count == 0 ? RomOutcome.Passed : RomOutcome.Failed, name, off.Count == 0 ? $"{want.Length} lines" : string.Join("; ", off));
        }

        // A frame's hblanks exactly, its dots ±10 and its cycles ±60; a delay's cycles ±3 and its slower ticks ±2.
        private static double TimerTolerance(string row) => row.EndsWith("frame delay", StringComparison.Ordinal)
            ? row.Contains("Hblank") ? 0 : row.Contains("Dot clock") ? 10 : 60
            : row.Contains("/8") || row.Contains("Dot clock") || row.Contains("Hblank") ? 2 : 3;

        private static readonly Regex TimerRow = new(@"^Timer(\d), clock:\s*(.+?),\s*(.+?):\s*(.*)$");

        // Each delay and frame row's settled values within its tolerance of the console's mean; the sync-mode tables are not graded - see Diamond_Native.md §11.12.
        private static RomVerdict Timers(string name, IReadOnlyList<string> lines, IReadOnlyList<string> console)
        {
            var off = new List<string>();
            int graded = 0;
            Dictionary<string, double[]> Rows(IEnumerable<string> ls) => ls.Select(l => TimerRow.Match(l)).Where(m => m.Success)
                .ToDictionary(m => $"Timer{m.Groups[1].Value}/{m.Groups[2].Value}/{m.Groups[3].Value}",
                    m => m.Groups[4].Value.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries).Skip(2).Select(v => double.Parse(v, CultureInfo.InvariantCulture)).ToArray());
            var ours = Rows(lines);
            foreach (var (row, values) in Rows(console))
            {
                if (!ours.TryGetValue(row, out var mine)) return new RomVerdict(RomOutcome.Incomplete, name, $"no row {row}");
                double tolerance = TimerTolerance(row);
                double mean = values.Average();
                graded++;
                foreach (double v in mine)
                    if (Math.Abs(v - mean) > tolerance) { off.Add($"{row}: {v} against {mean:0.0}"); break; }
            }
            return new RomVerdict(off.Count == 0 && graded > 0 ? RomOutcome.Passed : RomOutcome.Failed, name, off.Count == 0 ? $"{graded} rows" : string.Join("; ", off));
        }
    }
}
