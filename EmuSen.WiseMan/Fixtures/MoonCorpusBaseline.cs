using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;

namespace EmuSen.WiseMan.Fixtures
{
    // The NES corpus's verdicts per ROM: the C# core's, recorded once before it was deleted, and MoonRT's - see Moon_Native.md §9.3.
    public static class MoonCorpusBaseline
    {
        public sealed record Row(string Rom, string Sha, string Csharp, string MoonRt);

        public static string PathOf([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "MoonCorpusBaseline.tsv");

        public static IReadOnlyList<Row> Read() => File.ReadAllLines(PathOf())
            .Where(l => l.Length > 0 && l[0] != '#' && !l.StartsWith("rom\t", System.StringComparison.Ordinal))
            .Select(l => l.Split('\t')).Select(p => new Row(p[0], p[1], p[2], p[3])).ToList();

        // A verdict's standing: a pass above a failure above no verdict above an error; a ROM no board runs stands apart, and is held equal.
        public static int Rank(string verdict) => verdict switch
        {
            "Passed" => 3,
            _ when verdict.StartsWith("Failed", System.StringComparison.Ordinal) => 2,
            "NoResult" => 1,
            "Unsupported" => -1,
            _ => 0,
        };
    }
}
