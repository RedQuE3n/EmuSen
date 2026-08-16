using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Live control for the master switch, without touching individual flags - see §2.
    public class LogCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "log";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  log off                       silence every logging flag at once, without changing them",
            "  log on                        restore whatever each logging flag was individually set to",
            "  log status                    show whether the master switch is currently on or off",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return Usage;

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                DianaOSLogging.MasterEnabled = false;
                return "Logging disabled (individual flags unchanged).";
            }

            if (parts[1].Equals("on", StringComparison.OrdinalIgnoreCase))
            {
                DianaOSLogging.MasterEnabled = true;
                return "Logging enabled.";
            }

            if (parts[1].Equals("status", StringComparison.OrdinalIgnoreCase))
            {
                return $"Logging is {(DianaOSLogging.MasterEnabled ? "on" : "off")}.";
            }

            return Usage;
        }
    }
}
