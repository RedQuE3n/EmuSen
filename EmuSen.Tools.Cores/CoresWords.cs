using System.Collections.Generic;
using System.Linq;
using EmuSen.Galaxia.Native;
using System.Globalization;

namespace EmuSen.Tools.Cores
{
    // Cores' words for sizes, states and the core tool's refusals - see EmuSen_Cores.md §2.1 and §4.2.
    public static class CoresWords
    {
        // The tool's one-line state as Home's place shows it: a core waiting for approval first, then updates, else the counts; null before anything is listed (EmuSen_Interface.md §15.8, §19).
        public static ToolSummary? StateOf(IReadOnlyCollection<CoreRow> rows)
        {
            if (rows.Count == 0) return null;
            int approve = rows.Count(r => r.AwaitsApproval), updates = rows.Count(r => r.State == "update"), installed = rows.Count(r => r.Installed);
            const string Untrusted = " Nothing downloaded runs until you approve it.";
            return approve > 0 ? new($"{approve:N0} to approve", NeedsAttention: true, Sentence: (approve == 1 ? Awaiting(rows.First(r => r.AwaitsApproval)) : $"{approve:N0} cores wait for your approval.") + Untrusted)
                : updates > 0 ? new("↑ " + ToolSummary.Count(updates, "update", "updates"), Sentence: updates == 1 ? "A newer build of 1 core is offered." : $"Newer builds of {updates:N0} cores are offered.")
                : new($"{ToolSummary.Count(rows.Count, "core", "cores")} · {installed:N0} installed", Sentence: $"{ToolSummary.Count(rows.Count, "core", "cores")}, {installed:N0} of them installed; none waits for your approval.");
        }

        // The one core waiting, said with its console and where it came from: "PCEE2, a PlayStation 2 core from libretro's buildbot, waits for your approval."
        private static string Awaiting(CoreRow one)
        {
            string from = one.IsLibretro ? " from libretro's buildbot" : "";
            string what = one.ConsoleNames.Count > 0 ? $", a {one.ConsoleNames[0]} core{from}," : from.Length > 0 ? $",{from}," : "";
            return $"{one.Name}{what} waits for your approval.";
        }

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
