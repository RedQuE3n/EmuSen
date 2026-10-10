using System.Text;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Formats whatever AudioChannels reports; the model is the core's - see `man channels`.
    public class ChannelsCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "channels";
        public bool IsReadOnly => true;
        public string Usage => "  channels                      list audio channels/voices - index, active, envelope level (0-100), muted, core-specific detail (see `mute` to isolate one)";

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            var channels = target.AudioChannels.Current;
            if (channels.Count == 0) return "(this core reports no audio channels)";

            var sb = new StringBuilder();
            foreach (var c in channels)
            {
                sb.AppendLine($"[{c.Index}] {c.Name,-8} active={c.Active,-5} level={c.Level,3} muted={c.Muted,-5} {c.Info}");
            }
            return sb.ToString().TrimEnd('\n');
        }
    }
}
