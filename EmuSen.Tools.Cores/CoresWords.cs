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

        public static string Consoles(CoreRow row) => row.Consoles.Count > 0 ? string.Join(", ", row.Consoles) : "—";

        // The tool's refusal for the whole request, said as the window says it.
        public static string Refusal(string tool) => tool switch
        {
            "no buildbot listing is known for this platform" => "libretro publishes no cores for this kind of machine that EmuSen knows of, so only EmuSen's own cores are listed.",
            _ when tool.StartsWith("another Cores job", StringComparison.Ordinal) => "Another Cores job is changing the cores folder, in this program or another. Try again when it ends.",
            _ => tool.Length > 0 ? char.ToUpperInvariant(tool[0]) + tool[1..] + (tool.EndsWith('.') ? "" : ".") : tool,
        };
    }
}
