using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Root-gated - the one narrow preview of real permissions - see `man useradd`.
    public class UseraddCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;

        public UseraddCommand(Func<DianaShInterpreter> self)
        {
            _self = self;
        }

        public string Name => "useradd";
        public bool IsReadOnly => false;
        public string Usage => "  useradd <name>                add a new account (root only) - see `su`";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length < 2) return DianaShResult.Fail("useradd: usage: useradd <name>");

            if (!_self().CurrentUser.Equals("root", StringComparison.OrdinalIgnoreCase))
            {
                return DianaShResult.Fail("useradd: only root can add users.");
            }

            string name = string.Join(' ', args.Skip(1));
            if (!DianaShUserRegistry.TryAdd(name))
            {
                return DianaShResult.Fail($"useradd: '{name}' already exists.");
            }

            // An account is an identity, not a directory - see `man hier`.
            return $"Added user '{name}'.";
        }
    }
}
