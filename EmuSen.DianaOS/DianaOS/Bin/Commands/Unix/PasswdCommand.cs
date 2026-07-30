using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // `passwd` - see `man passwd`. Stores a hash for a future permission
    // system to check against (see DianaOSUserRegistry's own header
    // comment) - nothing reads it for gating yet, `su` in particular does
    // NOT consult it. No masked/interactive prompt (no reliable blocking
    // read exists across every host this shell runs under - a real
    // terminal, a GUI TextBox-driven console, a headless test) - the new
    // password is just a plain trailing argument, same "no interactive-
    // only path required" shape `bp add`/`watch add` already use.
    public class PasswdCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;

        public PasswdCommand(Func<DianaOSInterpreter> self)
        {
            _self = self;
        }

        public string Name => "passwd";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  passwd <newpassword>          change your own password",
            "  passwd <name> <newpassword>   change <name>'s password (root only)",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            switch (args.Length)
            {
                case 2:
                {
                    string self = _self().CurrentUser;
                    DianaOSUserRegistry.SetPassword(self, args[1]);
                    return $"Password updated for '{self}'.";
                }
                case 3:
                {
                    if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
                    {
                        return DianaOSResult.Fail("passwd: only root can change another user's password.");
                    }
                    string name = args[1];
                    if (!DianaOSUserRegistry.Exists(name))
                    {
                        return DianaOSResult.Fail($"passwd: no such user '{name}'.");
                    }
                    DianaOSUserRegistry.SetPassword(name, args[2]);
                    return $"Password updated for '{name}'.";
                }
                default:
                    return DianaOSResult.Fail("passwd: usage: passwd <newpassword> | passwd <name> <newpassword>");
            }
        }
    }
}
