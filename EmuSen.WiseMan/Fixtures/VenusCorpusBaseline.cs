using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;

namespace EmuSen.WiseMan.Fixtures
{
    // The SNES corpus's verdicts per ROM from the committed baseline: C# Venus's, frozen since it was deleted, Mesen's and VenusRT's - see VenusRT_Native.md §68.3.
    public static class VenusCorpusBaseline
    {
        public sealed record Row(string Md5, string Rom, string Csharp, string Mesen, string VenusRt);

        public static string PathOf([CallerFilePath] string here = "") =>
            Path.Combine(Path.GetDirectoryName(Path.GetDirectoryName(here)!)!, "Cores", "VenusRtBaseline.tsv");

        public static IReadOnlyList<Row> Read()
        {
            var lines = File.ReadAllLines(PathOf()).Where(l => l.Length > 0 && l[0] != '#').ToList();
            string[] header = lines[0].Split('\t');
            int Column(string name) => System.Array.IndexOf(header, name);
            int md5 = Column("md5"), rom = Column("rom"), venus = Column("venus"), mesen = Column("mesen"), venusRt = Column("venusrt");
            return lines.Skip(1).Select(l => l.Split('\t')).Select(c => new Row(c[md5], c[rom], c[venus], mesen < 0 ? "" : c[mesen], venusRt < 0 ? "" : c[venusRt])).ToList();
        }

        // A verdict's standing: a pass above a ROM that gave no verdict of its own or did not finish, above a failure, above an error.
        public static int Rank(string verdict) => verdict switch
        {
            "Passed" => 3,
            "Done" or "Visual" or "Incomplete" => 1,
            "Failed" => 0,
            _ => -1,
        };
    }
}
