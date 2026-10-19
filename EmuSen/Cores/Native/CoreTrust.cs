using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Native
{
    // The installation's cores.manifest and the player's cores.approved, one sha256sum format for both - see EmuSen_Backend.md §29.40.3.
    public static class CoreTrust
    {
        public const string ManifestName = "cores.manifest";
        public const string ApprovalsName = "cores.approved";

        // The approvals' file in the config directory, so a test's follows its config override.
        public static string ApprovalsPath => ConfigStore.For(ApprovalsName);

        public static bool IsHash(string text) => text.Length == 64 && text.All(Uri.IsHexDigit);

        // The lines that read as <64 hex digits>  <file name> (or " *name"), the hash lowercased; others are skipped.
        public static IEnumerable<(string Sha256, string Name)> Parse(string text)
        {
            foreach (string raw in text.Split('\n'))
            {
                string line = raw.EndsWith('\r') ? raw[..^1] : raw;
                if (line.Length < 64 || !IsHash(line[..64])) continue;
                string rest = line[64..];
                string? name = rest.StartsWith("  ", StringComparison.Ordinal) || rest.StartsWith(" *", StringComparison.Ordinal) ? rest[2..] : null;
                if (string.IsNullOrEmpty(name) || name.Contains('/')) continue;
                yield return (line[..64].ToLowerInvariant(), name);
            }
        }

        // The hashes the file approves, none where it is absent; refused, with the words said, where others may write it.
        public static IReadOnlyList<string> ReadApprovals(string path, Action<string>? say = null)
        {
            if (!File.Exists(path)) return Array.Empty<string>();
            if (!OperatingSystem.IsWindows() && (File.GetUnixFileMode(path) & UnixFileMode.OtherWrite) != 0)
            {
                say?.Invoke($"{path} is writable by others; its approvals were not read");
                return Array.Empty<string>();
            }
            try { return Parse(Encoding.UTF8.GetString(File.ReadAllBytes(path))).Select(e => e.Sha256).ToArray(); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { say?.Invoke($"{path}: {e.Message}; its approvals were not read"); return Array.Empty<string>(); }
        }

        // The text with the hash added under the file name, every other line kept; null where it is there already.
        public static string? WithApproval(string text, string sha256, string name)
        {
            string hash = sha256.ToLowerInvariant();
            if (Parse(text).Any(e => e.Sha256 == hash)) return null;
            if (name.Length == 0 || name.Contains('/') || name.Any(char.IsControl)) name = "-";
            var out_ = new StringBuilder(text);
            if (out_.Length > 0 && text[^1] != '\n') out_.Append('\n');
            return out_.Append(hash).Append("  ").Append(name).Append('\n').ToString();
        }

        // The text without the lines of the hash, every other line kept; null where it has none.
        public static string? WithoutApproval(string text, string sha256)
        {
            string hash = sha256.ToLowerInvariant();
            var out_ = new StringBuilder();
            bool removed = false;
            int at = 0;
            while (at < text.Length)
            {
                int end = text.IndexOf('\n', at);
                string line = end < 0 ? text[at..] : text[at..(end + 1)];
                at += line.Length;
                if (Parse(line).Any(e => e.Sha256 == hash)) removed = true;
                else out_.Append(line);
            }
            return removed ? out_.ToString() : null;
        }

        // The manifest of an installation directory, by file name; empty where it has none.
        public static IReadOnlyList<(string Sha256, string Name)> ManifestOf(string directory)
        {
            try { return Parse(File.ReadAllText(Path.Combine(directory, ManifestName))).ToArray(); }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException) { return Array.Empty<(string, string)>(); }
        }
    }
}
