using System;
using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Unix flavour, not access control: the stored hash gates nothing - see `man su`.
    public class SuCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;

        public SuCommand(Func<DianaShInterpreter> self)
        {
            _self = self;
        }

        public string Name => "su";
        public bool IsReadOnly => false;
        public string Usage => "  su [name]                     switch this shell to <name> (default: root) - see `useradd`";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string name = args.Length >= 2 ? string.Join(' ', args.Skip(1)) : "root";

            if (!DianaShUserRegistry.Exists(name))
            {
                return DianaShResult.Fail($"su: no such user '{name}' - see `useradd`.");
            }

            _self().CurrentUser = name;
            return $"Switched to '{name}'.";
        }
    }
}
