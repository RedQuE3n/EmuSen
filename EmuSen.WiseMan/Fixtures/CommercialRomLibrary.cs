using EmuSen.DianaSh.DianaSh.Etc;

namespace EmuSen.WiseMan.Fixtures
{
    // Whatever commercial ROMs happen to be sitting in the gitignored TestRoms
    // directory. Nothing here is committed and nothing here is required - see
    // Mercury_RealCartridges.md §1.
    public static class CommercialRomLibrary
    {
        // A folder that overrides the install's TestRoms, read in place and never written, as a worktree has none of its own.
        public const string Variable = "EMUSEN_COMMERCIAL_ROMS";

        public static string Root => Environment.GetEnvironmentVariable(Variable) is { Length: > 0 } folder ? folder : Path.Combine(DianaShSandbox.InstallDirectory, "TestRoms");

        public static IReadOnlyList<string> Find(params string[] extensions)
        {
            if (!Directory.Exists(Root)) return Array.Empty<string>();

            var found = new List<string>();
            foreach (string extension in extensions)
            {
                found.AddRange(Directory.GetFiles(Root, $"*{extension}"));
            }

            found.Sort(StringComparer.Ordinal);
            return found;
        }

        // xUnit rejects empty TheoryData, so "no ROMs present" has to arrive as one empty case.
        public static TheoryData<string> AsTheoryData(params string[] extensions)
        {
            var data = new TheoryData<string>();
            foreach (string path in Find(extensions)) data.Add(path);

            if (data.Count == 0) data.Add(string.Empty);
            return data;
        }
    }
}
