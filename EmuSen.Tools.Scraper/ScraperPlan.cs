using EmuSen.Galaxia.Native;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.LunaP.Controls;

namespace EmuSen.Tools.Scraper
{
    // What a run asks beside its games: the scraper's default kinds, every picture a theme can draw, then videos and overwrite (ES-DE's "Overwrite files and data").
    public sealed record ScrapeOptions(bool Videos = false, bool Overwrite = false);

    // The Scraper's words and the command lines it asks, free of any control so that they are tested alone - see EmuSen_LibraryTools.md §9.
    public static class ScraperPlan
    {
        public static readonly ManagerMark Scrape = new("scrape", "Scrape");

        // The kinds a game is said to have none of in the "No game image" section, ES-DE's own four.
        public const string GameImage = "miximage,screenshot,titlescreen,cover";

        public static IReadOnlyList<ManagerMark> MarksFor(ScrapeRow row) => [Scrape];

        // A section's choice as the command's words: the module, not the window, chooses the games.
        public static IReadOnlyList<string> ChoiceWords(string? section) => section switch
        {
            null or "all" => [],
            "favorites" => ["--favorites"],
            "missing" => ["--missing", "any"],
            "noimage" => ["--missing", GameImage],
            "nometadata" => ["--missing", "metadata"],
            _ when section.StartsWith("system:", StringComparison.Ordinal) => ["--system", section["system:".Length..]],
            _ when section.StartsWith("collection:", StringComparison.Ordinal) => ["--collection", section["collection:".Length..]],
            _ => [],
        };

        // The kinds' words, which ls, plan and run share.
        public static IReadOnlyList<string> KindWords(ScrapeOptions o)
        {
            var words = new List<string>();
            if (o.Videos) words.Add("--videos");
            return words;
        }

        // The tool's one-line state: the games the theme's kinds find without art, else that every game has it; null before anything is listed (EmuSen_Interface.md §19).
        public static ToolSummary? StateOf(IReadOnlyCollection<ScrapeRow> rows)
        {
            if (rows.Count == 0) return null;
            int lacking = rows.Count(r => r.Media.Count > 0 && r.Missing.Count > 0);
            return lacking > 0 ? new($"{lacking:N0} without art", $"{lacking:N0} no art", Sentence: (lacking == 1 ? "1 game has" : $"{lacking:N0} games have") + " no art yet; the Scraper fetches it from ScreenScraper.")
                : new("Every game has art", Sentence: "Every game in the library has its art.");
        }

        public static List<string> Line(string sub, string? section, ScrapeOptions o, bool byHand = false) => Line(sub, section, null, o, byHand);

        // The system's words and the criterion's or collection's, so a filter narrows a system's games.
        public static List<string> Line(string sub, string? section, string? filter, ScrapeOptions o, bool byHand = false)
        {
            var words = new List<string> { sub };
            if (!byHand) words.AddRange(ChoiceWords(section));
            if (!byHand) words.AddRange(ChoiceWords(filter));
            words.AddRange(KindWords(o));
            if (o.Overwrite && sub != "ls") words.Add("--overwrite");
            return words;
        }

        // A source word of the scraper's in the player's words.
        public static string Source(string? word) => word switch
        {
            null => "missing",
            "package" => "the game's package",
            "screenscraper" => "ScreenScraper",
            "esde" => "an ES-DE media folder",
            "hand" => "placed by hand",
            "openemu" => "OpenEmu",
            "other_game" => "another game's picture",
            _ => word,
        };

        public static string Media(ScrapeRow row) => row.Media.Count == 0 ? "" : $"{row.Have} of {row.Media.Count}";

        public static string Missing(ScrapeRow row) => row.Media.Count > 0 && row.Missing.Count == 0 ? "none" : string.Join(", ", row.Missing);

        public static string Summary(IReadOnlyList<ManagerMarked<ScrapeRow>> marked) =>
            marked.Count == 0 ? "" : marked.Count == 1 ? $"1 game marked to scrape: {marked[0].Row.Title}" : $"{marked.Count} games marked to scrape";

        // The plan sheet's words: the scraper's own sentence, then what is asked and how.
        public static string Text(string said, IReadOnlyList<string> kinds, ScrapeOptions o, IReadOnlyList<string> notes)
        {
            var lines = new List<string> { Sentence(said) };
            lines.Add($"Kinds: {(kinds.Count == 0 ? "the scraper's default" : string.Join(", ", kinds))}.");
            lines.Add(o.Overwrite ? "What is there is fetched again and replaced." : "What is there is kept; only what is missing is fetched.");
            lines.AddRange(notes.Select(Sentence));
            lines.Add("Nothing is sent to ScreenScraper until Apply.");
            return string.Join("\n", lines);
        }

        // A run's progress answer as the frame's line: the fraction, the line, and the end's sentence once it is not running.
        // The tool's one-line state while a run goes: its progress, in place of the count until the run ends (EmuSen_Interface.md §19.7).
        public static ToolSummary Running(int done, int total) =>
            new($"Scraping {done:N0} of {total:N0}", Sentence: $"Scraping {done:N0} of {total:N0} games; the Scraper's window follows the run, and its Stop ends it.");

        // How far a run is, from the scraper's answer.
        public static (int Done, int Total) Counts(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement p = doc.RootElement.GetProperty("progress");
            return (Int(p, "done"), Int(p, "total"));
        }

        public static (double? Fraction, string Line, bool Running, string? Said) Progress(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement p = doc.RootElement.GetProperty("progress");
            bool running = ScrapeRow.Text(p, "state") == "running";
            int done = Int(p, "done"), total = Int(p, "total");
            double? fraction = total > 0 ? (double)done / total : null;
            string line = $"Scraping: {done} of {total} games";
            return (fraction, line, running, ScrapeRow.Text(doc.RootElement, "said") is { } said ? Sentence(said) : null);
        }

        // What the last run did to each game it reached: its path and the scraper's outcome word.
        public static IReadOnlyList<(string Path, string Outcome, long Serial)> Outcomes(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement p = doc.RootElement.GetProperty("progress");
            if (!p.TryGetProperty("results", out JsonElement results) || results.ValueKind != JsonValueKind.Array) return [];
            return results.EnumerateArray()
                .Select(r => (ScrapeRow.Text(r, "path") ?? "", Outcome(ScrapeRow.Text(r, "outcome"), r.TryGetProperty("skipped", out JsonElement s) && s.ValueKind == JsonValueKind.True),
                    r.TryGetProperty("serial", out JsonElement n) && n.ValueKind == JsonValueKind.Number && n.TryGetInt64(out long serial) ? serial : 0L))
                .Where(r => r.Item1.Length > 0).ToList();
        }

        // The run's phase word: running, done, stopped, idle.
        public static string? State(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            return ScrapeRow.Text(doc.RootElement.GetProperty("progress"), "state");
        }

        // The games the last poll reported failed or left to retry, with the scraper's detail, for the error log.
        public static IReadOnlyList<(string Path, string Outcome, string Detail)> Failures(string answer)
        {
            using JsonDocument doc = JsonDocument.Parse(answer);
            JsonElement p = doc.RootElement.GetProperty("progress");
            if (!p.TryGetProperty("results", out JsonElement results) || results.ValueKind != JsonValueKind.Array) return [];
            return results.EnumerateArray().Where(r => ScrapeRow.Text(r, "outcome") is "error" or "retry")
                .Select(r => (ScrapeRow.Text(r, "path") ?? "", Outcome(ScrapeRow.Text(r, "outcome")), ScrapeRow.Text(r, "detail") ?? "")).ToList();
        }

        public static string Outcome(string? word, bool skipped = false) => skipped ? "Skipped" : word switch
        {
            "found" => "Found",
            "unknown" => "Not known to ScreenScraper",
            "error" => "Failed",
            "retry" => "To retry",
            "stopped" => "Stopped",
            _ => "Nothing new",
        };

        // A sentence of the scraper's with a capital and a full stop.
        public static string Sentence(string words)
        {
            string w = words.Trim();
            if (w.Length == 0) return w;
            w = char.ToUpperInvariant(w[0]) + w[1..];
            return w.EndsWith('.') ? w : w + ".";
        }

        private static int Int(JsonElement e, string name) => e.TryGetProperty(name, out JsonElement v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt32(out int n) ? n : 0;
    }
}
