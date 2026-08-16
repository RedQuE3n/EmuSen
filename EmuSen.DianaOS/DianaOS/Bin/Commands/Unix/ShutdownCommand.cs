using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Promoted from a hand-rolled string match; 'quit' is an alias, not an entry - see §3.17a.
    public class ShutdownCommand : IDianaOSCommand
    {
        public string Name => "shutdown";
        public bool IsReadOnly => false;
        public string Usage => "  shutdown | quit               terminate the process";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaOSResult.Ok("Shutting down.", new HostAction.Shutdown());
    }
}
