using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using EmuSen.Galaxia;

namespace EmuSen.WiseMan.Crystal
{
    // A parity class's answers as recorded from the C# before it was deleted, one line a case, and the check of the Rust against them; EMUSEN_PARITY_GOLDEN=record rewrites the file - see EmuSen_Backend.md §33.
    public sealed class ParityGolden
    {
        public const string Variable = "EMUSEN_PARITY_GOLDEN";

        private readonly string _name;
        private readonly List<(string Key, string Answer)> _lines = new();

        public ParityGolden(string name) => _name = name;

        public static string Folder => Path.Combine(ConfigRoot.Managed.Directory, "EmuSen.WiseMan", "Crystal", "Goldens");

        public string File => Path.Combine(Folder, _name + ".golden.gz");

        public static bool Recording => Environment.GetEnvironmentVariable(Variable) == "record";

        public int Count => _lines.Count;

        public void Add(string key, string answer) => _lines.Add((key, answer));

        public void Add(string key, Func<object?> answer, Func<object?, string> shown)
        {
            string text;
            try { text = shown(answer()); }
            catch (Exception e) { text = $"threw {e.GetType().Name}: {e.Message}"; }
            _lines.Add((key, text));
        }

        private readonly Dictionary<string, (int Start, List<string> Answers)> _open = new();
        private int _index;

        // An answer of a sweep, held with the others of its group as one digest of at most size answers.
        public void AddChunked(string group, string answer, int size)
        {
            if (!_open.TryGetValue(group, out var chunk)) _open[group] = chunk = (_index, new List<string>());
            chunk.Answers.Add(answer);
            _index++;
            if (chunk.Answers.Count < size) return;
            Digest($"{group} from {chunk.Start}", chunk.Answers);
            _open.Remove(group);
        }

        private void Flush()
        {
            foreach (var (group, chunk) in _open.OrderBy(o => o.Value.Start)) Digest($"{group} from {chunk.Start}", chunk.Answers);
            _open.Clear();
        }

        // Many answers held as one digest, for a sweep whose answers are too many to keep as text.
        public void Digest(string key, IEnumerable<string> answers) =>
            _lines.Add((key, Convert.ToHexString(SHA256.HashData(Encoding.UTF8.GetBytes(string.Join("\n", answers))))[..16]));

        // Text with every control character, backslash and lone surrogate written as an escape, so one case is one line of UTF-8.
        public static string Escape(string text)
        {
            var b = new StringBuilder(text.Length);
            for (int i = 0; i < text.Length; i++)
            {
                char c = text[i];
                bool pair = char.IsHighSurrogate(c) && i + 1 < text.Length && char.IsLowSurrogate(text[i + 1]);
                if (pair) { b.Append(c).Append(text[++i]); continue; }
                if (c == '\\') b.Append("\\\\");
                else if (c < 0x20 || c == 0x7F || char.IsSurrogate(c)) b.Append("\\u").Append(((int)c).ToString("X4"));
                else b.Append(c);
            }
            return b.ToString();
        }

        public static string Unescape(string text)
        {
            var b = new StringBuilder(text.Length);
            for (int i = 0; i < text.Length; i++)
            {
                if (text[i] != '\\') { b.Append(text[i]); continue; }
                if (text[i + 1] == '\\') { b.Append('\\'); i++; continue; }
                b.Append((char)Convert.ToInt32(text.Substring(i + 2, 4), 16));
                i += 5;
            }
            return b.ToString();
        }

        // The recorded cases after the file's first line, which says what it is, each key and answer unescaped.
        public static IReadOnlyList<(string Key, string Answer)> Read(string name)
        {
            using var file = System.IO.File.OpenRead(Path.Combine(Folder, name + ".golden.gz"));
            using var gzip = new GZipStream(file, CompressionMode.Decompress);
            using var reader = new StreamReader(gzip, Encoding.UTF8);
            var lines = new List<(string, string)>();
            reader.ReadLine();
            while (reader.ReadLine() is { } line)
            {
                int tab = line.IndexOf('\t');
                lines.Add((Unescape(line[..tab]), Unescape(line[(tab + 1)..])));
            }
            return lines;
        }

        // Records the cases under the variable; otherwise holds them to the file, case by case, and fails naming the first that differ.
        public void Check(int least = 1)
        {
            Flush();
            Assert.True(_lines.Count >= least, $"{_name}: {_lines.Count} cases, fewer than {least}");
            if (Recording)
            {
                Directory.CreateDirectory(Folder);
                using var file = System.IO.File.Create(File);
                using var gzip = new GZipStream(file, CompressionLevel.SmallestSize);
                using var writer = new StreamWriter(gzip, new UTF8Encoding(false)) { NewLine = "\n" };
                writer.WriteLine($"# {_name}: answers recorded from the C# implementation on 2026-10-20, before it was deleted - see EmuSen_Backend.md §33.");
                foreach (var (key, answer) in _lines) writer.WriteLine(Escape(key) + "\t" + Escape(answer));
                return;
            }
            var recorded = Read(_name);
            var differences = new List<string>();
            if (recorded.Count != _lines.Count) differences.Add($"{recorded.Count} cases recorded, {_lines.Count} run");
            for (int i = 0; i < Math.Min(recorded.Count, _lines.Count) && differences.Count < 12; i++)
            {
                if (recorded[i].Key != _lines[i].Key) differences.Add($"case {i}: recorded '{Escape(recorded[i].Key)}', run '{Escape(_lines[i].Key)}'");
                else if (recorded[i].Answer != _lines[i].Answer) differences.Add($"{Escape(_lines[i].Key)}:\n  recorded: {Escape(recorded[i].Answer)}\n  now:      {Escape(_lines[i].Answer)}");
            }
            Assert.True(differences.Count == 0, $"{_name}: {differences.Count} difference(s) from the golden:\n" + string.Join("\n", differences));
        }
    }
}
