using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using EmuSen.WiseMan.Cores;

namespace EmuSen.WiseMan.Crystal
{
    // The parity class's answers as the C# implementation gave them before it was deleted on 2026-11-04, and the Rust's held to them case by case - see EmuSen_Backend.md §40.8.
    public partial class CrystalParityTests
    {
        // The implementation each case runs; the C# ran first here until it was deleted.
        private static IReadOnlyList<CrystalImpl> Impls => new[] { CrystalImpl.Native };

        private string? _goldenTest;
        private readonly List<(string Key, string Answer)> _goldenLines = new();
        private IReadOnlyList<(string Key, string Answer)>? _recorded;
        private int _goldenCalls;

        private string GoldenName => "CrystalParity." + _goldenTest;

        // One call's answers from the Rust, what only it gives set aside, held to the recorded C#'s.
        private void Settle(IReadOnlyList<List<string>> runs, string test)
        {
            if (_goldenTest is not null && _goldenTest != test) throw new InvalidOperationException($"one golden a case: {_goldenTest} and {test}");
            _goldenTest = test;
            int call = _goldenCalls++;
            List<string> answers = runs.Single().Select(AsCSharp).Select(Normal).ToList();
            var lines = answers.Select((a, k) => ($"{call}.{k}", a)).ToList();
            _goldenLines.AddRange(lines);
            if (ParityGolden.Recording) return;
            _recorded ??= ParityGolden.Read(GoldenName);
            var differences = new List<string>();
            var recorded = _recorded.Where(r => r.Key.StartsWith($"{call}.", StringComparison.Ordinal)).ToList();
            if (recorded.Count != lines.Count) differences.Add($"call {call}: {recorded.Count} answers recorded, {lines.Count} given");
            for (int k = 0; k < Math.Min(recorded.Count, lines.Count) && differences.Count < 8; k++)
                if (recorded[k].Answer != lines[k].Item2)
                    differences.Add($"answer {lines[k].Item1}:\n  recorded: {ParityGolden.Escape(recorded[k].Answer)}\n  now:      {ParityGolden.Escape(lines[k].Item2)}");
            foreach (string d in differences) _output.WriteLine(d);
            Assert.True(differences.Count == 0, $"{GoldenName}: {differences.Count} difference(s) from the recorded C# answers, the first: {differences.FirstOrDefault()}");
        }

        // Under EMUSEN_PARITY_GOLDEN=record the case's answers are written, now the Rust's, for a change reviewed answer by answer; otherwise every recorded call must have been made.
        private void FinishGolden()
        {
            if (_goldenTest is null) return;
            if (ParityGolden.Recording)
            {
                Directory.CreateDirectory(ParityGolden.Folder);
                using var file = File.Create(Path.Combine(ParityGolden.Folder, GoldenName + ".golden.gz"));
                using var gzip = new GZipStream(file, CompressionLevel.SmallestSize);
                using var writer = new StreamWriter(gzip, new UTF8Encoding(false)) { NewLine = "\n" };
                writer.WriteLine($"# {GoldenName}: the frontend API's answers recorded from the Rust implementation on {DateTime.UtcNow:yyyy-MM-dd}, after a reviewed change to the C#'s of 2026-11-04 - see EmuSen_Backend.md §40.8.");
                foreach (var (key, answer) in _goldenLines) writer.WriteLine(ParityGolden.Escape(key) + "\t" + ParityGolden.Escape(answer));
                return;
            }
            int made = _goldenLines.Count, held = (_recorded ?? ParityGolden.Read(GoldenName)).Count;
            Assert.True(made == held, $"{GoldenName}: {held} answers recorded, {made} given");
        }

        // The hashes of the libraries a case can list, each named by its file, since a rebuild changes them and nothing else.
        private static readonly Lazy<Dictionary<string, string>> LibraryHashes = new(() =>
        {
            var named = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            var files = new[] { CoreAdapterTests.LibraryPath("v1_test_core"), CoreAdapterTests.LibraryPath("v1_plain_core") }
                .Concat(new[] { AppContext.BaseDirectory, Path.Combine(AppContext.BaseDirectory, "cores") }.Where(Directory.Exists)
                    .SelectMany(d => Directory.EnumerateFiles(d)).Where(f => f.EndsWith(".so") || f.EndsWith(".dll") || f.EndsWith(".dylib")));
            foreach (string f in files.Where(File.Exists))
            {
                using var stream = File.OpenRead(f);
                named[Convert.ToHexString(SHA256.HashData(stream))] = Path.GetFileName(f);
            }
            return named;
        });

        // An answer with what differs from run to run made plain: the test's folder, the build's, a test tree's name and the libraries' hashes.
        private string Normal(string answer)
        {
            string build = AppContext.BaseDirectory.TrimEnd(Path.DirectorySeparatorChar);
            string text = answer.Replace(_root, "<root>", StringComparison.Ordinal).Replace(build, "<build>", StringComparison.Ordinal);
            text = System.Text.RegularExpressions.Regex.Replace(text, "cores[0-9a-f]{6}", "cores<tree>");
            text = System.Text.RegularExpressions.Regex.Replace(text, "(?m)^[0-9a-f]{16}/", "<unpacked>/");
            text = System.Text.RegularExpressions.Regex.Replace(text, "(?m)((^|/)(\\.unpacked|\\.engine-choices-copied|package\\.json)) [0-9A-F]{64}$", "$1 <written at run time>");
            return System.Text.RegularExpressions.Regex.Replace(text, "[0-9A-Fa-f]{64}", m => LibraryHashes.Value.TryGetValue(m.Value, out string? name) ? $"<sha256 of {name}>" : m.Value);
        }
    }
}
