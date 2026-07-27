using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Arms/disarms a core's own live CPU instruction trace from the F4
    // prompt, instead of requiring a rebuild and a boot-time flag - the
    // countdown is instruction-count-based and starts from whenever this
    // command runs, not from power-on, so it can be aimed at a specific
    // moment in a play session (e.g. "right before the thing I want to
    // see happens") rather than burning its whole budget during the
    // boot/reset routine. Doesn't touch the IDebugTarget at all - a
    // global settings toggle, not something scoped to a particular core
    // instance.
    //
    // Genuinely core-agnostic now: arms/disarms through whichever
    // ICpuTraceSwitch instance the host passes in (see
    // DianaOSInterpreter.CreateDefault's cpuTraceSwitch parameter),
    // rather than reaching into one specific core's own settings class
    // directly - same reasoning as CheatCommand's injected codecs.
    // Without one registered (a future core with no equivalent trace
    // mechanism, or a standalone launch with no core at all), this just
    // reports there's nothing to arm.
    public class TraceCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        private readonly ICpuTraceSwitch? _traceSwitch;

        public TraceCommand(ICpuTraceSwitch? traceSwitch = null)
        {
            _traceSwitch = traceSwitch;
        }

        public string Name => "trace";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  trace <count>                 arm a live CPU instruction trace for the next <count> instructions",
            "  trace off                     cancel an in-progress trace early",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return "Usage: trace <count> | trace off";
            if (_traceSwitch is null) return "No CPU trace switch is registered for this target.";

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                _traceSwitch.Disarm();
                return "Trace disabled.";
            }

            int count = ParseHex(parts[1]);
            _traceSwitch.Arm(count);
            return $"Trace armed for the next {count} instructions.";
        }
    }
}
