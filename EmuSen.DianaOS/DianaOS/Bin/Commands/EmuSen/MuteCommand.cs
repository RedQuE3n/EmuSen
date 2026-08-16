using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Muting excludes a voice from the mix without pausing its state - see §3.1a.
    public class MuteCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "mute";
        public bool IsReadOnly => false;
        public string Usage => "  mute <index> <on|off>         mute/unmute one audio channel - channel's own playback state still advances, it's just excluded from the mix";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 3) return Usage;
            if (!int.TryParse(parts[1], out int index)) return Usage;
            bool on = parts[2].Equals("on", System.StringComparison.OrdinalIgnoreCase);
            target.SetChannelMuted(index, on);
            return $"Channel {index} {(on ? "muted" : "unmuted")}.";
        }
    }
}
