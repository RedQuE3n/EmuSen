using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Arms a one-shot halt-before-next-instruction (BreakpointRegistry.
    // ArmSingleStep) and signals HostAction.Step so the caller's own loop
    // resumes just long enough to let that one instruction run - promoted
    // from a hand-rolled string match inside EmuSen.Hotaru's own
    // RunDebugPrompt into a real, shared command. Unlike Resume/Shutdown,
    // this one genuinely needs IDebugTarget access (to arm the single-
    // step on the actual BreakpointRegistry), so it isn't just a pure
    // signal - RequireTarget gives it the same clean "No ROM loaded"
    // failure every other target-touching command already has instead of
    // a NullReferenceException. 's' is recognized as an alias directly in
    // DianaOSInterpreter.Dispatch (normalized to 'step' before the
    // registry lookup), not a second registry entry.
    public class StepCommand : IDianaOSCommand
    {
        public string Name => "step";
        public bool IsReadOnly => false;
        public string Usage => "  step | s                      single-step one CPU instruction, then halt again";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            resolved.Breakpoints.ArmSingleStep();
            return DianaOSResult.Ok("Stepping one instruction...", new HostAction.Step());
        }
    }
}
