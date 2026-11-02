using System.Text.Json;
using EmuSen.LunaP.Controls;

namespace EmuSen.Tools.Cores
{
    // One job of a confirmed plan: its request, what it is doing in words, and the rows it works on.
    public sealed record CoresStep(string Request, string Doing, IReadOnlyList<CoreRow> Rows);

    // The marks a core is offered, the summary and plan they make, and the jobs that carry them out - see EmuSen_Cores.md §2.1 and §6.2.
    public static class CoresPlan
    {
        public static readonly ManagerMark Install = new("install", "install");
        public static readonly ManagerMark Update = new("update", "update");
        public static readonly ManagerMark Remove = new("remove", "remove");

        // A project core is offered nothing, being the installation's; a core the player put in /Cores is never removed by Cores (§5.6).
        public static IReadOnlyList<ManagerMark> MarksFor(CoreRow row) => !row.IsLibretro ? [] : row.State switch
        {
            "available" => [Install],
            "update" => [Update, Remove],
            "installed" => [Remove],
            _ => [],
        };

        private static string Sized(CoreRow row) => row.Size is long n and > 0 ? $"{row.Name} ({CoresWords.Size(n)})" : row.Name;

        private static IEnumerable<CoreRow> Of(IReadOnlyList<ManagerMarked<CoreRow>> marked, ManagerMark mark) => marked.Where(m => m.Mark.Key == mark.Key).Select(m => m.Row);

        // The apply bar's one line: counts per mark and the bytes to download, the names being the plan's.
        public static string Summary(IReadOnlyList<ManagerMarked<CoreRow>> marked)
        {
            var parts = new List<string>();
            foreach (ManagerMark mark in new[] { Install, Update, Remove })
                if (Of(marked, mark).Count() is int n and > 0) parts.Add($"{mark.Label} {n}");
            long bytes = Of(marked, Install).Concat(Of(marked, Update)).Sum(r => r.Size ?? 0);
            return $"{marked.Count} marked: {string.Join(", ", parts)}{(bytes > 0 ? $", about {CoresWords.Size(bytes)} to download" : "")}";
        }

        // The plan's question: one marked core by its action and name, several by their count (EmuSen_Interface.md §15.6).
        public static string Title(IReadOnlyList<ManagerMarked<CoreRow>> marked) => marked.Count == 1
            ? $"{char.ToUpperInvariant(marked[0].Mark.Label[0])}{marked[0].Mark.Label[1..]} {marked[0].Row.Name}?"
            : $"Apply {marked.Count} changes?";

        // What will happen, before anything runs (EmuSen_Cores.md §2.1).
        public static string Text(IReadOnlyList<ManagerMarked<CoreRow>> marked)
        {
            var lines = new List<string>();
            List<CoreRow> install = Of(marked, Install).ToList(), update = Of(marked, Update).ToList(), remove = Of(marked, Remove).ToList();
            if (install.Count > 0) lines.Add($"Download and install {install.Count} {(install.Count == 1 ? "core" : "cores")} from buildbot.libretro.com: {string.Join(", ", install.Select(Sized))}.");
            if (update.Count > 0) lines.Add($"Update {string.Join(", ", update.Select(r => r.Name))}; the build installed now is kept for Roll Back.");
            if (install.Count + update.Count > 0) lines.Add("Each core is checked before it is kept, and nothing downloaded runs until you approve it.");
            if (remove.Count > 0) lines.Add($"Remove {string.Join(", ", remove.Select(r => r.Name))}. Your saves and their settings are kept.");
            return string.Join("\n\n", lines);
        }

        // The jobs, in the order they run: installs, updates, then removals, one job each.
        public static IReadOnlyList<CoresStep> Steps(IReadOnlyList<ManagerMarked<CoreRow>> marked)
        {
            var steps = new List<CoresStep>();
            void Add(ManagerMark mark, string op, string doing)
            {
                List<CoreRow> rows = Of(marked, mark).ToList();
                if (rows.Count == 0) return;
                string request = JsonSerializer.Serialize(new { op, cores = rows.Select(r => r.Stem).ToArray() });
                steps.Add(new CoresStep(request, doing, rows));
            }
            Add(Install, "core-get", "Installing");
            Add(Update, "core-update", "Updating");
            Add(Remove, "core-rm", "Removing");
            return steps;
        }
    }
}
