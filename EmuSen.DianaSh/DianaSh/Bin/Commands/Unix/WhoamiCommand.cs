using System;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // `whoami` - see `man whoami`.
    public class WhoamiCommand : IDianaShCommand
    {
        private readonly Func<DianaShInterpreter> _self;

        public WhoamiCommand(Func<DianaShInterpreter> self)
        {
            _self = self;
        }

        public string Name => "whoami";
        public bool IsReadOnly => true;
        public string Usage => "  whoami                        print the account this shell is currently running as";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) => _self().CurrentUser;
    }
}
