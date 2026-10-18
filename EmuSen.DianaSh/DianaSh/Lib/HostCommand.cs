using EmuSen.Crystal.Debugger;

namespace EmuSen.DianaSh.DianaSh.Lib
{
    // A host's own command that needs no debug target, so a frontend on the frontend API writes one without naming a runtime type - see EmuSen_Backend.md §14.11.
    public abstract class HostCommand : IDianaShCommand
    {
        public abstract string Name { get; }
        public abstract bool IsReadOnly { get; }
        public abstract string Usage { get; }

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) => Run(args, stdin);

        protected abstract DianaShResult Run(string[] args, string? stdin);
    }
}
