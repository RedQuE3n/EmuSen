using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Who is running each session, an axis `ps` rows could not carry - see `man who`.
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
            // With no session manager, still answer for this one shell - see `man tmux`.
            if (_sessions is null) return $"  {_self().CurrentUser,-10} (this shell)";

            return string.Join('\n', _sessions.Sessions.Select(s =>
                $"  {s.Interpreter.CurrentUser,-10} {s.Name}{(s == _sessions.Current ? "  (current)" : "")}"));
        }
    }
}
