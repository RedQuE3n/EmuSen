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

    // What a prune would do, worked out before anything is touched. A plan
    // that CanApply is false for is a refusal with a Reason, never a
    // silently empty deletion - see `man cheat`.
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

    // Drops the cheat-database folders no implemented core can use. The
    // libretro database ships ~44 systems and a build with one core needs
    // one of them, so this is the difference between a 250MB tree and a
    // 16MB one - and every CheatDatabase scan walks all of it.
    //
    // Core-agnostic: it is handed the set of system names to KEEP and has no
    // idea what a core is. Which names those are comes from the core
    // registry - see CoreDescriptor.SupportedCheatSystems and
    // EmuSen_Settings_Reference.md §4.16.
    public static class CheatDatabasePruner
    {
        // Works out what would go without touching anything. Refuses, rather
        // than planning a deletion, in the two cases where an empty or
        // unmatched keep-set would wipe the whole database:
        //
        // - Nothing to keep. A registry whose cores claim no cheat systems
        //   is a build that has not filled CheatSystems in yet, not a build
        //   that wants every cheat gone.
        // - Nothing kept matches what is on disk. That is a wrong folder or
        //   a wrong mapping, and the honest response is to say so - the one
        //   case where "delete all 44 systems" is exactly what it looks like.
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

        // Deletes what <plan> listed. Returns how many folders actually went
        // and anything that would not delete, which is reported rather than
        // thrown - one locked folder must not abort the other forty.
        public static (int Removed, IReadOnlyList<string> Failed) Apply(CheatDatabase database, CheatPrunePlan plan)
        {
            if (!plan.CanApply) return (0, Array.Empty<string>());

            var failed = new List<string>();
            int removed = 0;

            foreach (PrunedSystem system in plan.Removing)
            {
                // Never outside the tree it was planned against, whatever a
                // folder name or a symlink claims.
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

        // Resolved on both sides, so neither a `..` segment nor a symlinked
        // system folder can reach out of the cheat directory.
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
