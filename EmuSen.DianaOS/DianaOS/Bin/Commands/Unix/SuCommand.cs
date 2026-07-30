using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // `su` - see `man su`. Switches THIS shell's own active account -
    // real `su` semantics for the "no argument" case (become root), but
    // deliberately NOT gated by DianaOSUserRegistry's stored password
    // hash (see that type's own header comment on why - no enforcement
    // exists anywhere yet, this is Unix flavor, not access control).
    public class SuCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;

        public SuCommand(Func<DianaOSInterpreter> self)
        {
            _self = self;
        }

        public string Name => "su";
        public bool IsReadOnly => false;
        public string Usage => "  su [name]                     switch this shell to <name> (default: root) - see `useradd`";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            string name = args.Length >= 2 ? string.Join(' ', args.Skip(1)) : "root";

            if (!DianaOSUserRegistry.Exists(name))
            {
                return DianaOSResult.Fail($"su: no such user '{name}' - see `useradd`.");
            }

            _self().CurrentUser = name;
            return $"Switched to '{name}'.";
        }
    }
}
