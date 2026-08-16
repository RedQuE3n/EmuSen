using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Exists to print ids `kill` can consume unchanged - see `man ps`.
    public class PsCommand : IDianaOSCommand
    {
        private readonly DianaOSSessionManager? _sessions;

        public PsCommand(DianaOSSessionManager? sessions)
        {
            _sessions = sessions;
        }

        public string Name => "ps";
        public bool IsReadOnly => true;
        public string Usage => "  ps | jobs                     list active breakpoints/watches/sessions as one table - see `kill`";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            var lines = new List<string>();

            if (target != null)
            {
                foreach (var bp in target.Breakpoints.GetBreakpoints())
                {
                    string id = $"bp{bp.Id}";
                    lines.Add($"  {id,-10} breakpoint  ${bp.Address:X6} ({(bp.Enabled ? "enabled" : "disabled")}, hit {bp.HitCount}x)");
                }

                foreach (var w in target.Watches.GetWatches())
                {
                    string id = $"watch{w.Id}";
                    lines.Add($"  {id,-10} watch       {w.SpaceName} 0x{w.StartAddress:X}-0x{w.StartAddress + w.Length - 1:X} ({w.Kind.ToString().ToLowerInvariant()})");
                }
            }

            if (_sessions != null)
            {
                foreach (var s in _sessions.Sessions)
                {
                    string marker = s == _sessions.Current ? "(current)" : "";
                    lines.Add($"  {s.Name,-10} session     {marker}");
                }
            }

            if (lines.Count == 0) return "No active breakpoints, watches, or sessions.";
            return string.Join('\n', lines);
        }
    }
}
