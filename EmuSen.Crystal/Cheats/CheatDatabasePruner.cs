using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Crystal.Cheats
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

    // Handed the names to keep, with no idea what a core is; emusen-cheats' prune module - see EmuSen_Settings_Reference.md §4.16, EmuSen_Backend.md §15.3.
    public static class CheatDatabasePruner
    {
        // Refuses the two cases where an empty or unmatched keep-set would wipe everything - see §4.16.
        public static CheatPrunePlan Plan(CheatDatabase database, IReadOnlyCollection<string> keep) => database.NativePlan(keep);

        // Failures are reported, not thrown: one locked folder must not abort the other forty; never outside the tree it was planned against.
        public static (int Removed, IReadOnlyList<string> Failed) Apply(CheatDatabase database, CheatPrunePlan plan) =>
            plan.CanApply ? CheatsNative.Apply(database.Directory, plan) : (0, Array.Empty<string>());
    }
}
