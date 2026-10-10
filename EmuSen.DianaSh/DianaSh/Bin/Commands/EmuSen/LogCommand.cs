using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Live control for the master switch, without touching individual flags - see §2.
    public class LogCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "log";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  log off                       silence every logging flag at once, without changing them",
            "  log on                        restore whatever each logging flag was individually set to",
            "  log status                    show whether the master switch is currently on or off",
        });

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return Usage;

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                DianaShLogging.MasterEnabled = false;
                return "Logging disabled (individual flags unchanged).";
            }

            if (parts[1].Equals("on", StringComparison.OrdinalIgnoreCase))
            {
                DianaShLogging.MasterEnabled = true;
                return "Logging enabled.";
            }

            if (parts[1].Equals("status", StringComparison.OrdinalIgnoreCase))
            {
                return $"Logging is {(DianaShLogging.MasterEnabled ? "on" : "off")}.";
            }

            return Usage;
        }
    }
}
