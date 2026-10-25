using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;

namespace EmuSen.Mistress.Views
{
    // The CHD tool's states, refusals and failures in the Compressed Discs window's words - see EmuSen_GamePackages.md §13.3.
    public static class CompressedDiscsWords
    {
        // The tool's words for why a job did nothing, each as the window says it; the tool's own are kept where none matches.
        private static readonly (Regex Tool, Func<Match, string> Said)[] Refusals =
        [
            (new(@"^AppSettings\.RomDirectory is not set"), _ => "No library folder is set, so there are no discs to convert. Choose the library's folder in Preferences, then open this again."),
            (new(@"^The library (.+) is not a folder\.$"), m => $"The library folder {m.Groups[1].Value} is not there, so nothing was done. Is its drive connected?"),
            (new(@"^The tree names no (\w+) folder\.$"), m => $"EmuSen could not find its own {m.Groups[1].Value} folder, so nothing was done."),
            (new(@"^(\w+) \((.+)\) and the library \((.+)\) overlap"), m => $"EmuSen's {m.Groups[1].Value} folder ({m.Groups[2].Value}) and the library ({m.Groups[3].Value}) overlap. The library is never written to, so nothing was done: keep the library outside EmuSen's own folders."),
            (new(@"^chdman was not found at (.+?) \("), m => $"chdman, the program that writes CHD files, is not beside EmuSen (it was looked for at {m.Groups[1].Value}), so nothing was converted. EmuSen's own builds carry it."),
            (new(@"^chdman at (.+) has SHA-256 \w+, but this build is pinned to"), _ => "The chdman beside EmuSen is not the one this build was made with, so it was not run and nothing was converted. Installing EmuSen again puts the right one back."),
            (new(@"^chdman\.pin has no chdman for (.+?):"), m => $"This build of EmuSen carries no chdman for this kind of machine ({m.Groups[1].Value}), so discs cannot be converted here."),
            (new(@"^chdman at (.+) could not be run: (.+)$"), m => $"chdman could not be started ({m.Groups[2].Value}), so nothing was converted. On Linux it needs SDL2 (libSDL2-2.0.so.0) installed."),
            (new(@"^chdman at (.+) printed no version line: (.+)$"), m => $"chdman did not answer as chdman does ({m.Groups[2].Value}), so nothing was converted."),
            (new(@"^(.+): no such file or folder in the library$"), m => $"{m.Groups[1].Value} is no longer in the library, so it was left out."),
        ];

        // The tool's words for what became of one disc, each as the window says it after the state.
        private static readonly (Regex Tool, Func<Match, string> Said)[] Reasons =
        [
            (new(@"^a check failed$"), _ => "a check did not pass"),
            (new(@"^(\d+) bytes are free at (.+), and the disc needs (\d+) with the margin$"), m => $"there is not room for it: {Size(long.Parse(m.Groups[1].Value, CultureInfo.InvariantCulture))} free at {m.Groups[2].Value}, and it needs {Size(long.Parse(m.Groups[3].Value, CultureInfo.InvariantCulture))}"),
            (new(@"^the original changed during the conversion"), _ => "its files in the library changed while it was converting, so the CHD was thrown away; converting it again starts afresh"),
            (new(@"^the original could not be hashed: (.+)$"), m => $"its files could not be read ({m.Groups[1].Value})"),
            (new(@"^the original does not read as a disc: (.+)$"), m => $"it does not read as a disc ({m.Groups[1].Value})"),
            (new(@"^chdman createcd failed: (.+)$"), m => $"chdman could not make the CHD ({m.Groups[1].Value})"),
            (new(@"^chdman info failed: (.+)$"), m => $"chdman could not read the CHD it made ({m.Groups[1].Value})"),
            (new(@"^chdman could not be run: (.+)$"), m => $"chdman could not be started ({m.Groups[1].Value})"),
            (new(@"^(.+) holds (\d+) track files and no sheet"), m => $"{m.Groups[1].Value} holds {m.Groups[2].Value} track files and no cue sheet, so the order of its tracks and the gaps between them are unknown"),
            (new(@"^the sheet names a WAVE file"), _ => "its cue sheet names a WAVE file, whose header a CHD does not keep"),
            (new(@"^an \.m3u is not converted"), _ => "a playlist (.m3u) is not converted as one: choose each of its discs instead"),
            (new(@"^the sheet names (.+), which is not there$"), m => $"its cue sheet names {m.Groups[1].Value}, which is not in the library"),
            (new(@"^the sheet names (.+), which is not a file beside it$"), m => $"its cue sheet names {m.Groups[1].Value}, which is not a file beside the sheet"),
            (new(@"^(.+) is neither a raw data track nor an ISO 9660 image$"), m => $"{m.Groups[1].Value} is neither a disc's data track nor an ISO image"),
            (new(@"^(.+) is not a whole number of ([\d,]+)-byte sectors$"), m => $"{m.Groups[1].Value} is not a whole number of {m.Groups[2].Value}-byte sectors, as if it were cut short"),
            (new(@"^the CHD could not be put in place: (.+)$"), m => $"the finished CHD could not be put in place ({m.Groups[1].Value})"),
            (new(@"^the CHD (.+) is not there$"), m => $"its CHD, {m.Groups[1].Value}, is not there"),
            (new(@"^not converted$"), _ => "it is not converted"),
            (new(@"^(.+) could not be made: (.+)$"), m => $"the folder {m.Groups[1].Value} could not be made ({m.Groups[2].Value})"),
        ];

        public static string Refusal(string tool) => Say(Refusals, tool) ?? $"Nothing was done: {tool}";

        public static string Reason(string tool) => Say(Reasons, tool) ?? tool;

        private static string? Say((Regex Tool, Func<Match, string> Said)[] table, string tool)
        {
            foreach (var (pattern, said) in table)
                if (pattern.Match(tool) is { Success: true } m) return said(m);
            return null;
        }

        // A disc's state line: what it is, its size, and why where the tool said why.
        public static string State(CompressedDisc d, string op)
        {
            string shrunk = d.Original > 0 && d.Chd > 0 ? $": {Size(d.Chd)}, {d.Chd * 100.0 / d.Original:F0}% of {Size(d.Original)}" : "";
            string why = d.Words is { Length: > 0 } w ? $": {Reason(w)}" : "";
            return d.State switch
            {
                "waiting" => "Waiting",
                "working" => op == "check" ? "Checking..." : "Converting and checking...",
                "converted" when d.Seconds is { } s => $"Converted and verified{shrunk}, in {s.ToString("0.#", CultureInfo.InvariantCulture)} s",
                "converted" => $"Converted and verified{shrunk}",
                "already" => "Already converted and verified",
                "verified" => "Verified again: all three checks passed",
                "not converted" when op == "check" => "Not converted, so not checked",
                "not converted" => "Not converted",
                "changed" => "Changed since it was converted: converting it again replaces its CHD",
                "missing" => "Its CHD is gone: converting it again makes a new one",
                "skipped" => $"Left as it is{why}",
                "failed" when op == "check" => $"Failed the check{why}",
                "failed" => $"Failed, and no CHD was kept{why}",
                "stopped" => "Stopped before it finished: Resume converts it",
                _ => d.State + why,
            };
        }

        // The three checks' results, each named as a player would read it; null where the event carried none.
        public static string? Checks(JsonElement checks)
        {
            if (checks.ValueKind != JsonValueKind.Object) return null;
            var said = new List<string>();
            foreach ((string key, string name) in new[] { ("chdman", "chdman's verify"), ("tracks", "every track against the original"), ("toc", "the table of contents") })
            {
                if (!checks.TryGetProperty(key, out JsonElement v) || v.GetString() is not { } word) continue;
                said.Add(word == "passed" ? $"{name}: passed" : word.StartsWith("failed: ", StringComparison.Ordinal) ? $"{name}: failed, {word["failed: ".Length..]}" : $"{name}: {word}");
            }
            return said.Count == 0 ? null : string.Join("; ", said) + ".";
        }

        // What a finished batch did, each state counted that came up.
        public static string Summary(string op, IReadOnlyDictionary<string, long> counts, string seconds)
        {
            static string N(long n, string one, string many) => $"{n} {(n == 1 ? one : many)}";
            var parts = new List<string>();
            foreach ((string state, string one, string many) in op == "check"
                ? new[] { ("verified", "disc verified", "discs verified"), ("failed", "failed", "failed"), ("not converted", "not converted", "not converted"), ("stopped", "stopped", "stopped") }
                : new[] { ("converted", "disc converted and verified", "discs converted and verified"), ("already", "already converted", "already converted"), ("failed", "failed", "failed"), ("skipped", "left as it is", "left as they are"), ("stopped", "stopped", "stopped") })
                if (counts.TryGetValue(state, out long n) && n > 0) parts.Add(N(n, one, many));
            string what = parts.Count == 0 ? "No disc" : string.Join(", ", parts);
            string tail = op == "convert" ? " Nothing in the library was changed." : "";
            return $"{char.ToUpperInvariant(what[0])}{what[1..]}, in {seconds} s.{tail}";
        }

        // Bytes as a player reads them, in units of 1,024.
        public static string Size(long bytes)
        {
            string[] units = ["bytes", "KB", "MB", "GB", "TB"];
            double value = bytes;
            int unit = 0;
            while (value >= 1024 && unit < units.Length - 1) { value /= 1024; unit++; }
            return unit == 0 ? bytes.ToString("N0", CultureInfo.InvariantCulture) + " bytes" : $"{value.ToString(value >= 100 ? "0" : "0.0", CultureInfo.InvariantCulture)} {units[unit]}";
        }
    }
}
