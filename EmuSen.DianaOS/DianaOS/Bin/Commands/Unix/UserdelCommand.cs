using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Refuses root, and refuses an account any live session is using - see `man userdel`.
    public class UserdelCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;
        private readonly DianaOSSessionManager? _sessions;

        public UserdelCommand(Func<DianaOSInterpreter> self, DianaOSSessionManager? sessions)
        {
            _self = self;
            _sessions = sessions;
        }

        public string Name => "userdel";
        public bool IsReadOnly => false;
        public string Usage => "  userdel <name>                remove an account (root only) - refuses root itself or one currently in use";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("userdel: usage: userdel <name>");

            if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaOSResult.Fail("userdel: only root can remove users.");
            }

            string name = string.Join(' ', args.Skip(1));

            if (!DianaOSUserRegistry.Exists(name))
            {
                return DianaOSResult.Fail($"userdel: no such user '{name}'.");
            }

            if (name.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaOSResult.Fail("userdel: refusing to remove 'root'.");
            }

            if (_sessions != null)
            {
                DianaOSSession? activeIn = _sessions.Sessions.FirstOrDefault(
                    s => s.Interpreter.CurrentUser.Equals(name, StringComparison.OrdinalIgnoreCase));
                if (activeIn != null)
                {
                    return DianaOSResult.Fail($"userdel: '{name}' is currently in use by session '{activeIn.Name}' - `su` that session to another account first.");
                }
            }
            else if (_self().CurrentUser.Equals(name, StringComparison.OrdinalIgnoreCase))
            {
                return DianaOSResult.Fail($"userdel: '{name}' is the account this shell is currently running as - `su` to another account first.");
            }

            DianaOSUserRegistry.TryRemove(name);
            return $"Removed user '{name}'.";
        }
    }
}
