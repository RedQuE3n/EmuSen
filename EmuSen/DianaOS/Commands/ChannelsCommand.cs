using System.Text;

namespace EmuSen.DianaOS.Commands
{
    // Lists whatever audio channels/voices IDebugTarget.GetAudioChannels()
    // reports - core-agnostic on purpose, same split as `sprites`/`pal`:
    // this command just formats a table, it has no idea whether it's
    // looking at 8 SNES DSP voices or some other core's completely
    // different channel model. Built alongside `mute` (same file's
    // sibling) diagnosing a "part of the music is missing" report, where
    // the only prior visibility into per-voice state was either the final
    // mixed audio (`audiodump`) or a KeyOn event as it happened
    // (console-only DspKeyOnLogging) - neither answers "is this voice
    // active right now, and what's its envelope actually doing."
    public class ChannelsCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "channels";
        public bool IsReadOnly => true;
        public string Usage => "  channels                      list audio channels/voices - index, active, envelope level (0-100), muted, core-specific detail (see `mute` to isolate one)";

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
            var channels = target.GetAudioChannels();
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
