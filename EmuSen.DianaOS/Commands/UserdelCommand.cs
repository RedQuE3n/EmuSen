using System;
using System.Linq;

namespace EmuSen.DianaOS.Commands
{
    // `userdel` - see `man userdel`. Root-gated, same reasoning as
    // `useradd`. Refuses to remove 'root' itself (DianaOSUserRegistry.
    // TryRemove already guards this at the registry level; this command
    // just surfaces a clean message instead of a silent no-op) and
    // refuses to remove an account that's currently in use by ANY live
    // session (or by this shell itself, when no session manager is
    // registered) - mirrors `kill`'s own "can't kill the only remaining
    // session" style guard, just checked before mutating instead of after.
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
