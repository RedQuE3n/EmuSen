using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Arms the halt, then signals Step so the caller's loop reaches it - see `man step`.
    public class StepCommand : IDianaShCommand
    {
        public string Name => "step";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  step | s [<count>]            single-step one CPU instruction (or <count> of them)",
            "  step over                     run the call at the current instruction to completion, then halt",
            "  step out                      run until the current routine returns to its caller",
            "  step <cpu> [<count>|over|out] same, stepping another processor - `cpus` lists them",
        });

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaShResult.Fail(ex.Message); }

            var (cpu, at) = ResolveCpu(resolved, args, 1);
            if (cpu == null && args.Length > 1 && DebugCpus.IsScopeWord(resolved.DebugCpus, args[1]))
            {
                return DianaShResult.Fail("This cartridge has no coprocessor to step.");
            }
            if (cpu is { CanHalt: false }) return DianaShResult.Fail($"{cpu.Name} cannot be halted by this core, so it cannot be stepped.");

            var breakpoints = cpu?.Breakpoints ?? resolved.Breakpoints;
            string mode = args.Length > at ? args[at].ToLowerInvariant() : string.Empty;

            if (mode is "over" or "out")
            {
                var stack = cpu != null ? cpu.CallStack : resolved.CallStack;
                if (stack == null) return DianaShResult.Fail($"This core does not report a call stack, so `step {mode}` has no depth to measure against.");

                int depth = mode == "over" ? stack.Depth : stack.Depth - 1;
                if (depth < 0) return DianaShResult.Fail("Already at the outermost frame - nothing to step out of.");

                breakpoints.ArmStepToDepth(depth);
                return DianaShResult.Ok(mode == "over" ? "Stepping over..." : "Stepping out...", new HostAction.Step());
            }

            int count = 1;
            if (mode.Length > 0)
            {
                try { count = ParseHex(args[at]); }
                catch (Exception) { return DianaShResult.Fail($"'{args[at]}' is not 'over', 'out', or a count."); }
                if (count < 1) return DianaShResult.Fail("Step count must be at least 1.");
            }

            breakpoints.ArmStep(count);
            return DianaShResult.Ok(count == 1 ? "Stepping one instruction..." : $"Stepping {count} instructions...", new HostAction.Step());
        }
    }
}
