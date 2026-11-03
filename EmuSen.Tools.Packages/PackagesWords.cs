using EmuSen.Galaxia.Native;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using static EmuSen.Galaxia.Native.ToolEvents;

namespace EmuSen.Tools.Packages
{
    // The listing's states and each job's outcome in Packages' words, the tool's own kept where none fits - see EmuSen_Packages.md §7.
    public static class PackagesWords
    {
        // The tool's one-line state: the packages whose state needs a look, else how many are packaged; null before anything is listed (EmuSen_Interface.md §19).
        public static ToolSummary? StateOf(IReadOnlyCollection<PackageRow> rows)
        {
            if (rows.Count == 0) return null;
            int changed = rows.Count(r => r.Problem), packaged = rows.Count(r => r.Packaged);
            return changed > 0 ? new($"{changed:N0} changed", Sentence: changed == 1 ? "1 package no longer matches its record: it moved, went missing or changed." : $"{changed:N0} packages no longer match their records: each moved, went missing or changed.")
                : packaged > 0 ? new($"{packaged:N0} packaged", Sentence: packaged == 1 ? "1 game is packaged." : $"{packaged:N0} games are packaged.")
                : new("None packaged yet", Sentence: "No game is packaged yet.");
        }

        // A row's state as the State column says it.
        // The state with its glyph, a word beside a mark: ✓ for a package in order, ⚠ for a problem (EmuSen_Interface.md §5.7).
        public static string Status(PackageRow row) => row.Problem ? "⚠ " + State(row) : row.Packaged ? "✓ " + State(row) : State(row);

        public static string State(PackageRow row) => row.State switch
        {
            "unpackaged" => "Not packaged",
            "ok" => "OK",
            "changed" => "Changed",
            "moved" => "Moved",
            "missing" => "Missing",
            "refused" => "Unreadable",
            string s => s,
        };

        // A mode as the Mode column says it.
        public static string Mode(PackageRow row) => row.Mode switch
        {
            "linked" => "linked",
            "self-contained" => "self-contained",
            "" => "—",
            string m => m,
        };

        private static string Why(JsonElement e) => Text(e, "words") is { Length: > 0 } w ? $": {ToolWords.Reason(w)}" : "";

        // What one package or disc event of a job did, null for an event that is not an outcome.
        public static string? Outcome(JsonElement e)
        {
            string state = Text(e, "state") ?? "";
            switch (Text(e, "event"))
            {
                case "package":
                    return state switch
                    {
                        "made" when Text(e, "mode") == "self-contained" => "Made self-contained, the game copied in",
                        "made" => "Made, linked to the game where it is",
                        "contained" => "Made self-contained, the game copied in",
                        "zipped" => e.TryGetProperty("zip", out JsonElement z) && z.ValueKind == JsonValueKind.Object ? $"Zipped, {ToolWords.Size(Number(z, "bytes"))}" : "Zipped",
                        "already" => "Already a package",
                        "verified" => "Checked: every file is as recorded",
                        "moved" => Text(e, "to") is { } to ? $"Moved: found again at {to}, and its manifest now says so" : "Moved",
                        "stopped" => "Stopped before it finished",
                        _ => $"{CultureInfo.InvariantCulture.TextInfo.ToTitleCase(state)}{Why(e)}",
                    };
                case "disc" when state != "working":
                    long chd = Number(e, "chd"), original = Number(e, "original");
                    return state switch
                    {
                        "converted" when chd > 0 && original > 0 => $"Converted and verified: {ToolWords.Size(chd)}, {(chd * 100.0 / original).ToString("F0", CultureInfo.InvariantCulture)}% of the original",
                        "converted" => "Converted and verified",
                        "already" => "Already converted and verified",
                        "skipped" => $"Left as it is{Why(e)}",
                        "failed" => $"Failed, and no CHD was kept{Why(e)}",
                        "stopped" => "Stopped before it finished",
                        _ => $"{state}{Why(e)}",
                    };
                default:
                    return null;
            }
        }
    }
}
