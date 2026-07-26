using System;
using EmuSen.Debug;

namespace EmuSen.DianaOS.Commands
{
    // Live control for DebugSettings.MasterLoggingEnabled - the single
    // switch that silences every *Logging flag at once (see that field's
    // own comment) without touching any of their individually-set values.
    // Doesn't touch the IDebugTarget at all, same as TraceCommand - a
    // global settings toggle, not something scoped to a particular core
    // instance.
    public class LogCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "log";
        public string Usage => string.Join('\n', new[]
        {
            "  log off                       silence every logging flag at once, without changing them",
            "  log on                        restore whatever each logging flag was individually set to",
            "  log status                    show whether the master switch is currently on or off",
        });

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return Usage;

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                DebugSettings.MasterLoggingEnabled = false;
                return "Logging disabled (individual flags unchanged).";
            }

            if (parts[1].Equals("on", StringComparison.OrdinalIgnoreCase))
            {
                DebugSettings.MasterLoggingEnabled = true;
                return "Logging enabled.";
            }

            if (parts[1].Equals("status", StringComparison.OrdinalIgnoreCase))
            {
                return $"Logging is {(DebugSettings.MasterLoggingEnabled ? "on" : "off")}.";
            }

            return Usage;
        }
    }
}
