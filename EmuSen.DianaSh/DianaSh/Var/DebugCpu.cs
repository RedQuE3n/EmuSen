using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Cauldron;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Var
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

        // Sets one named register, false if that name is not writable - see `man regs`.
        public Func<string, ulong, bool>? RegisterWriter { get; init; }

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

        // `cpu` is the main processor, the first listed, whatever the core names it - see `man cpus`.
        public static DebugCpu? Find(IReadOnlyList<DebugCpu> cpus, string name)
            => cpus.FirstOrDefault(c => string.Equals(c.Name, name, StringComparison.OrdinalIgnoreCase))
            ?? (cpus.Count > 0 && string.Equals(name, MainName, StringComparison.OrdinalIgnoreCase) ? cpus[0] : null);

        public static bool IsMain(IReadOnlyList<DebugCpu> cpus, DebugCpu? cpu)
            => cpu != null && cpus.Count > 0 && string.Equals(cpu.Name, cpus[0].Name, StringComparison.OrdinalIgnoreCase);

        public static bool IsScopeWord(IReadOnlyList<DebugCpu> cpus, string word)
            => Find(cpus, word) != null || string.Equals(word, "cop", StringComparison.OrdinalIgnoreCase);

        // What a bare `cop` means now that chips have names - see `man cpus`.
        public static DebugCpu? Coprocessor(IReadOnlyList<DebugCpu> cpus)
            => cpus.Skip(1).FirstOrDefault(c => !string.Equals(c.Name, "spc", StringComparison.OrdinalIgnoreCase));

        public static string NameList(IReadOnlyList<DebugCpu> cpus)
            => cpus.Count == 0 ? "(none)" : string.Join(", ", cpus.Select(c => c.Name));
    }
}
