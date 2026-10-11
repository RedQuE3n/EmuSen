using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Var.Shell;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Who is running each session, an axis `ps` rows could not carry - see `man who`.
    public class WhoCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;
        private readonly DianaShSessionManager? _sessions;

        public WhoCommand(Func<DianaShInterpreter> self, DianaShSessionManager? sessions)
        {
            _self = self;
            _sessions = sessions;
        }

        public string Name => "who";
        public bool IsReadOnly => true;
        public string Usage => "  who                           list every session and which account it's running as";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            // With no session manager, still answer for this one shell - see `man tmux`.
            if (_sessions is null) return $"  {_self().CurrentUser,-10} (this shell)";

            return string.Join('\n', _sessions.Sessions.Select(s =>
                $"  {s.Interpreter.CurrentUser,-10} {s.Name}{(s == _sessions.Current ? "  (current)" : "")}"));
        }
    }
}
