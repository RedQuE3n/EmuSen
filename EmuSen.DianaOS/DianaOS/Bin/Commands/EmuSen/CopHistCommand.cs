using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // `regs` shows the instant; this shows the run-up to it - see §3.23a.
    public class CopHistCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "cophist";
        public bool IsReadOnly => true;
        public string Usage =>
            "  cophist [<reg>] [<count>]     Coprocessor register history (default: all regs, last 16)";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = DebugCommandHelpers.RequireTarget(target);

            if (target is not IHistoricalCoprocessorTarget historical)
            {
                return "This core does not keep coprocessor register history.";
            }

            var provider = historical.CoprocessorHistory;
            var frames = provider.GetHistory();
            if (frames.Count == 0 || frames[^1].Count == 0)
            {
                return "No coprocessor present on this cartridge.";
            }

            string? wanted = parts.Length > 1 && !int.TryParse(parts[1], out _) ? parts[1] : null;
            int count = 16;
            for (int i = 1; i < parts.Length; i++)
            {
                if (int.TryParse(parts[i], out int parsed)) { count = Math.Max(1, parsed); break; }
            }

            var shown = frames.Skip(Math.Max(0, frames.Count - count)).ToList();

            // Ties a row back to a real moment rather than just "16 ago".
            long newest = provider.RefreshCount;
            long firstIndex = newest - shown.Count + 1;

            var sb = new StringBuilder();
            sb.AppendLine($"Coprocessor history: {shown.Count} of {frames.Count} retained (capacity {provider.Capacity}),");
            sb.AppendLine($"unchanged for {provider.RefreshesSinceChange} refresh(es).");

            var names = wanted != null
                ? shown[^1].Where(r => string.Equals(r.Name, wanted, StringComparison.OrdinalIgnoreCase)).Select(r => r.Name).ToList()
                : shown[^1].Select(r => r.Name).ToList();

            if (names.Count == 0) return $"No coprocessor register named '{wanted}'.";

            sb.AppendLine("  refresh  " + string.Join("  ", names.Select(n => n.PadLeft(6))));
            for (int i = 0; i < shown.Count; i++)
            {
                var byName = shown[i].ToDictionary(r => r.Name, r => r);
                var cells = names.Select(n => byName.TryGetValue(n, out var r)
                    ? r.Value.ToString("X" + Math.Max(1, r.BitWidth / 4)).PadLeft(6)
                    : "".PadLeft(6));
                sb.AppendLine($"  {firstIndex + i,7}  " + string.Join("  ", cells));
            }

            return sb.ToString().TrimEnd();
        }
    }

    // Separate from IDebugTarget so a core without history carries no ring buffer - see §3.1a.
    public interface IHistoricalCoprocessorTarget
    {
        HistoryProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorHistory { get; }
    }
}
