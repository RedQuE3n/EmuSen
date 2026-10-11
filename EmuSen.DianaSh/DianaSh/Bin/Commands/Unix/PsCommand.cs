using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Exists to print ids `kill` can consume unchanged - see `man ps`.
    public class PsCommand : IDianaShCommand
    {
        private readonly DianaShSessionManager? _sessions;

        public PsCommand(DianaShSessionManager? sessions)
        {
            _sessions = sessions;
        }

        public string Name => "ps";
        public bool IsReadOnly => true;
        public string Usage => "  ps | jobs                     list active breakpoints/watches/sessions as one table - see `kill`";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
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
