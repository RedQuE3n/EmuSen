using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Formats whatever AudioChannels reports; the model is the core's - see `man channels`.
    public class ChannelsCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "channels";
        public bool IsReadOnly => true;
        public string Usage => "  channels                      list audio channels/voices - index, active, envelope level (0-100), muted, core-specific detail (see `mute` to isolate one)";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
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
