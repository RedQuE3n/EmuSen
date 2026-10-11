using System;
using EmuSen.DianaSh.DianaSh.Lib;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Live control for the emulator's master logging switch, handed in rather than reached - see `man log`.
    public class LogCommand : IDianaShCommand
    {
        private readonly Func<bool> _isEnabled;
        private readonly Action<bool> _setEnabled;

        public LogCommand(Func<bool> isEnabled, Action<bool> setEnabled)
        {
            _isEnabled = isEnabled;
            _setEnabled = setEnabled;
        }

        public string Name => "log";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  log off                       silence every logging flag at once, without changing them",
            "  log on                        restore whatever each logging flag was individually set to",
            "  log status                    show whether the master switch is currently on or off",
        });

        public DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return Usage;

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                _setEnabled(false);
                return "Logging disabled (individual flags unchanged).";
            }

            if (parts[1].Equals("on", StringComparison.OrdinalIgnoreCase))
            {
                _setEnabled(true);
                return "Logging enabled.";
            }

            if (parts[1].Equals("status", StringComparison.OrdinalIgnoreCase))
            {
                return $"Logging is {(_isEnabled() ? "on" : "off")}.";
            }

            return Usage;
        }
    }
}
