using System.Linq;
using static EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    public class WatchCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "watch";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  watch add <space> <addr> <len> [write|read|both]",
            "                                register a watch, prints matching accesses live + stores them",
            "                                (default write, matching this command's original behavior)",
            "  watch list                    list active watches with their IDs and access kind",
            "  watch log <id> [<count>]      show a watch's recorded events (default 20)",
            "  watch summary <id>            group a watch's recorded events by access site (Context),",
            "                                showing each unique site once with a hit count instead of",
            "                                every individual event - the dynamic, addressing-mode-agnostic",
            "                                equivalent of `readers`/`writers`, which only find statically-",
            "                                visible absolute/absolute-long instructions and miss indexed/",
            "                                indirect access entirely. Only covers whatever's still in the",
            "                                watch's own bounded event buffer, same as `watch log`.",
            "  watch clear <id>              clear a watch's stored events (doesn't remove the watch)",
            "  watch remove <id>             remove a watch entirely",
        });

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: watch add|list|log|clear|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var watches = target.Watches;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 5) return "Usage: watch add <space> <addr> <len> [write|read|both]";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    int len = ParseHex(parts[4]);
                    WatchKind kind = WatchKind.Write;
                    if (parts.Length >= 6)
                    {
                        kind = parts[5].ToLowerInvariant() switch
                        {
                            "write" => WatchKind.Write,
                            "read" => WatchKind.Read,
                            "both" => WatchKind.Both,
                            _ => throw new System.ArgumentException($"Unknown watch kind '{parts[5]}'. Use write, read, or both.")
                        };
                    }
                    int id = watches.AddWatch(parts[2], addr, len, kind);
                    return $"Watch #{id} added: {parts[2]} 0x{addr:X}-0x{addr + len - 1:X} ({kind.ToString().ToLowerInvariant()})";
                }
                case "list":
                {
                    var list = watches.GetWatches();
                    if (list.Count == 0) return "No active watches.";
                    return string.Join('\n', list.Select(w => $"  #{w.Id}: {w.SpaceName} 0x{w.StartAddress:X}-0x{w.StartAddress + w.Length - 1:X} ({w.Kind.ToString().ToLowerInvariant()})"));
                }
                case "log":
                {
                    if (parts.Length < 3) return "Usage: watch log <id> [<count>]";
                    int id = ParseHex(parts[2]);
                    int count = parts.Length >= 4 ? ParseHex(parts[3]) : 20;
                    var events = watches.GetEvents(id, count);
                    if (events.Count == 0) return $"No events recorded for watch #{id} (or it doesn't exist).";
                    return string.Join('\n', events.Select(e => $"  [{e.Sequence}] {(e.AccessKind == WatchKind.Write ? "W" : "R")} 0x{e.Address:X} = 0x{e.Value:X2} ({e.Context})"));
                }
                case "summary":
                {
                    if (parts.Length < 3) return "Usage: watch summary <id>";
                    int id = ParseHex(parts[2]);
                    // int.MaxValue rather than a fixed count - GetEvents'
                    // Skip(Max(0, Count - maxCount)) naturally returns
                    // everything still in the watch's own bounded buffer
                    // when maxCount exceeds it, no separate "give me
                    // everything" API needed on WatchRegistry itself.
                    var events = watches.GetEvents(id, int.MaxValue);
                    if (events.Count == 0) return $"No events recorded for watch #{id} (or it doesn't exist).";

                    var sites = events
                        .GroupBy(e => (e.AccessKind, e.Context))
                        .Select(g => (g.Key.AccessKind, g.Key.Context, Count: g.Count()))
                        .OrderByDescending(s => s.Count);

                    return string.Join('\n', sites.Select(s =>
                        $"  {s.Count,6}x  {(s.AccessKind == WatchKind.Write ? "W" : "R")}  {s.Context}"));
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
                    return $"Unknown 'watch' subcommand '{sub}'. Try add/list/log/summary/clear/remove.";
            }
        }
    }
}
