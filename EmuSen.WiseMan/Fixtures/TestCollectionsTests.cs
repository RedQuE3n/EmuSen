using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text.RegularExpressions;
using Xunit;

namespace EmuSen.WiseMan.Fixtures
{
    // A test class that sets a process static runs in the serial collection, or the classes beside it read its value - see EmuSen_Galaxia.md §3.14.
    public class TestCollectionsTests
    {
        private static readonly Regex SetsAStatic = new(@"(ConfigStore|DataStore)\.Override(Legacy)?Directory\s*=[^=]|ErrorLog\.DirectoryOverride\s*=[^=]|FetchOverride\s*=[^=]");
        private static readonly Regex Declares = new(@"^\s*(?:(?:public|internal|private|protected|sealed|static|partial|abstract|unsafe)\s+)*class\s+([A-Za-z0-9_]+)", RegexOptions.Multiline);
        private static readonly Regex IsATest = new(@"\[(Fact|Theory|[A-Za-z0-9]+(Fact|Theory))\b");

        private static string Here([CallerFilePath] string path = "") => Path.GetDirectoryName(path)!;

        [Fact]
        public void Every_test_class_that_sets_a_process_static_is_in_the_serial_collection()
        {
            var files = Directory.EnumerateFiles(Path.Combine(Here(), ".."), "*.cs", SearchOption.AllDirectories)
                .Where(f => !f.Contains("/obj/") && !f.Contains("/bin/")).Select(f => (Path: f, Text: File.ReadAllText(f))).ToList();
            var byClass = new Dictionary<string, List<string>>();
            foreach (var (_, text) in files)
                foreach (string name in Declares.Matches(text).Select(m => m.Groups[1].Value).Distinct())
                    (byClass.TryGetValue(name, out var list) ? list : byClass[name] = new List<string>()).Add(text);
            var offenders = byClass.Where(c => c.Value.Any(t => SetsAStatic.IsMatch(t)) && c.Value.Any(t => IsATest.IsMatch(t)))
                .Where(c => !c.Value.Any(t => t.Contains("TestCollections.ProcessGlobals")))
                .Select(c => c.Key).Where(name => name != nameof(TestCollectionsTests)).OrderBy(n => n).ToList();
            Assert.Empty(offenders);
        }
    }
}
