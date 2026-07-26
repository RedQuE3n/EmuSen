using System.Linq;
using static EmuSen.Shell.Commands.DebugCommandHelpers;

namespace EmuSen.Shell.Commands
{
    // Frame-scoped value logging - see FrameLogRegistry's own comment for
    // how this differs from `watch` (sampled once per frame regardless of
    // access, rather than triggered by an actual read/write). Useful for
    // "what does this value do over time" questions a write watch can't
    // answer on its own, e.g. a counter that's only ever written once at
    // level start and then left alone while other code reads it every
    // frame - a write watch would show exactly one event; a frame log
    // shows the value at every frame in between.
    public class FrameLogCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "framelog";
        public string Usage => string.Join('\n', new[]
        {
            "  framelog add <space> <addr> [<width>]",
            "                                register a per-frame value sample (width 1/2/4 bytes, default 1)",
            "  framelog list                 list active frame logs with their IDs",
            "  framelog show <id> [<count>]  show a frame log's recorded (frame, value) samples (default 20)",
            "  framelog clear <id>           clear a frame log's stored samples (doesn't remove it)",
            "  framelog remove <id>          remove a frame log entirely",
        });

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: framelog add|list|show|clear|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var frameLog = target.FrameLog;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 4) return "Usage: framelog add <space> <addr> [<width>]";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    int width = parts.Length >= 5 ? ParseHex(parts[4]) : 1;
                    if (width != 1 && width != 2 && width != 4) return "width must be 1, 2, or 4";
                    int id = frameLog.AddEntry(parts[2], addr, width);
                    return $"Frame log #{id} added: {parts[2]} 0x{addr:X} (width {width})";
                }
                case "list":
                {
                    var list = frameLog.GetEntries();
                    if (list.Count == 0) return "No active frame logs.";
                    return string.Join('\n', list.Select(e => $"  #{e.Id}: {e.SpaceName} 0x{e.Address:X} (width {e.Width})"));
                }
                case "show":
                {
                    if (parts.Length < 3) return "Usage: framelog show <id> [<count>]";
                    int id = ParseHex(parts[2]);
                    int count = parts.Length >= 4 ? ParseHex(parts[3]) : 20;
                    var samples = frameLog.GetSamples(id, count);
                    if (samples.Count == 0) return $"No samples recorded for frame log #{id} (or it doesn't exist).";
                    return string.Join('\n', samples.Select(s => $"  frame {s.FrameCount}: 0x{s.Value:X}"));
                }
                case "clear":
                {
                    if (parts.Length < 3) return "Usage: framelog clear <id>";
                    frameLog.ClearSamples(ParseHex(parts[2]));
                    return $"Cleared samples for frame log #{parts[2]}.";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: framelog remove <id>";
                    bool removed = frameLog.RemoveEntry(ParseHex(parts[2]));
                    return removed ? $"Frame log #{parts[2]} removed." : $"No frame log #{parts[2]} found.";
                }
                default:
                    return $"Unknown 'framelog' subcommand '{sub}'. Try add/list/show/clear/remove.";
            }
        }
    }
}
