using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Arms a halt on the BreakpointRegistry and signals HostAction.Step so
    // the caller's loop resumes far enough to reach it - see `man step`.
    // 's' is aliased in DianaOSInterpreter.Dispatch, not registered twice.
    public class StepCommand : IDianaOSCommand
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

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var (cpu, at) = ResolveCpu(resolved, args, 1);
            if (cpu == null && args.Length > 1 && DebugCpus.IsScopeWord(resolved.DebugCpus, args[1]))
            {
                return DianaOSResult.Fail("This cartridge has no coprocessor to step.");
            }
            if (cpu is { CanHalt: false }) return DianaOSResult.Fail($"{cpu.Name} cannot be halted by this core, so it cannot be stepped.");

            var breakpoints = cpu?.Breakpoints ?? resolved.Breakpoints;
            string mode = args.Length > at ? args[at].ToLowerInvariant() : string.Empty;

            if (mode is "over" or "out")
            {
                var stack = cpu != null ? cpu.CallStack : resolved.CallStack;
                if (stack == null) return DianaOSResult.Fail($"This core does not report a call stack, so `step {mode}` has no depth to measure against.");

                int depth = mode == "over" ? stack.Depth : stack.Depth - 1;
                if (depth < 0) return DianaOSResult.Fail("Already at the outermost frame - nothing to step out of.");

                breakpoints.ArmStepToDepth(depth);
                return DianaOSResult.Ok(mode == "over" ? "Stepping over..." : "Stepping out...", new HostAction.Step());
            }

            int count = 1;
            if (mode.Length > 0)
            {
                try { count = ParseHex(args[at]); }
                catch (Exception) { return DianaOSResult.Fail($"'{args[at]}' is not 'over', 'out', or a count."); }
                if (count < 1) return DianaOSResult.Fail("Step count must be at least 1.");
            }

            breakpoints.ArmStep(count);
            return DianaOSResult.Ok(count == 1 ? "Stepping one instruction..." : $"Stepping {count} instructions...", new HostAction.Step());
        }
    }
}
