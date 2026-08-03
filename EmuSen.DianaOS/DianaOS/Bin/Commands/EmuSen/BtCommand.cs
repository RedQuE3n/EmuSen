using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // The live call/return chain - see `man bt`.
    public class BtCommand : IDianaOSCommand
    {
        public string Name => "bt";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  bt [<count>]                  backtrace: the live call chain, innermost frame first",
            "  bt reset                      forget the current chain (after a stack-manipulating routine)",
            "  bt <cpu> [<count>|reset]      same, on another processor - `cpus` lists them",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var (cpu, at) = ResolveCpu(resolved, parts, 1);
            if (cpu == null && parts.Length > 1 && DebugCpus.IsScopeWord(resolved.DebugCpus, parts[1]))
            {
                return DianaOSResult.Fail("This cartridge has no coprocessor to report a call stack for.");
            }

            var stack = cpu != null ? cpu.CallStack : resolved.CallStack;
            if (stack == null)
            {
                return DianaOSResult.Fail(cpu != null && cpu.Name != DebugCpus.MainName
                    ? $"{cpu.Name} does not report a call stack - see `man cpus`."
                    : "This core does not report a call stack.");
            }

            if (parts.Length > at && parts[at].Equals("reset", StringComparison.OrdinalIgnoreCase))
            {
                stack.Reset();
                return "Call stack cleared.";
            }

            int limit = parts.Length > at ? ParseHex(parts[at]) : 32;
            var frames = stack.Backtrace();
            if (frames.Count == 0)
            {
                return stack.UnmatchedReturns > 0
                    ? $"Call stack empty ({stack.UnmatchedReturns} unmatched return(s) seen)."
                    : "Call stack empty - nothing has been called since the last reset.";
            }

            var labels = resolved.Labels;
            var lines = new List<string>();
            int depth = frames.Count;
            foreach (var frame in frames.Take(limit))
            {
                string kind = frame.Kind == CallFrameKind.Call ? "call" : frame.Kind.ToString().ToUpperInvariant();
                lines.Add($"  #{depth - 1,-3} {Describe(labels, frame.Target)}  <- {kind} from {Describe(labels, frame.Source)}  (frame {frame.FrameNumber})");
                depth--;
            }
            if (frames.Count > limit) lines.Add($"  ... {frames.Count - limit} more frame(s)");
            if (stack.UnmatchedReturns > 0) lines.Add($"  ({stack.UnmatchedReturns} unmatched return(s) seen - see `man bt`)");
            return string.Join('\n', lines);
        }

        private static string Describe(LabelRegistry? labels, int address)
            => labels?.Describe(address) ?? $"${address:X6}";
    }
}
