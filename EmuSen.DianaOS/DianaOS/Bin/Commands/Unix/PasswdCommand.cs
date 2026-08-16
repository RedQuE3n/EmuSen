using System;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Stores a hash nothing gates on yet, and takes it as a plain argument - see `man passwd`.
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
