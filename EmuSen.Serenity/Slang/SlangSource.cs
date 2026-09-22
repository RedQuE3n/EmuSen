using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;

namespace EmuSen.Serenity.Slang
{
    // One #pragma parameter: its id, what it is called, its default and its range - see EmuSen_Serenity.md §7.2.
    public sealed record SlangParameter(string Id, string Description, float Initial, float Minimum, float Maximum, float Step);

    // A .slang file with its includes in place, split into its two stages, and what its pragmas declared - see EmuSen_Serenity.md §7.2.
    public sealed class SlangSource
    {
        private static readonly Regex Include = new(@"^\s*#\s*include\s+""([^""]+)""", RegexOptions.Compiled);
        private static readonly Regex OptionalInclude = new(@"^\s*#\s*pragma\s+include_optional\s+""([^""]+)""", RegexOptions.Compiled);
        private static readonly Regex Stage = new(@"^\s*#\s*pragma\s+stage\s+(vertex|fragment)\b", RegexOptions.Compiled);
        private static readonly Regex Name = new(@"^\s*#\s*pragma\s+name\s+(\S+)", RegexOptions.Compiled);
        private static readonly Regex Format = new(@"^\s*#\s*pragma\s+format\s+(\S+)", RegexOptions.Compiled);
        private static readonly Regex Parameter = new(@"^\s*#\s*pragma\s+parameter\s+(\S+)\s+""([^""]*)""\s+(\S+)\s+(\S+)\s+(\S+)(?:\s+(\S+))?", RegexOptions.Compiled);

        public const int MaxIncludeDepth = 32;

        public string Path { get; }
        public string Vertex { get; }
        public string Fragment { get; }
        public string? PassName { get; }
        public string? FramebufferFormat { get; }
        public IReadOnlyList<SlangParameter> Parameters { get; }

        private SlangSource(string path, string vertex, string fragment, string? name, string? format, IReadOnlyList<SlangParameter> parameters)
        {
            Path = path;
            Vertex = vertex;
            Fragment = fragment;
            PassName = name;
            FramebufferFormat = format;
            Parameters = parameters;
        }

        public static SlangSource Load(string path)
        {
            var lines = new List<string>();
            Expand(System.IO.Path.GetFullPath(path), lines, 0);

            var vertex = new StringBuilder();
            var fragment = new StringBuilder();
            var parameters = new List<SlangParameter>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            string? name = null, format = null;
            string? stage = null;

            // A pragma line is kept as a blank so the compiler's line numbers still point at the right line of the expanded text.
            foreach (string line in lines)
            {
                string keep = line;
                if (Stage.Match(line) is { Success: true } s) { stage = s.Groups[1].Value; keep = ""; }
                else if (Name.Match(line) is { Success: true } n) { name = n.Groups[1].Value; keep = ""; }
                else if (Format.Match(line) is { Success: true } f) { format = f.Groups[1].Value; keep = ""; }
                else if (Parameter.Match(line) is { Success: true } p)
                {
                    if (seen.Add(p.Groups[1].Value))
                        parameters.Add(new SlangParameter(p.Groups[1].Value, p.Groups[2].Value, Number(p.Groups[3].Value), Number(p.Groups[4].Value),
                            Number(p.Groups[5].Value), p.Groups[6].Success ? Number(p.Groups[6].Value) : 0f));
                    keep = "";
                }

                if (stage is null or "vertex") vertex.Append(keep).Append('\n');
                else vertex.Append('\n');
                if (stage is null or "fragment") fragment.Append(keep).Append('\n');
                else fragment.Append('\n');
            }

            if (stage is null) throw new InvalidDataException($"{path} has no #pragma stage.");
            return new SlangSource(System.IO.Path.GetFullPath(path), vertex.ToString(), fragment.ToString(), name, format, parameters);
        }

        private static float Number(string text) =>
            float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out float value) ? value : 0f;

        // Includes are textual and relative to the file that names them; an optional one that is absent is simply skipped.
        private static void Expand(string path, List<string> lines, int depth)
        {
            if (depth > MaxIncludeDepth) throw new InvalidDataException($"{path}: includes go deeper than {MaxIncludeDepth}; taken to be a loop.");
            if (!File.Exists(path)) throw new FileNotFoundException($"The shader file {path} does not exist.", path);
            string directory = System.IO.Path.GetDirectoryName(path)!;
            foreach (string line in File.ReadLines(path))
            {
                if (Include.Match(line) is { Success: true } include)
                {
                    Expand(System.IO.Path.GetFullPath(System.IO.Path.Combine(directory, include.Groups[1].Value)), lines, depth + 1);
                    continue;
                }
                if (OptionalInclude.Match(line) is { Success: true } optional)
                {
                    string target = System.IO.Path.GetFullPath(System.IO.Path.Combine(directory, optional.Groups[1].Value));
                    if (File.Exists(target)) Expand(target, lines, depth + 1);
                    continue;
                }
                lines.Add(line);
            }
        }
    }
}
