using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Arms a core's trace through an injected switch, counted from now - see §3.3b.
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
