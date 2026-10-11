using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // The target's free-text dump; reports not read-only, as the builtin it was did - see `man summary` and EmuSen_DianaSh.md §13.5.
    public class SummaryCommand : IDianaShCommand
    {
        public string Name => "summary";
        public bool IsReadOnly => false;
        public string Usage => "  summary                       free-text state dump (whatever isn't structured above yet)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => target?.GetSummaryText() ?? "No target attached.";
    }
}
