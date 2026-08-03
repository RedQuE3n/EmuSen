using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;
using EmuSen.Galaxia.Text;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
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
    public class BreakCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "bp";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  bp add <addr> [if <expr>]     halt just before <addr> runs (24-bit CPU address)",
            "  bp write <space> <addr> [<v>] [if <expr>]",
            "                                halt just after anything writes <addr> in <space>",
            "  bp list                       list active breakpoints with their IDs and hit counts",
            "  bp on|off <id>                enable/disable one breakpoint without removing it",
            "  bp remove <id>                remove a breakpoint entirely",
            "  bp <cpu> add|list|remove ...  same, on another processor - `cpus` lists them, and each",
            "                                one's addresses are its own address space",
            "  (see also: `cpus`, `step`, `runto`, and the F4 prompt's own 'continue'/'c')",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: bp [<cpu>] add|list|remove ...";

            // An optional scope word in front of the subcommand - see `man cpus`.
            var (cpu, at) = ResolveCpu(target, parts, 1);
            if (cpu == null && DebugCpus.IsScopeWord(target.DebugCpus, parts[1]))
            {
                return "This cartridge has no coprocessor CPU to break on.";
            }
            if (parts.Length <= at) return "Usage: bp <cpu> add|list|remove ...";
            if (cpu is { CanHalt: false }) return $"{cpu.Name} cannot be halted by this core, so it takes no breakpoints.";

            // No CPU list published: the legacy single-registry path.
            var breakpoints = cpu?.Breakpoints ?? target.Breakpoints;

            string sub = parts[at].ToLowerInvariant();
            string addrArg = parts.Length > at + 1 ? parts[at + 1] : string.Empty;
            bool coprocessor = cpu != null && cpu.Name != DebugCpus.MainName;
            string scope = coprocessor ? $"{cpu!.Name.ToUpperInvariant()} breakpoint" : "Breakpoint";

            switch (sub)
            {
                case "add":
                {
                    if (addrArg.Length == 0) return "Usage: bp add <addr> [if <expr>]";
                    int addr = ParseHex(addrArg);
                    string? condition = TakeCondition(parts, at + 2);
                    int id = breakpoints.AddBreakpoint(addr, condition);
                    string ifText = condition == null ? string.Empty : $" if {condition}";
                    return $"{scope} #{id} added at ${addr:X6}{ifText}.";
                }
                case "write":
                {
                    // Halts at the instruction AFTER the store - see BreakpointRegistry.NoteWrite.
                    if (parts.Length < at + 3) return "Usage: bp write <space> <addr> [<value>] [if <expr>]";
                    var space = FindSpace(target, parts[at + 1]);
                    int addr = ParseHex(parts[at + 2]);
                    bool hasValue = parts.Length > at + 3 && !parts[at + 3].Equals("if", System.StringComparison.OrdinalIgnoreCase);
                    int value = hasValue ? ParseHex(parts[at + 3]) : -1;
                    string? condition = TakeCondition(parts, hasValue ? at + 4 : at + 3);
                    int id = breakpoints.AddDataBreakpoint(space.Name, addr, value, condition);
                    string valueText = value < 0 ? string.Empty : $" = 0x{value:X2}";
                    string ifText = condition == null ? string.Empty : $" if {condition}";
                    return $"{scope} #{id} added on writes to {space.Name} 0x{addr:X}{valueText}{ifText}.";
                }
                case "list":
                {
                    var list = breakpoints.GetBreakpoints();
                    var dataList = breakpoints.GetDataBreakpoints();
                    if (list.Count == 0 && dataList.Count == 0) return coprocessor ? $"No active {cpu!.Name} breakpoints." : "No active breakpoints.";
                    var labels = target.Labels;
                    var lines = list.Select(b =>
                        $"  #{b.Id}: {(labels?.Describe(b.Address) ?? $"${b.Address:X6}")}{Condition(b.Condition)} ({(b.Enabled ? "enabled" : "disabled")}, hit {b.HitCount}x)")
                        .Concat(dataList.Select(b =>
                        $"  #{b.Id}: write {b.Space} 0x{b.Address:X}{(b.Value < 0 ? string.Empty : $" = 0x{b.Value:X2}")}{Condition(b.Condition)} ({(b.Enabled ? "enabled" : "disabled")}, hit {b.HitCount}x)"));
                    return string.Join('\n', lines);
                }
                case "on":
                case "off":
                {
                    if (addrArg.Length == 0) return $"Usage: bp {sub} <id>";
                    int id = ParseHex(addrArg);
                    bool changed = breakpoints.SetEnabled(id, sub == "on");
                    return changed
                        ? $"{scope} #{id} {(sub == "on" ? "enabled" : "disabled")}."
                        : DianaOSResult.Fail($"No {(coprocessor ? cpu!.Name + " " : string.Empty)}breakpoint #{id} found.");
                }
                case "remove":
                {
                    if (addrArg.Length == 0) return "Usage: bp remove <id>";
                    bool removed = breakpoints.RemoveBreakpoint(ParseHex(addrArg));
                    return removed ? $"{scope} #{addrArg} removed." : $"No {(coprocessor ? cpu!.Name + " " : string.Empty)}breakpoint #{addrArg} found.";
                }
                default:
                {
                    // Named once so the suggestion and the "Try" list cannot drift.
                    string[] subcommands = { "add", "write", "list", "on", "off", "remove" };
                    return $"Unknown 'bp' subcommand '{sub}'.{Suggestion.Hint(sub, subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }

        // Everything after an `if` word, rejoined - see `man bp`.
        private static string? TakeCondition(string[] parts, int from)
        {
            if (parts.Length <= from) return null;
            if (!parts[from].Equals("if", System.StringComparison.OrdinalIgnoreCase)) return null;
            if (parts.Length <= from + 1) return null;
            return string.Join(' ', parts.Skip(from + 1));
        }

        private static string Condition(string? condition) => condition == null ? string.Empty : $" if {condition}";
    }
}
