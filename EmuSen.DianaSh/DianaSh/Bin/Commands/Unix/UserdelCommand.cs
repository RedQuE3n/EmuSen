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
    // Refuses root, and refuses an account any live session is using - see `man userdel`.
    public class UserdelCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;
        private readonly DianaShSessionManager? _sessions;

        public UserdelCommand(Func<DianaShInterpreter> self, DianaShSessionManager? sessions)
        {
            _self = self;
            _sessions = sessions;
        }

        public string Name => "userdel";
        public bool IsReadOnly => false;
        public string Usage => "  userdel <name>                remove an account (root only) - refuses root itself or one currently in use";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaShResult.Fail("userdel: usage: userdel <name>");

            if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaShResult.Fail("userdel: only root can remove users.");
            }

            string name = string.Join(' ', args.Skip(1));

            if (!DianaShUserRegistry.Exists(name))
            {
                return DianaShResult.Fail($"userdel: no such user '{name}'.");
            }

            if (name.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaShResult.Fail("userdel: refusing to remove 'root'.");
            }

            if (_sessions != null)
            {
                DianaShSession? activeIn = _sessions.Sessions.FirstOrDefault(
                    s => s.Interpreter.CurrentUser.Equals(name, StringComparison.OrdinalIgnoreCase));
                if (activeIn != null)
                {
                    return DianaShResult.Fail($"userdel: '{name}' is currently in use by session '{activeIn.Name}' - `su` that session to another account first.");
                }
            }
            else if (_self().CurrentUser.Equals(name, StringComparison.OrdinalIgnoreCase))
            {
                return DianaShResult.Fail($"userdel: '{name}' is the account this shell is currently running as - `su` to another account first.");
            }

            DianaShUserRegistry.TryRemove(name);
            return $"Removed user '{name}'.";
        }
    }
}
