using System;
using EmuSen.Debug;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // Arms/disarms DebugSettings.CpuVerboseLogging live from the F4
    // prompt, instead of requiring a rebuild and a boot-time flag - the
    // countdown is instruction-count-based and starts from whenever this
    // command runs, not from power-on, so it can be aimed at a specific
    // moment in a play session (e.g. "right before the thing I want to
    // see happens") rather than burning its whole budget during the
    // boot/reset routine. Doesn't touch the IDebugTarget at all - a
    // global settings toggle, not something scoped to a particular core
    // instance.
    public class TraceCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "trace";
        public string Usage => string.Join('\n', new[]
        {
            "  trace <count>                 arm a live CPU instruction trace for the next <count> instructions",
            "  trace off                     cancel an in-progress trace early",
        });

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return "Usage: trace <count> | trace off";

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                DebugSettings.CpuVerboseLogging = false;
                DebugSettings.CpuTraceCountdown = 0;
                return "Trace disabled.";
            }

            int count = ParseHex(parts[1]);
            DebugSettings.CpuTraceCountdown = count;
            DebugSettings.CpuVerboseLogging = true;
            return $"Trace armed for the next {count} instructions.";
        }
    }
}
