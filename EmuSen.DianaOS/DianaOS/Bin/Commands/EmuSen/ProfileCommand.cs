using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Text;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Which routines the instruction budget actually goes to - see `man profile`.
    public class ProfileCommand : IDianaOSCommand
    {
        public string Name => "profile";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  profile on                    start charging instructions to the innermost routine",
            "  profile off                   stop (collected counts stay readable)",
            "  profile clear                 zero the counts, stay armed",
            "  profile top [<n>]             hottest routines, default 20",
            "  profile <cpu> on|off|top ...  same, on another processor - `cpus` lists them",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var (cpu, at) = ResolveCpu(resolved, parts, 1);
            if (cpu == null && parts.Length > 1 && DebugCpus.IsScopeWord(resolved.DebugCpus, parts[1]))
            {
                return DianaOSResult.Fail("This cartridge has no coprocessor to profile.");
            }

            var stack = cpu != null ? cpu.CallStack : resolved.CallStack;
            if (stack == null)
            {
                return DianaOSResult.Fail(cpu != null && cpu.Name != DebugCpus.MainName
                    ? $"{cpu.Name} does not report a call stack, so it cannot be profiled - see `man cpus`."
                    : "This core does not report a call stack, so it cannot be profiled.");
            }
            if (parts.Length <= at) return "Usage: profile on|off|clear|top ...";

            switch (parts[at].ToLowerInvariant())
            {
                case "on":
                    stack.ArmProfiler();
                    return "Profiling instructions per routine.";
                case "off":
                    stack.DisarmProfiler();
                    return $"Profiling stopped after {stack.ProfiledInstructions} instruction(s).";
                case "clear":
                    stack.ClearProfile();
                    return "Profile cleared.";
                case "top":
                {
                    int limit = parts.Length > at + 1 ? ParseHex(parts[at + 1]) : 20;
                    var rows = stack.Hottest(limit);
                    if (rows.Count == 0) return stack.IsProfiling ? "Nothing profiled yet." : "Not profiling - run `profile on` first.";
                    string header = $"{stack.ProfiledInstructions} instruction(s) profiled{(stack.IsProfiling ? string.Empty : ", profiling stopped")}:";
                    return header + "\n" + string.Join('\n', rows.Select(r =>
                        $"  {r.Percent,6:F2}%  {r.Instructions,-12} {r.Calls,-8} {Describe(resolved, r.Address)}"));
                }
                default:
                {
                    string[] subcommands = { "on", "off", "clear", "top" };
                    return $"Unknown 'profile' subcommand '{parts[at]}'.{Suggestion.Hint(parts[at], subcommands)} Try {string.Join('/', subcommands)}.";
                }
            }
        }

        // Address 0 is the "not inside any recorded call" bucket - see `man profile`.
        private static string Describe(IDebugTarget target, int address)
            => address == 0 ? "(outside any recorded call)" : target.Labels?.Describe(address) ?? $"${address:X6}";
    }
}
