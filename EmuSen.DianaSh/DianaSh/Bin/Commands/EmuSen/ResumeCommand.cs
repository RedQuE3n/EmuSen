using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Mistress replaces this with its own pause-aware version - see `man resume` and §3.17a.
    public class ResumeCommand : IDianaShCommand
    {
        public string Name => "resume";
        public bool IsReadOnly => false;
        public string Usage => "  resume | continue | c         resume emulation after an interactive halt (F4/breakpoint/single-step)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaShResult.Ok("Resuming.", new HostAction.Resume());
    }
}
