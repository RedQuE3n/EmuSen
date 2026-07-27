using System.Linq;
using static EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    // Manages execution breakpoints (see BreakpointRegistry.cs) - the
    // control-flow counterpart to `watch`. Deliberately just add/list/
    // remove here: actually halting and resuming execution requires
    // re-entering a core's own RunFrame() loop, which only the frontend's
    // own main loop can do (see EmuSen.Hotaru/Program.cs's
    // IsHaltedAtBreakpoint check and its `step`/`continue` handling in the
    // F4 prompt) - same reason `exit`/`step` aren't DianaOSInterpreter
    // commands either. This command only edits the registry any frontend
    // consults; it doesn't know or care whether emulation is currently
    // halted.
    //
    // Named `bp`, not `break` - the shell's own `break`/`continue` loop-
    // control keywords (Parser.cs) are hardcoded, zero-argument statements
    // at the grammar level, the same way real bash treats them, so a
    // command literally named `break` is unreachable: `break add 8000`
    // parses as the bare loop-control statement followed by a syntax
    // error on the leftover `add 8000`. Caught by the shell harness after
    // the control-flow keywords were added - see EmuSen_Debugging_Tools_Reference_v5.md.
    public class BreakCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "bp";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  bp add <addr>                 halt execution just before <addr> runs (24-bit CPU address)",
            "  bp list                       list active breakpoints with their IDs and hit counts",
            "  bp remove <id>                remove a breakpoint entirely",
            "  (see also: the F4 prompt's own 'step'/'s' and 'continue'/'c' - not",
            "  DianaOSInterpreter commands, since they need to resume the core's",
            "  own frame loop, not just edit the breakpoint list)",
        });

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: bp add|list|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var breakpoints = target.Breakpoints;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: bp add <addr>";
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
                    if (parts.Length < 3) return "Usage: bp remove <id>";
                    bool removed = breakpoints.RemoveBreakpoint(ParseHex(parts[2]));
                    return removed ? $"Breakpoint #{parts[2]} removed." : $"No breakpoint #{parts[2]} found.";
                }
                default:
                    return $"Unknown 'bp' subcommand '{sub}'. Try add/list/remove.";
            }
        }
    }
}
