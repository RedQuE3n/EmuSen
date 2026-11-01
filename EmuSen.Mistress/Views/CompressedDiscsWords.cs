using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using EmuSen.Tools.Packages;

namespace EmuSen.Mistress.Views
{
    // The CHD tool's states, refusals and failures in the Compressed Discs window's words - see EmuSen_GamePackages.md §13.3.
    public static class CompressedDiscsWords
    {
        public static string Refusal(string tool) => ToolWords.Refusal(tool);

        public static string Reason(string tool) => ToolWords.Reason(tool);

        // A disc's state line: what it is, its size, and why where the tool said why.
        public static string State(CompressedDisc d, string op)
        {
            string shrunk = d.Original > 0 && d.Chd > 0 ? $": {Size(d.Chd)}, {(d.Chd * 100.0 / d.Original).ToString("F0", CultureInfo.InvariantCulture)}% of the original" : "";
            string why = d.Words is { Length: > 0 } w ? $": {Reason(w)}" : "";
            string beside = d.Where == "library" ? ", beside the original" : "";
            return d.State switch
            {
                "waiting" => "Waiting",
                "working" => op == "check" ? "Checking..." : "Converting and checking...",
                "converted" when d.Seconds is { } s => $"Converted and verified{beside}{shrunk}, in {s.ToString("0.#", CultureInfo.InvariantCulture)} s",
                "converted" => $"Converted and verified{beside}{shrunk}",
                "already" => $"Already converted and verified{beside}",
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
        public static string Summary(string op, IReadOnlyDictionary<string, long> counts, string seconds, bool beside = false)
        {
            static string N(long n, string one, string many) => $"{n} {(n == 1 ? one : many)}";
            var parts = new List<string>();
            foreach ((string state, string one, string many) in op == "check"
                ? new[] { ("verified", "disc verified", "discs verified"), ("failed", "failed", "failed"), ("not converted", "not converted", "not converted"), ("stopped", "stopped", "stopped") }
                : new[] { ("converted", "disc converted and verified", "discs converted and verified"), ("already", "already converted", "already converted"), ("failed", "failed", "failed"), ("skipped", "left as it is", "left as they are"), ("stopped", "stopped", "stopped") })
                if (counts.TryGetValue(state, out long n) && n > 0) parts.Add(N(n, one, many));
            string what = parts.Count == 0 ? "No disc" : string.Join(", ", parts);
            long added = counts.TryGetValue("converted", out long c) ? c : 0;
            string tail = op != "convert" ? "" : beside ? $" {added} CHD file{(added == 1 ? " was" : "s were")} added to the library beside {(added == 1 ? "its original" : "their originals")}; no original was changed." : " Nothing in the library was changed.";
            return $"{char.ToUpperInvariant(what[0])}{what[1..]}, in {seconds} s.{tail}";
        }

        public static string Size(long bytes) => ToolWords.Size(bytes);
    }
}
