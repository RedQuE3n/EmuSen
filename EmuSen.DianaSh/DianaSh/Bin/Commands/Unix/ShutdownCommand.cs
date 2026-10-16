using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Promoted from a hand-rolled string match; 'quit' is an alias, not an entry - see §3.17a.
    public class ShutdownCommand : IDianaShCommand
    {
        public string Name => "shutdown";
        public bool IsReadOnly => false;
        public string Usage => "  shutdown | quit | exit        terminate the process";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaShResult.Ok("Shutting down.", new HostAction.Shutdown());
    }
}
