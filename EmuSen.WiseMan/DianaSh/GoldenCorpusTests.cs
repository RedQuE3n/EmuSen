using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.DianaSh
{
    // The C# shell's outputs, held so a second shell can be checked against them byte for byte - see EmuSen_DianaSh.md §13.4.
    [Collection(TestCollections.ProcessGlobals)]
    public class GoldenCorpusTests
    {
        private static string Here([CallerFilePath] string path = "") => Path.GetDirectoryName(path)!;

        private static string Corpus => Path.Combine(Here(), "Golden");

        private static string Repo => Path.GetFullPath(Path.Combine(Here(), "..", ".."));

        private static string CommandSources => Path.Combine(Repo, "EmuSen.DianaSh", "DianaSh", "Bin", "Commands");

        private static bool Recording => Environment.GetEnvironmentVariable("EMUSEN_DIANASH_GOLDEN") == "record";

        // The builtins Dispatch answers itself, with the help line each one's usage starts at.
        private static readonly (string File, string Prefix)[] Builtins =
        {
            ("help", "  help "), ("man", "  man "), ("summary", "  summary "), ("export", "  export "),
            ("unset", "  unset "), ("source", "  source "), ("which", "  which "), ("command", "  command "), ("trust", "  trust "), ("shell", "  if/then/"),
        };

        public static string FileName(string command) => command == "[" ? "bracket" : command;

        // A command's usage lines as the live shell prints them; a builtin's are its lines of `help`.
        public static IReadOnlyList<string> LiveUsage(string command)
        {
            IDianaShCommand? found = GoldenRunner.Commands().FirstOrDefault(c => c.Name == command);
            if (found is not null) return found.Usage.Split('\n');
            string prefix = Builtins.Single(b => b.File == command).Prefix;
            string[] help = DianaShInterpreter.CreateDefault(null).Submit("help").Output.Split('\n');
            int at = Array.FindIndex(help, l => l.StartsWith(prefix, StringComparison.Ordinal));
            var lines = new List<string> { help[at] };
            while (at + 1 < help.Length && help[at + 1].StartsWith("                                ", StringComparison.Ordinal)) lines.Add(help[++at]);
            return lines;
        }

        public static IEnumerable<string> Expected() =>
            GoldenRunner.Commands().Select(c => FileName(c.Name)).Concat(Builtins.Select(b => b.File)).Distinct();

        public static IEnumerable<object[]> Files() =>
            Directory.EnumerateFiles(Corpus, "*.golden").Select(Path.GetFileName).Order(StringComparer.Ordinal).Select(f => new object[] { f! });

        private static GoldenFile Load(string name) => GoldenFile.Parse(File.ReadAllBytes(Path.Combine(Corpus, name)));

        // Whether this run's shell has the file's command: a file marked for the Rust shell is that shell's alone.
        private static bool Runs(GoldenFile file) => file.Shell != "rust" || EmuSen.DianaSh.DianaSh.Bin.Native.DianaShNative.Active;

        // Every case of the file run again, its usage taken live, its patterns kept where they still hold.
        public static GoldenFile Record(GoldenFile file, GoldenRunner runner)
        {
            var recorded = new GoldenFile { Command = file.Command, Shell = file.Shell, Unreachable = file.Unreachable, Uncovered = file.Uncovered, Usage = LiveUsage(file.Command).ToList() };
            foreach (GoldenCase c in file.Cases)
            {
                List<GoldenStep> run = runner.Run(c);
                var copy = new GoldenCase { Title = c.Title, Target = c.Target, Frames = c.Frames, Covers = c.Covers, Files = c.Files, VaryNumbers = c.VaryNumbers, NoSessions = c.NoSessions, NoWindow = c.NoWindow };
                for (int i = 0; i < run.Count; i++) copy.Steps.Add(GoldenRunner.Recorded(run[i], c.Steps[i], c.VaryNumbers));
                recorded.Cases.Add(copy);
            }
            return recorded;
        }

        [Theory]
        [MemberData(nameof(Files))]
        public void A_command_prints_what_its_recording_holds(string name)
        {
            GoldenFile file = Load(name);
            if (!Runs(file)) return;
            using var runner = new GoldenRunner();
            if (Recording)
            {
                File.WriteAllBytes(Path.Combine(Corpus, name), Record(file, runner).Write());
                return;
            }
            // Every case is run and every difference named, since a message cut short names only the first case.
            var differences = file.Cases.Select(c => (Case: c, Run: runner.Run(c))).Where(r => !runner.SystemsCase).Select(r => GoldenRunner.Compare(name, r.Case, r.Run)).OfType<string>().ToList();
            Assert.True(differences.Count == 0, string.Join("\n\n", differences));
        }

        [Fact]
        public void Every_command_and_builtin_has_a_file_of_its_own()
        {
            var have = Files().Where(f => Runs(Load((string)f[0]))).Select(f => Path.GetFileNameWithoutExtension((string)f[0])).ToHashSet();
            Assert.Empty(Expected().Where(e => !have.Contains(e)));
            Assert.Empty(have.Where(h => !Expected().Contains(h)));
            foreach (string name in have) Assert.Equal(name, FileName(Load(name + ".golden").Command));
        }

        [Fact]
        public void Every_usage_line_is_covered_by_a_case()
        {
            var missing = new List<string>();
            foreach (object[] row in Files())
            {
                GoldenFile file = Load((string)row[0]);
                if (!Runs(file)) continue;
                IReadOnlyList<string> live = LiveUsage(file.Command);
                if (!live.SequenceEqual(file.Usage)) missing.Add($"{row[0]}: its recorded usage lines are not the live ones - record it again");
                var covered = file.Cases.SelectMany(c => c.Covers).Concat(file.Uncovered.Select(u => u.Line)).ToHashSet();
                for (int i = 1; i <= live.Count; i++)
                    if (!covered.Contains(i)) missing.Add($"{row[0]}: usage line {i} has no case: {live[i - 1].Trim()}");
                foreach (int line in file.Cases.SelectMany(c => c.Covers).Where(n => n < 1 || n > live.Count)) missing.Add($"{row[0]}: covers {line}, which is not a usage line");
            }
            Assert.True(missing.Count == 0, string.Join('\n', missing));
        }

        [Fact]
        public void Every_manual_page_is_recorded()
        {
            var pages = ManPages.Names.ToList();
            var asked = Load("man.golden").Cases.SelectMany(c => c.Steps).Select(s => s.Input).ToHashSet();
            Assert.Empty(pages.Where(p => !asked.Contains($"man {p}")));
            Assert.True(pages.Count >= 98, $"{pages.Count} pages found in {ManPages.Directory}");
        }

        // Each Fail("...") the code words must appear in its command's outputs, or be listed as one no case can reach.
        [Fact]
        public void Every_failure_the_code_words_is_printed_by_a_case()
        {
            var outputs = new Dictionary<string, string>();
            var unreachable = new Dictionary<string, HashSet<string>>();
            foreach (object[] row in Files())
            {
                GoldenFile file = Load((string)row[0]);
                string name = Path.GetFileNameWithoutExtension((string)row[0]);
                outputs[name] = string.Join('\n', file.Cases.SelectMany(c => c.Steps).Select(s => s.Output is null ? s.Pattern ?? "" : Encoding.UTF8.GetString(s.Output)));
                unreachable[name] = file.Unreachable.Select(u => u.Fragment).ToHashSet();
            }
            string all = string.Join('\n', outputs.Values);
            var allUnreachable = unreachable.Values.SelectMany(u => u).ToHashSet();

            var bySource = GoldenRunner.Commands().GroupBy(c => c.GetType().Name).ToDictionary(g => g.Key, g => g.Select(c => FileName(c.Name)).ToList());
            var missing = new List<string>();
            foreach (string path in Directory.EnumerateFiles(CommandSources, "*.cs", SearchOption.AllDirectories).Append(Path.Combine(Repo, "EmuSen.DianaSh", "DianaSh", "Bin", "DianaShInterpreter.cs")))
            {
                string source = File.ReadAllText(path);
                string type = Path.GetFileNameWithoutExtension(path);
                List<string>? files = bySource.TryGetValue(type, out var f) ? f : null;
                if (type == "TestCommand") files = new List<string> { "test", "bracket" };
                foreach (string fragment in FailFragments(source))
                {
                    bool printed = files is null ? all.Contains(fragment) : files.Any(n => outputs.GetValueOrDefault(n, "").Contains(fragment));
                    bool listed = files is null ? allUnreachable.Contains(fragment) : files.Any(n => unreachable.GetValueOrDefault(n)?.Contains(fragment) == true);
                    if (!printed && !listed) missing.Add($"{Path.GetFileName(path)}: \"{fragment}\"");
                }
            }
            Assert.True(missing.Count == 0, $"{missing.Count} failures no case prints:\n" + string.Join('\n', missing));
        }

        [Fact]
        public void Two_recordings_of_the_corpus_are_identical()
        {
            foreach (object[] row in Files())
            {
                GoldenFile file = Load((string)row[0]);
                if (!Runs(file)) continue;
                byte[] first, second;
                using (var runner = new GoldenRunner()) first = Record(file, runner).Write();
                using (var runner = new GoldenRunner()) second = Record(file, runner).Write();
                int n = 0;
                while (n < first.Length && n < second.Length && first[n] == second[n]) n++;
                Assert.True(n == first.Length && n == second.Length, $"{row[0]}: two recordings first differ at byte {n}");
            }
        }

        // The literal text of every Fail(...) call, cut at each interpolation hole and each line break; pieces under 8 characters are skipped.
        public static IEnumerable<string> FailFragments(string source)
        {
            foreach (Match call in Regex.Matches(source, @"\bFail\("))
            {
                int at = call.Index + call.Length, depth = 1;
                var pieces = new List<string>();
                while (at < source.Length && depth > 0)
                {
                    char ch = source[at];
                    if (ch == '(') { depth++; at++; }
                    else if (ch == ')') { depth--; at++; }
                    else if (ch == '"' || (ch == '$' && at + 1 < source.Length && source[at + 1] == '"'))
                    {
                        bool interpolated = ch == '$';
                        at += interpolated ? 2 : 1;
                        var piece = new StringBuilder();
                        while (at < source.Length && source[at] != '"')
                        {
                            char c = source[at];
                            if (c == '\\' && at + 1 < source.Length)
                            {
                                char e = source[at + 1];
                                piece.Append(e switch { 'n' => '\n', 't' => '\t', _ => e });
                                at += 2;
                            }
                            else if (interpolated && c == '{' && at + 1 < source.Length && source[at + 1] == '{') { piece.Append('{'); at += 2; }
                            else if (interpolated && c == '}' && at + 1 < source.Length && source[at + 1] == '}') { piece.Append('}'); at += 2; }
                            else if (interpolated && c == '{')
                            {
                                pieces.Add(piece.ToString());
                                piece.Clear();
                                at = SkipHole(source, at);
                            }
                            else { piece.Append(c); at++; }
                        }
                        pieces.Add(piece.ToString());
                        at++;
                    }
                    else at++;
                }
                foreach (string piece in pieces.SelectMany(p => p.Split('\n')).Select(p => p.Trim()).Where(p => p.Length >= 8))
                    yield return piece;
            }
        }

        private static int SkipHole(string source, int at)
        {
            int depth = 0;
            while (at < source.Length)
            {
                char c = source[at];
                if (c == '{') depth++;
                else if (c == '}' && --depth == 0) return at + 1;
                else if (c == '"')
                {
                    at++;
                    while (at < source.Length && source[at] != '"') at += source[at] == '\\' ? 2 : 1;
                }
                else if (c == '\'')
                {
                    at++;
                    while (at < source.Length && source[at] != '\'') at += source[at] == '\\' ? 2 : 1;
                }
                at++;
            }
            return at;
        }
    }
}
