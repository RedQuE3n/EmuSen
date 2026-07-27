using System;
using System.Linq;

namespace EmuSen.DianaOS.Commands
{
    // `kill` - see `man kill`. One verb that removes a breakpoint, a watch,
    // or a tmux session, dispatching on the shape of <id> exactly as `ps`
    // prints it ("bp<N>"/"watch<N>"/a session name) - routes straight to
    // whichever registry already owns that kind of thing (BreakpointRegistry/
    // WatchRegistry/DianaOSSessionManager) rather than reimplementing any of
    // their own removal logic (including DianaOSSessionManager's "can't kill
    // the only remaining session" guard and its current-session hand-off).
    // Doesn't replace `bp remove`/`watch remove`/`tmux kill` - those still
    // work exactly as before - it's just a single memorable verb that
    // doesn't require knowing which of the three sub-shells owns a given id.
    public class KillCommand : IDianaOSCommand
    {
        private readonly DianaOSSessionManager? _sessions;

        public KillCommand(DianaOSSessionManager? sessions)
        {
            _sessions = sessions;
        }

        public string Name => "kill";
        public bool IsReadOnly => false;
        public string Usage => "  kill <id>                     remove a breakpoint/watch/session by its `ps` id (bp<N>, watch<N>, or a session name)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("kill: usage: kill <id>");
            string id = string.Join(' ', args.Skip(1));

            if (id.StartsWith("bp", StringComparison.OrdinalIgnoreCase)
                && int.TryParse(id.Substring(2), out int bpId))
            {
                var t = DebugCommandHelpers.RequireTarget(target);
                return t.Breakpoints.RemoveBreakpoint(bpId)
                    ? $"Breakpoint #{bpId} removed."
                    : DianaOSResult.Fail($"kill: no breakpoint #{bpId} found.");
            }

            if (id.StartsWith("watch", StringComparison.OrdinalIgnoreCase)
                && int.TryParse(id.Substring(5), out int watchId))
            {
                var t = DebugCommandHelpers.RequireTarget(target);
                return t.Watches.RemoveWatch(watchId)
                    ? $"Watch #{watchId} removed."
                    : DianaOSResult.Fail($"kill: no watch #{watchId} found.");
            }

            if (_sessions is null)
            {
                return DianaOSResult.Fail($"kill: no session manager is registered for this host, and '{id}' isn't a bp<N>/watch<N> id.");
            }

            bool wasCurrent = _sessions.Current?.Name.Equals(id, StringComparison.OrdinalIgnoreCase) == true;
            if (!_sessions.TryKill(id, out string error)) return DianaOSResult.Fail($"kill: {error}");
            if (wasCurrent)
            {
                return DianaOSResult.Ok($"[kill] Killed session '{id}'; switched to '{_sessions.Current!.Name}'.", new HostAction.SwitchSession(_sessions.Current!.Name));
            }
            return $"Session '{id}' removed.";
        }
    }
}
