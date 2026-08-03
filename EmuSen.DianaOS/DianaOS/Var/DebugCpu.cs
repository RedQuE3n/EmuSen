using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Var
{
    // One separately-steppable processor a debugger can aim at - see `man cpus`.
    public sealed class DebugCpu
    {
        // Lowercase, and what a scoped command's scope word matches.
        public string Name { get; }

        public string Description { get; }

        // Never null, so a scoped command needs no second null check.
        public BreakpointRegistry Breakpoints { get; }

        public CoverageRegistry? Coverage { get; init; }
        public CallStackRegistry? CallStack { get; init; }
        public IExpressionContext? Expressions { get; init; }

        // Which memory space `disasm <cpu>` defaults to.
        public string? CodeSpace { get; init; }

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>>? Registers { get; init; }

        public Func<int>? ProgramCounter { get; init; }

        // False when the core can observe this chip but not halt it.
        public bool CanHalt { get; init; } = true;

        public DebugCpu(string name, string description, BreakpointRegistry breakpoints)
        {
            Name = name;
            Description = description;
            Breakpoints = breakpoints;
        }
    }

    // Scope-word resolution, here rather than in the shell so it is testable without one.
    public static class DebugCpus
    {
        public const string MainName = "cpu";

        public static DebugCpu? Find(IReadOnlyList<DebugCpu> cpus, string name)
            => cpus.FirstOrDefault(c => string.Equals(c.Name, name, StringComparison.OrdinalIgnoreCase));

        public static bool IsScopeWord(IReadOnlyList<DebugCpu> cpus, string word)
            => Find(cpus, word) != null || string.Equals(word, "cop", StringComparison.OrdinalIgnoreCase);

        // What a bare `cop` means now that chips have names - see `man cpus`.
        public static DebugCpu? Coprocessor(IReadOnlyList<DebugCpu> cpus)
            => cpus.FirstOrDefault(c => !string.Equals(c.Name, MainName, StringComparison.OrdinalIgnoreCase)
                                     && !string.Equals(c.Name, "spc", StringComparison.OrdinalIgnoreCase));

        public static string NameList(IReadOnlyList<DebugCpu> cpus)
            => cpus.Count == 0 ? "(none)" : string.Join(", ", cpus.Select(c => c.Name));
    }
}
