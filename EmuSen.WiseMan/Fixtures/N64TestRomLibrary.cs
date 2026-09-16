using EmuSen.DianaOS.DianaOS.Etc;

namespace EmuSen.WiseMan.Fixtures
{
    // The N64 hardware corpus in the gitignored TestRoms/n64, none required - see Mars_TestOracle.md §4.
    public static class N64TestRomLibrary
    {
        public const string SubdirectoryName = "n64";

        // Built from MIT source rather than released, so the name is the corpus's own - see Mars_TestOracle.md §1.
        public const string SystemTestRomName = "n64-systemtest.z64";

        public static string Root => Path.Combine(DianaOSSandbox.InstallDirectory, "TestRoms", SubdirectoryName);

        public static IReadOnlyList<string> Find()
        {
            if (!Directory.Exists(Root)) return Array.Empty<string>();

            var found = new List<string>();
            foreach (string extension in new[] { ".z64", ".n64", ".v64" })
            {
                found.AddRange(Directory.GetFiles(Root, $"*{extension}", SearchOption.AllDirectories));
            }

            found.Sort(StringComparer.Ordinal);
            return found;
        }

        // Null when the corpus is absent, which every test here treats as a skip rather than a failure.
        public static string? FindSystemTest()
        {
            foreach (string path in Find())
            {
                if (Path.GetFileName(path) == SystemTestRomName) return path;
            }

            return null;
        }

        // xUnit rejects empty TheoryData, so "no ROMs present" has to arrive as one empty case.
        public static TheoryData<string> AsTheoryData()
        {
            var data = new TheoryData<string>();
            foreach (string path in Find()) data.Add(path);

            if (data.Count == 0) data.Add(string.Empty);
            return data;
        }
    }
}
