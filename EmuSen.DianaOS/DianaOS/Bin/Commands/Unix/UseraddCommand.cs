using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // `useradd` - see `man useradd`. Root-gated - the one deliberate,
    // narrow preview of real permissions this pass adds (account
    // management itself, not a general per-command permission
    // framework - see DianaOSUserRegistry's own header comment on what's
    // still future work).
    public class UseraddCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;

        public UseraddCommand(Func<DianaOSInterpreter> self)
        {
            _self = self;
        }

        public string Name => "useradd";
        public bool IsReadOnly => false;
        public string Usage => "  useradd <name>                add a new account (root only) - see `su`";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaOSResult.Fail("useradd: usage: useradd <name>");

            if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaOSResult.Fail("useradd: only root can add users.");
            }

            string name = string.Join(' ', args.Skip(1));
            if (!DianaOSUserRegistry.TryAdd(name))
            {
                return DianaOSResult.Fail($"useradd: '{name}' already exists.");
            }

            System.IO.Directory.CreateDirectory(DianaOSSandbox.HomeDirectory(name));
            return $"Added user '{name}'.";
        }
    }
}
