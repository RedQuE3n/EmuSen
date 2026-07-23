using System.Linq;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    public class WatchCommand : IDebugCommand
    {
        public string Name => "watch";
        public string Usage => string.Join('\n', new[]
        {
            "  watch add <space> <addr> <len> register a watch, prints matching writes live + stores them",
            "  watch list                    list active watches with their IDs",
            "  watch log <id> [<count>]      show a watch's recorded events (default 20)",
            "  watch clear <id>              clear a watch's stored events (doesn't remove the watch)",
            "  watch remove <id>             remove a watch entirely",
        });

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 2) return "Usage: watch add|list|log|clear|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var watches = target.Watches;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 5) return "Usage: watch add <space> <addr> <len>";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    int len = ParseHex(parts[4]);
                    int id = watches.AddWatch(parts[2], addr, len);
                    return $"Watch #{id} added: {parts[2]} 0x{addr:X}-0x{addr + len - 1:X}";
                }
                case "list":
                {
                    var list = watches.GetWatches();
                    if (list.Count == 0) return "No active watches.";
                    return string.Join('\n', list.Select(w => $"  #{w.Id}: {w.SpaceName} 0x{w.StartAddress:X}-0x{w.StartAddress + w.Length - 1:X}"));
                }
                case "log":
                {
                    if (parts.Length < 3) return "Usage: watch log <id> [<count>]";
                    int id = ParseHex(parts[2]);
                    int count = parts.Length >= 4 ? ParseHex(parts[3]) : 20;
                    var events = watches.GetEvents(id, count);
                    if (events.Count == 0) return $"No events recorded for watch #{id} (or it doesn't exist).";
                    return string.Join('\n', events.Select(e => $"  [{e.Sequence}] 0x{e.Address:X} = 0x{e.Value:X2} ({e.Context})"));
                }
                case "clear":
                {
                    if (parts.Length < 3) return "Usage: watch clear <id>";
                    watches.ClearEvents(ParseHex(parts[2]));
                    return $"Cleared events for watch #{parts[2]}.";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: watch remove <id>";
                    bool removed = watches.RemoveWatch(ParseHex(parts[2]));
                    return removed ? $"Watch #{parts[2]} removed." : $"No watch #{parts[2]} found.";
                }
                default:
                    return $"Unknown 'watch' subcommand '{sub}'. Try add/list/log/clear/remove.";
            }
        }
    }
}
