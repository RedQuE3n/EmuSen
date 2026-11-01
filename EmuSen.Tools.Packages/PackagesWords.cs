using System.Globalization;
using System.Text.Json;
using static EmuSen.Tools.Packages.ToolEvents;

namespace EmuSen.Tools.Packages
{
    // The listing's states and each job's outcome in Packages' words, the tool's own kept where none fits - see EmuSen_Packages.md §7.
    public static class PackagesWords
    {
        // A row's state as the State column says it.
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
