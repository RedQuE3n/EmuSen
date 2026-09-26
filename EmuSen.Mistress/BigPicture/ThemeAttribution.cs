using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Mistress.BigPicture.Theme;

namespace EmuSen.Mistress.BigPicture
{
    // Who made an installed theme, under what licence, and where it came from, read from the theme's own files at display time so EmuSen carries no copy - see EmuSen_BigPicture.md §6 and §16.
    public sealed record ThemeAttribution(string Name, string Author, IReadOnlyList<string> Licence, IReadOnlyList<string> Credits, string Source, string? Commit, string Statement)
    {
        private static readonly string[] LicenceFiles = ["LICENSE", "LICENSE.md", "LICENSE.txt", "LICENCE", "LICENCE.md", "COPYING"];

        public static ThemeAttribution Read(string directory)
        {
            string name = ThemeCapabilitiesReader.Read(directory).ThemeName;
            ThemeStamp? stamp = ThemeDownloads.Stamp(directory);
            string? readme = new[] { "README.md", "README", "readme.md", "Readme.md" }.Select(f => Path.Combine(directory, f)).FirstOrDefault(File.Exists);
            IReadOnlyList<string> lines = readme is null ? [] : File.ReadAllLines(readme);

            IReadOnlyList<string> licence = Section(lines, "licen");
            if (licence.Count == 0 && LicenceFiles.Select(f => Path.Combine(directory, f)).FirstOrDefault(File.Exists) is { } file)
                licence = File.ReadLines(file).Select(Plain).Where(l => l.Length > 0).Take(12).ToList();
            if (licence.Count == 0) licence = ["The theme states no licence in a README or LICENSE file."];

            string author = stamp is not null ? $"{stamp.Owner}, the owner of its repository on GitHub" : "not stated; the folder was not downloaded by Mistress";
            string source = stamp is not null ? $"{stamp.Source.Url}, branch {stamp.Branch}" : $"read in place from {directory}";
            string statement = stamp is not null
                ? $"Downloaded at your request on {stamp.Downloaded:yyyy-MM-dd}. It is not part of EmuSen, and EmuSen distributes none of it."
                : "Read in place; Mistress never writes to this folder. It is not part of EmuSen, and EmuSen distributes none of it.";
            return new ThemeAttribution(name, author, licence, Section(lines, "credit"), source, stamp?.Commit, statement);
        }

        // The lines under the first Markdown heading whose words contain the key, up to the next heading of the same or a higher level.
        public static IReadOnlyList<string> Section(IReadOnlyList<string> lines, string key)
        {
            int start = -1, level = 0;
            for (int i = 0; i < lines.Count; i++)
            {
                Match m = Regex.Match(lines[i], @"^(#{1,6})\s+(.*)$");
                if (!m.Success) continue;
                if (start >= 0 && m.Groups[1].Length <= level) return Collect(lines, start, i);
                if (start < 0 && Plain(m.Groups[2].Value).Contains(key, StringComparison.OrdinalIgnoreCase))
                {
                    start = i + 1;
                    level = m.Groups[1].Length;
                }
            }
            return start < 0 ? [] : Collect(lines, start, lines.Count);
        }

        private static IReadOnlyList<string> Collect(IReadOnlyList<string> lines, int from, int to) =>
            lines.Skip(from).Take(to - from).Select(Plain).Where(l => l.Length > 0).ToList();

        // Markdown's marks taken off: emphasis, images, links as "text (address)", bullets as a dot.
        public static string Plain(string line)
        {
            string s = Regex.Replace(line, @"!\[[^\]]*\]\([^)]*\)", "");
            s = Regex.Replace(s, @"\[([^\]]*)\]\(([^)]*)\)", "$1 ($2)");
            s = Regex.Replace(s, @"(\*\*|__|`)", "");
            s = Regex.Replace(s, @"^\s*[*\-+]\s+", "• ");
            s = Regex.Replace(s, @"^#+\s*", "");
            s = Regex.Replace(s, @"<[^>]+>", "");
            return s.Trim();
        }
    }
}
