using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // One verb that dispatches on the shape of the id `ps` printed - see `man kill`.
    public class KillCommand : IDianaShCommand
    {
        private readonly DianaShSessionManager? _sessions;

        public KillCommand(DianaShSessionManager? sessions)
        {
            _sessions = sessions;
        }

        public string Name => "kill";
        public bool IsReadOnly => false;
        public string Usage => "  kill <id>                     remove a breakpoint/watch/session by its `ps` id (bp<N>, watch<N>, or a session name)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaShResult.Fail("kill: usage: kill <id>");
            string id = string.Join(' ', args.Skip(1));

            if (id.StartsWith("bp", StringComparison.OrdinalIgnoreCase)
                && int.TryParse(id.Substring(2), out int bpId))
            {
                var t = DebugCommandHelpers.RequireTarget(target);
                return t.Breakpoints.RemoveBreakpoint(bpId)
                    ? $"Breakpoint #{bpId} removed."
                    : DianaShResult.Fail($"kill: no breakpoint #{bpId} found.");
            }

            if (id.StartsWith("watch", StringComparison.OrdinalIgnoreCase)
                && int.TryParse(id.Substring(5), out int watchId))
            {
                var t = DebugCommandHelpers.RequireTarget(target);
                return t.Watches.RemoveWatch(watchId)
                    ? $"Watch #{watchId} removed."
                    : DianaShResult.Fail($"kill: no watch #{watchId} found.");
            }

            if (_sessions is null)
            {
                return DianaShResult.Fail($"kill: no session manager is registered for this host, and '{id}' isn't a bp<N>/watch<N> id.");
            }

            bool wasCurrent = _sessions.Current?.Name.Equals(id, StringComparison.OrdinalIgnoreCase) == true;
            if (!_sessions.TryKill(id, out string error)) return DianaShResult.Fail($"kill: {error}");
            if (wasCurrent)
            {
                return DianaShResult.Ok($"[kill] Killed session '{id}'; switched to '{_sessions.Current!.Name}'.", new HostAction.SwitchSession(_sessions.Current!.Name));
            }
            return $"Session '{id}' removed.";
        }
    }
}
