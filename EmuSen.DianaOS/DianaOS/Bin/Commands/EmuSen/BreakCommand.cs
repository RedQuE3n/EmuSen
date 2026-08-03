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
    // Edits the BreakpointRegistry a frontend's own run loop consults - see `man bp`.
    public class BreakCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "bp";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  bp add <addr>[-<end>] [if <expr>] [log <expr>]",
            "                                halt just before <addr> runs (24-bit CPU address, or a range);",
            "                                with 'log' it records <expr> and carries on instead of halting",
            "  bp write <space> <addr>[-<end>] [<v>] [changed] [if <expr>]",
            "                                halt just after anything writes that address",
            "  bp read <space> <addr>[-<end>] [<v>] [if <expr>]",
            "                                halt just after anything reads it",
            "  bp uninit <space>             halt the first time never-written memory is read back",
            "  bp when <condition> [log|off] halt (or just record) when the hardware does something",
            "                                it should not - `bp when` alone lists what this core detects",
            "  bp depth <n>                  halt if the call stack ever gets deeper than <n>",
            "  bp forbid <addr>-<end>        never halt while the PC is in this range",
            "  bp log [<count>] | bp log clear",
            "                                show (or forget) what the logpoints have recorded",
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
                    if (addrArg.Length == 0) return "Usage: bp add <addr>[-<end>] [if <expr>]";
                    var (start, end) = ParseRange(addrArg);
                    var (condition, log) = TakeClauses(parts, at + 2);
                    int id = breakpoints.AddBreakpoint(start, end, condition, log);
                    return $"{scope} #{id} {(log == null ? "added at" : "logging at")} {Range(start, end)}{Clauses(condition, log)}.";
                }
                case "read":
                case "write":
                {
                    // Halts at the instruction AFTER the access - see `man bp`.
                    if (parts.Length < at + 3) return $"Usage: bp {sub} <space> <addr>[-<end>] [<value>] [changed] [if <expr>]";
                    var space = FindSpace(target, parts[at + 1]);
                    var (start, end) = ParseRange(parts[at + 2]);

                    int value = -1;
                    bool changedOnly = false;
                    int i = at + 3;
                    for (; i < parts.Length && !IsClauseWord(parts[i]); i++)
                    {
                        if (parts[i].Equals("changed", System.StringComparison.OrdinalIgnoreCase)) { changedOnly = true; continue; }
                        value = ParseHex(parts[i]);
                    }
                    var (condition, log) = TakeClauses(parts, i);
                    if (changedOnly && sub == "read") return "'changed' only applies to writes - a read never changes the value.";

                    int id = breakpoints.AddDataBreakpoint(space.Name, start, end, value, onRead: sub == "read", changedOnly, condition, log);
                    string valueText = value < 0 ? string.Empty : $" = 0x{value:X2}";
                    string changedText = changedOnly ? " (only when it changes)" : string.Empty;
                    string verb = log == null ? "added on" : "logging on";
                    return $"{scope} #{id} {verb} {sub}s of {space.Name} {DataRange(start, end)}{valueText}{changedText}{Clauses(condition, log)}.";
                }
                case "uninit":
                {
                    if (addrArg.Length == 0) return "Usage: bp uninit <space>";
                    var space = FindSpace(target, addrArg);
                    if (!breakpoints.ArmUninitializedReadBreak(space.Name, space.Size))
                        return DianaOSResult.Fail($"{space.Name} reports no size, so its writes cannot be tracked.");
                    return $"Halting on the first read of any never-written byte of {space.Name}.";
                }
                case "when":
                {
                    var available = target.BreakConditions;
                    if (available.Count == 0)
                        return DianaOSResult.Fail("This core reports no hardware conditions to break on.");

                    var armed = breakpoints.GetConditions();
                    if (addrArg.Length == 0)
                    {
                        var rows = available.Select(c =>
                        {
                            var match = armed.FirstOrDefault(a => a.Name.Equals(c.Name, System.StringComparison.OrdinalIgnoreCase));
                            string state = match.Name == null
                                ? "off"
                                : $"{(match.LogOnly ? "logging" : "halting")}, {match.HitCount}x";
                            return $"  {c.Name,-12} {c.Description}  [{state}]";
                        });
                        return "Conditions this core detects:\n" + string.Join('\n', rows);
                    }

                    var chosen = available.FirstOrDefault(c => c.Name.Equals(addrArg, System.StringComparison.OrdinalIgnoreCase));
                    if (chosen.Name == null)
                    {
                        string[] names = available.Select(c => c.Name).ToArray();
                        return DianaOSResult.Fail($"'{addrArg}' is not a condition this core detects.{Suggestion.Hint(addrArg, names)} Try {string.Join('/', names)}.");
                    }

                    string mode = parts.Length > at + 2 ? parts[at + 2].ToLowerInvariant() : string.Empty;
                    if (mode == "off")
                    {
                        return breakpoints.DisarmCondition(chosen.Name)
                            ? $"No longer watching for {chosen.Name}."
                            : $"{chosen.Name} was not armed.";
                    }
                    if (mode.Length > 0 && mode != "log")
                        return $"Usage: bp when {chosen.Name} [log|off]";

                    bool logOnly = mode == "log";
                    breakpoints.ArmCondition(chosen.Name, logOnly);
                    return logOnly
                        ? $"Recording every {chosen.Name} ({chosen.Description}) without halting - read it back with `bp log`."
                        : $"Halting on {chosen.Name} ({chosen.Description}).";
                }
                case "depth":
                {
                    if (addrArg.Length == 0) return "Usage: bp depth <n>";
                    if (addrArg.Equals("off", System.StringComparison.OrdinalIgnoreCase))
                    {
                        breakpoints.DisarmDepthGuard();
                        return "Call-depth guard removed.";
                    }
                    int depth = ParseHex(addrArg);
                    if (!breakpoints.ArmDepthGuard(depth))
                        return DianaOSResult.Fail("This core does not report a call stack, so there is no depth to guard.");
                    return $"Halting if the call stack ever gets deeper than {depth}.";
                }
                case "forbid":
                {
                    if (addrArg.Length == 0) return "Usage: bp forbid <addr>-<end>";
                    var (start, end) = ParseRange(addrArg);
                    int id = breakpoints.AddForbidRange(start, end);
                    return $"Forbid range #{id} added: nothing halts while the PC is in {Range(start, end)}.";
                }
                case "log":
                {
                    if (addrArg.Equals("clear", System.StringComparison.OrdinalIgnoreCase))
                    {
                        breakpoints.ClearLog();
                        return "Logpoint log cleared.";
                    }
                    int count = addrArg.Length == 0 ? 32 : ParseHex(addrArg);
                    var entries = breakpoints.LogTail(count);
                    if (entries.Count == 0)
                    {
                        return breakpoints.GetBreakpoints().Any(b => b.LogExpression != null)
                            || breakpoints.GetDataBreakpoints().Any(b => b.LogExpression != null)
                            || breakpoints.GetConditions().Any(c => c.LogOnly)
                            ? "Nothing logged yet - the logpoints have not been hit."
                            : "No logpoints set - add one with `bp add <addr> log <expr>`.";
                    }
                    var labels = target.Labels;
                    // Id 0 is a `bp when` condition, which belongs to no numbered breakpoint.
                    var logged = entries.Select(e => $"  {(e.Id == 0 ? "when" : $"#{e.Id}")} {labels?.Describe(e.Address) ?? $"${e.Address:X6}"}  {e.Text}");
                    string more = breakpoints.LogEntriesRecorded > entries.Count
                        ? $"\n  ({breakpoints.LogEntriesRecorded} recorded in total)"
                        : string.Empty;
                    return string.Join('\n', logged) + more;
                }
                case "list":
                {
                    var list = breakpoints.GetBreakpoints();
                    var dataList = breakpoints.GetDataBreakpoints();
                    var forbidList = breakpoints.GetForbidRanges();
                    var conditionList = breakpoints.GetConditions();
                    if (list.Count == 0 && dataList.Count == 0 && forbidList.Count == 0 && conditionList.Count == 0)
                    {
                        string none = coprocessor ? $"No active {cpu!.Name} breakpoints." : "No active breakpoints.";
                        return breakpoints.IsUninitializedReadBreakArmed
                            ? $"{none}\n  (halting on uninitialized reads of {breakpoints.UninitializedReadSpace})"
                            : none;
                    }
                    var labels = target.Labels;
                    var lines = list.Select(b =>
                        $"  #{b.Id}: {Describe(labels, b.Address, b.EndAddress)}{Clauses(b.Condition, b.LogExpression)} ({(b.Enabled ? "enabled" : "disabled")}, {(b.LogExpression == null ? "hit" : "logged")} {b.HitCount}x)")
                        .Concat(dataList.Select(b =>
                        $"  #{b.Id}: {(b.OnRead ? "read" : "write")} {b.Space} {DataRange(b.Address, b.EndAddress)}{(b.Value < 0 ? string.Empty : $" = 0x{b.Value:X2}")}{(b.ChangedOnly ? " changed" : string.Empty)}{Clauses(b.Condition, b.LogExpression)} ({(b.Enabled ? "enabled" : "disabled")}, {(b.LogExpression == null ? "hit" : "logged")} {b.HitCount}x)"))
                        .Concat(forbidList.Select(r =>
                        $"  #{r.Id}: forbid {Range(r.Address, r.EndAddress)} ({(r.Enabled ? "enabled" : "disabled")})"))
                        .ToList();
                    foreach (var c in conditionList)
                        lines.Add($"  when {c.Name}: {(c.LogOnly ? "logging" : "halting")} ({(c.Enabled ? "enabled" : "disabled")}, {c.HitCount}x)");
                    if (breakpoints.DepthGuard >= 0) lines.Add($"  depth guard: halt above depth {breakpoints.DepthGuard}");
                    if (breakpoints.IsUninitializedReadBreakArmed) lines.Add($"  uninitialized reads: {breakpoints.UninitializedReadSpace}");
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
                    string[] subcommands = { "add", "read", "write", "uninit", "when", "depth", "forbid", "log", "list", "on", "off", "remove" };
                    return $"Unknown 'bp' subcommand '{sub}'.{Suggestion.Hint(sub, subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }

        // A 24-bit CPU address, so it keeps the $XXXXXX form `callers`/`writers` print.
        private static string Range(int start, int end)
            => start == end ? $"${start:X6}" : $"${start:X6}-${end:X6}";

        // An offset into a memory space, which is not bank-addressed - see `man spaces`.
        private static string DataRange(int start, int end)
            => start == end ? $"0x{start:X}" : $"0x{start:X}-0x{end:X}";

        // A label only ever names a single address, so a range prints as one.
        private static string Describe(LabelRegistry? labels, int start, int end)
            => start == end ? (labels?.Describe(start) ?? $"${start:X6}") : Range(start, end);

        private static bool IsClauseWord(string word)
            => word.Equals("if", System.StringComparison.OrdinalIgnoreCase)
            || word.Equals("log", System.StringComparison.OrdinalIgnoreCase);

        // The trailing `if <cond>` and `log <expr>` clauses, in either order - see `man bp`.
        private static (string? Condition, string? Log) TakeClauses(string[] parts, int from)
        {
            string? condition = null;
            string? log = null;

            for (int i = from; i < parts.Length; )
            {
                if (!IsClauseWord(parts[i])) { i++; continue; }

                bool isIf = parts[i].Equals("if", System.StringComparison.OrdinalIgnoreCase);
                int start = i + 1;
                int end = start;
                while (end < parts.Length && !IsClauseWord(parts[end])) end++;

                if (end > start)
                {
                    string text = string.Join(' ', parts[start..end]);
                    if (isIf) condition = text; else log = text;
                }
                i = end;
            }
            return (condition, log);
        }

        private static string Clauses(string? condition, string? log)
            => (condition == null ? string.Empty : $" if {condition}")
             + (log == null ? string.Empty : $" log {log}");
    }
}
