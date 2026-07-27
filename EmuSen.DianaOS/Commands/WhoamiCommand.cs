using System;

namespace EmuSen.DianaOS.Commands
{
    // `whoami` - see `man whoami`.
    public class WhoamiCommand : IDianaOSCommand
    {
        private readonly Func<DianaOSInterpreter> _self;

        public WhoamiCommand(Func<DianaOSInterpreter> self)
        {
            _self = self;
        }

        public string Name => "whoami";
        public bool IsReadOnly => true;
        public string Usage => "  whoami                        print the account this shell is currently running as";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => _self().CurrentUser;
    }
}
