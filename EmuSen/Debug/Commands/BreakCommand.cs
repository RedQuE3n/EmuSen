using System.Linq;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    // Manages execution breakpoints (see BreakpointRegistry.cs) - the
    // control-flow counterpart to `watch`. Deliberately just add/list/
    // remove here: actually halting and resuming execution requires
    // re-entering a core's own RunFrame() loop, which only the frontend's
    // own main loop can do (see EmuSen.RaylibFrontend/Program.cs's
    // IsHaltedAtBreakpoint check and its `step`/`continue` handling in the
    // F4 prompt) - same reason `exit`/`step` aren't DebugCommandProcessor
    // commands either. This command only edits the registry any frontend
    // consults; it doesn't know or care whether emulation is currently
    // halted.
    public class BreakCommand : IDebugCommand
    {
        public string Name => "break";
        public string Usage => string.Join('\n', new[]
        {
            "  break add <addr>              halt execution just before <addr> runs (24-bit CPU address)",
            "  break list                     list active breakpoints with their IDs and hit counts",
            "  break remove <id>              remove a breakpoint entirely",
            "  (see also: the F4 prompt's own 'step'/'s' and 'continue'/'c' - not",
            "  DebugCommandProcessor commands, since they need to resume the core's",
            "  own frame loop, not just edit the breakpoint list)",
        });

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 2) return "Usage: break add|list|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var breakpoints = target.Breakpoints;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: break add <addr>";
                    int addr = ParseHex(parts[2]);
                    int id = breakpoints.AddBreakpoint(addr);
                    return $"Breakpoint #{id} added at ${addr:X6}.";
                }
                case "list":
                {
                    var list = breakpoints.GetBreakpoints();
                    if (list.Count == 0) return "No active breakpoints.";
                    return string.Join('\n', list.Select(b =>
                        $"  #{b.Id}: ${b.Address:X6} ({(b.Enabled ? "enabled" : "disabled")}, hit {b.HitCount}x)"));
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: break remove <id>";
                    bool removed = breakpoints.RemoveBreakpoint(ParseHex(parts[2]));
                    return removed ? $"Breakpoint #{parts[2]} removed." : $"No breakpoint #{parts[2]} found.";
                }
                default:
                    return $"Unknown 'break' subcommand '{sub}'. Try add/list/remove.";
            }
        }
    }
}
