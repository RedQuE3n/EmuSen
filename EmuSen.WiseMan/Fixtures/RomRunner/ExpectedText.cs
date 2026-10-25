namespace EmuSen.WiseMan.Fixtures.RomRunner
{
    // ps2autotests' rule: a run's whole output, from the expected text's first line on and its line ends normalised, equals the program's .expected file - see Sapphire_Native.md §14.4.
    public static class ExpectedText
    {
        // A program's expected text beside it, as ps2autotests keeps them: alu.elf's is alu.expected.
        public static string? Beside(string path) => Path.ChangeExtension(path, ".expected") is var p && File.Exists(p) ? File.ReadAllText(p) : null;

        // CR LF and CR become LF, and the line feeds that end the text go.
        public static string Normalise(string text) => text.Replace("\r\n", "\n").Replace('\r', '\n').TrimEnd('\n');

        public static RomVerdict Grade(string name, string? output, string expected)
        {
            if (output is null) return new RomVerdict(RomOutcome.NoDump, name, "no output channel");
            var want = Normalise(expected).Split('\n');
            if (want.Length == 0 || want[0].Length == 0) return new RomVerdict(RomOutcome.Incomplete, name, "an empty expected text");
            int start = Normalise(output).IndexOf(want[0], StringComparison.Ordinal);
            if (start < 0) return new RomVerdict(RomOutcome.Incomplete, name, $"the program never printed '{want[0]}'");
            var have = Normalise(output)[start..].Split('\n');
            for (int i = 0; i < Math.Min(have.Length, want.Length); i++)
                if (have[i] != want[i])
                    return new RomVerdict(i == have.Length - 1 && want[i].StartsWith(have[i], StringComparison.Ordinal) ? RomOutcome.Incomplete : RomOutcome.Failed, name, $"line {i + 1}: '{have[i]}' against '{want[i]}'");
            if (have.Length < want.Length) return new RomVerdict(RomOutcome.Incomplete, name, $"{have.Length} of {want.Length} lines");
            if (have.Length > want.Length) return new RomVerdict(RomOutcome.Failed, name, $"{have.Length - want.Length} lines past the end, first '{have[want.Length]}'");
            return new RomVerdict(RomOutcome.Passed, name, $"{want.Length} lines");
        }

        // The run's last snapshot's text in the channel's space against the expected text.
        public static RomVerdict Grade(string name, RomRun run, string expected, string space = TtyReader.Space) =>
            Grade(name, run.Snapshots.Count == 0 ? null : TtyReader.Text(run.Snapshots[^1], space), expected);

        // One program as data: its path suffix ("cpu/ee/alu.elf"), its expected text and the channel's space, found by observation (P2).
        public static RomProtocol Protocol(string suffix, string expected, string space = TtyReader.Space) =>
            new($"expected:{suffix}", path => path.Replace('\\', '/').EndsWith(suffix, StringComparison.Ordinal), run => Grade($"expected:{suffix}", run, expected, space));

        // Every program under a root with its expected text beside it, .elf for the EE and .irx for the IOP, one protocol each.
        public static IReadOnlyList<RomProtocol> Corpus(string root, string space = TtyReader.Space) =>
            Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories)
                .Where(p => p.EndsWith(".elf", StringComparison.Ordinal) || p.EndsWith(".irx", StringComparison.Ordinal))
                .Where(p => Beside(p) is not null).Order(StringComparer.Ordinal)
                .Select(p => Protocol(Path.GetRelativePath(root, p).Replace('\\', '/'), Beside(p)!, space)).ToArray();
    }
}
