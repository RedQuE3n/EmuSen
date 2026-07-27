namespace EmuSen.DianaOS.Commands.EmuSen
{
    // Solo/mute one audio channel for isolation testing - see
    // IDebugTarget.SetChannelMuted's own comment on why muting doesn't
    // pause that channel's own playback/envelope state, only excludes it
    // from the final mix. Pairs with `channels` (this file's sibling) and
    // `audiodump`/GetAudioSamples: mute every voice except one suspect,
    // dump audio, and hear (or measure) exactly what that voice produces
    // in isolation, without needing a separate solo-rendering pipeline.
    public class MuteCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "mute";
        public bool IsReadOnly => false;
        public string Usage => "  mute <index> <on|off>         mute/unmute one audio channel - channel's own playback state still advances, it's just excluded from the mix";

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 3) return Usage;
            if (!int.TryParse(parts[1], out int index)) return Usage;
            bool on = parts[2].Equals("on", System.StringComparison.OrdinalIgnoreCase);
            target.SetChannelMuted(index, on);
            return $"Channel {index} {(on ? "muted" : "unmuted")}.";
        }
    }
}
