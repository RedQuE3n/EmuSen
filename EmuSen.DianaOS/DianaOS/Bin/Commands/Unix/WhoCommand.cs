using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // `who` - see `man who`. A different axis from `ps`: `ps` lists
    // killable THINGS (breakpoints/watches/sessions); this lists WHO is
    // running each session. Kept as a separate command rather than a
    // column bolted onto `ps`'s table, since bp/watch rows would never
    // have a meaningful "user" value.
    public class WhoCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;
        private readonly DianaOSSessionManager? _sessions;

        public WhoCommand(Func<DianaOSInterpreter> self, DianaOSSessionManager? sessions)
        {
            _self = self;
            _sessions = sessions;
        }

        public string Name => "who";
        public bool IsReadOnly => true;
        public string Usage => "  who                           list every session and which account it's running as";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            // No session manager registered for this host (see `man
            // tmux`) - still useful to answer "who is this one shell
            // running as" rather than failing outright.
            if (_sessions is null) return $"  {_self().CurrentUser,-10} (this shell)";

            return string.Join('\n', _sessions.Sessions.Select(s =>
                $"  {s.Interpreter.CurrentUser,-10} {s.Name}{(s == _sessions.Current ? "  (current)" : "")}"));
        }
    }
}
