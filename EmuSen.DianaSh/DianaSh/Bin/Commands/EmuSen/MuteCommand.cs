using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Muting excludes a voice from the mix without pausing its state - see §3.1a.
    public class MuteCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "mute";
        public bool IsReadOnly => false;
        public string Usage => "  mute <index> <on|off>         mute/unmute one audio channel - channel's own playback state still advances, it's just excluded from the mix";

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 3) return Usage;
            if (!int.TryParse(parts[1], out int index)) return Usage;
            bool on = parts[2].Equals("on", System.StringComparison.OrdinalIgnoreCase);
            target.SetChannelMuted(index, on);
            return $"Channel {index} {(on ? "muted" : "unmuted")}.";
        }
    }
}
