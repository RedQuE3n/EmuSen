using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Stores a hash nothing gates on yet, and takes it as a plain argument - see `man passwd`.
    public class PasswdCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;

        public PasswdCommand(Func<DianaShInterpreter> self)
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

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            switch (args.Length)
            {
                case 2:
                {
                    string self = _self().CurrentUser;
                    DianaShUserRegistry.SetPassword(self, args[1]);
                    return $"Password updated for '{self}'.";
                }
                case 3:
                {
                    if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
                    {
                        return DianaShResult.Fail("passwd: only root can change another user's password.");
                    }
                    string name = args[1];
                    if (!DianaShUserRegistry.Exists(name))
                    {
                        return DianaShResult.Fail($"passwd: no such user '{name}'.");
                    }
                    DianaShUserRegistry.SetPassword(name, args[2]);
                    return $"Password updated for '{name}'.";
                }
                default:
                    return DianaShResult.Fail("passwd: usage: passwd <newpassword> | passwd <name> <newpassword>");
            }
        }
    }
}
