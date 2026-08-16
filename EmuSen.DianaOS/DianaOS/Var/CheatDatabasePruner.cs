using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // One system folder the pruner would remove.
    public readonly struct PrunedSystem
    {
        public string System { get; init; }
        public string Path { get; init; }
        public int Files { get; init; }
        public long Bytes { get; init; }
    }

    // A plan that cannot apply is a refusal with a reason, never an empty deletion - see `man cheat`.
    public sealed class CheatPrunePlan
    {
        public required IReadOnlyList<PrunedSystem> Removing { get; init; }
        public required IReadOnlyList<string> Keeping { get; init; }
        public required bool CanApply { get; init; }
        public required string? Reason { get; init; }

        public int Files => Removing.Sum(s => s.Files);
        public long Bytes => Removing.Sum(s => s.Bytes);

        public static string Human(long bytes) => bytes switch
        {
            >= 1L << 30 => $"{bytes / (double)(1L << 30):F1}G",
            >= 1L << 20 => $"{bytes / (double)(1L << 20):F0}M",
            >= 1L << 10 => $"{bytes / (double)(1L << 10):F0}K",
            _ => $"{bytes}B",
        };
    }

    // Handed the names to keep, with no idea what a core is - see EmuSen_Settings_Reference.md §4.16.
    public static class CheatDatabasePruner
    {
        // Refuses the two cases where an empty or unmatched keep-set would wipe everything - see §4.16.
        public static CheatPrunePlan Plan(CheatDatabase database, IReadOnlyCollection<string> keep)
        {
            var keepSet = new HashSet<string>(keep, StringComparer.OrdinalIgnoreCase);
            IReadOnlyList<(string System, int Count)> present = database.Systems();

            List<string> keeping = present.Where(s => keepSet.Contains(s.System)).Select(s => s.System).ToList();
            List<(string System, int Count)> removing = present.Where(s => !keepSet.Contains(s.System)).ToList();

            string? reason = null;
            if (keepSet.Count == 0) reason = "no core in this build claims any cheat system, so there is nothing to prune against.";
            else if (present.Count == 0) reason = $"no cheat files in {database.Directory}.";
            else if (keeping.Count == 0) reason = "none of the systems on disk match any core in this build - refusing to delete the whole database.";
            else if (removing.Count == 0) reason = "every system on disk is already one a core can use.";

            return new CheatPrunePlan
            {
                Removing = reason is null ? removing.Select(s => Measure(database.Directory, s.System, s.Count)).ToList() : Array.Empty<PrunedSystem>(),
                Keeping = keeping,
                CanApply = reason is null,
                Reason = reason,
            };
        }

        private static PrunedSystem Measure(string root, string system, int files)
        {
            string path = Path.Combine(root, system);
            long bytes = 0;
            try
            {
                foreach (string file in Directory.EnumerateFiles(path, "*", SearchOption.AllDirectories))
                {
                    try { bytes += new FileInfo(file).Length; } catch { }
                }
            }
            catch { }

            return new PrunedSystem { System = system, Path = path, Files = files, Bytes = bytes };
        }

        // Failures are reported, not thrown: one locked folder must not abort the other forty.
        public static (int Removed, IReadOnlyList<string> Failed) Apply(CheatDatabase database, CheatPrunePlan plan)
        {
            if (!plan.CanApply) return (0, Array.Empty<string>());

            var failed = new List<string>();
            int removed = 0;

            foreach (PrunedSystem system in plan.Removing)
            {
                // Never outside the tree it was planned against, whatever a name or symlink claims.
                if (!IsInside(database.Directory, system.Path))
                {
                    failed.Add($"{system.System} (outside {database.Directory})");
                    continue;
                }

                try
                {
                    Directory.Delete(system.Path, recursive: true);
                    removed++;
                }
                catch (Exception ex)
                {
                    failed.Add($"{system.System} ({ex.Message})");
                }
            }

            return (removed, failed);
        }

        // Resolved on both sides, so neither .. nor a symlink reaches out of the tree.
        private static bool IsInside(string root, string candidate)
        {
            try
            {
                string fullRoot = Path.TrimEndingDirectorySeparator(Path.GetFullPath(root));
                string fullCandidate = Path.TrimEndingDirectorySeparator(Path.GetFullPath(candidate));

                if (new DirectoryInfo(fullCandidate).LinkTarget is not null) return false;

                return fullCandidate.Length > fullRoot.Length
                    && fullCandidate.StartsWith(fullRoot, StringComparison.Ordinal)
                    && fullCandidate[fullRoot.Length] == Path.DirectorySeparatorChar;
            }
            catch
            {
                return false;
            }
        }
    }
}
