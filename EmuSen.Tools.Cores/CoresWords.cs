using System.Globalization;

namespace EmuSen.Tools.Cores
{
    // Cores' words for sizes, states and the core tool's refusals - see EmuSen_Cores.md §2.1 and §4.2.
    public static class CoresWords
    {
        public static string Size(long bytes) => bytes switch
        {
            < 1024 => $"{bytes} B",
            < 1024 * 1024 => (bytes / 1024.0).ToString("0", CultureInfo.InvariantCulture) + " KB",
            < 1024L * 1024 * 1024 => (bytes / 1048576.0).ToString("0.#", CultureInfo.InvariantCulture) + " MB",
            _ => (bytes / 1073741824.0).ToString("0.#", CultureInfo.InvariantCulture) + " GB",
        };

        public static string State(CoreRow row) => row.AwaitsApproval ? "waiting for approval" : row.Trust == "needs_refresh" ? "needs refresh: Verify" : row.State switch
        {
            "update" => "update ↑",
            "unsupported" => "not supported",
            "added" or "approved" => "added by you",
            "manifest" => "built in",
            "not_read" => "not readable",
            "needs_approval" => "waiting for approval",
            var s => s,
        };

        // A date the tool wrote as 2026-10-31, said as 31 Oct 2026; any other words as they are.
        public static string Day(string date) => DateOnly.TryParseExact(date, "yyyy-MM-dd", CultureInfo.InvariantCulture, DateTimeStyles.None, out DateOnly day) ? day.ToString("d MMM yyyy", CultureInfo.InvariantCulture) : date;

        // A date the tool wrote as 2026-10-31, said in full as 31 October 2026; null for anything else.
        public static string? LongDay(string date) => DateOnly.TryParseExact(date, "yyyy-MM-dd", CultureInfo.InvariantCulture, DateTimeStyles.None, out DateOnly day) ? day.ToString("d MMMM yyyy", CultureInfo.InvariantCulture) : null;

        public static string Consoles(CoreRow row) => row.ConsoleNames.Count > 0 ? string.Join(", ", row.ConsoleNames) : "—";

        // The status in the list, a glyph beside a word in Title Case, one owner with its colour (EmuSen_Interface.md §15.3).
        public static string Status(CoreRow row) => row.IsDefault ? "● Default"
            : row.AwaitsApproval ? "⚠ To approve"
            : row.Trust == "needs_refresh" ? "⚠ Needs refresh"
            : row.State switch
            {
                "update" => "↑ Update",
                "installed" => "✓ Installed",
                "available" => "Available",
                "unsupported" or "not_read" => "Not supported",
                "built in" or "manifest" => "Built in",
                "added" or "approved" => "✓ Added by you",
                var s => s.Length > 0 ? char.ToUpperInvariant(s[0]) + s[1..] : s,
            };

        // The status's colour, a resource key the frame paints the cell with.
        public static string? Tone(CoreRow row) => row.IsDefault ? null
            : row.AwaitsApproval || row.Trust == "needs_refresh" || row.State == "update" ? "LunaAttention"
            : row.State is "installed" or "added" or "approved" ? "LunaSuccess"
            : "LunaMuted";

        // The tool's refusal for the whole request, said as the window says it.
        public static string Refusal(string tool) => tool switch
        {
            "no buildbot listing is known for this platform" => "libretro publishes no cores for this kind of machine that EmuSen knows of, so only EmuSen's own cores are listed.",
            _ when tool.StartsWith("another Cores job", StringComparison.Ordinal) => "Another Cores job is changing the cores folder, in this program or another. Try again when it ends.",
            _ => tool.Length > 0 ? char.ToUpperInvariant(tool[0]) + tool[1..] + (tool.EndsWith('.') ? "" : ".") : tool,
        };
    }
}
