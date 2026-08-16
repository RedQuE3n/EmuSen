using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Mistress replaces this with its own pause-aware version - see `man resume` and §3.17a.
    public class ResumeCommand : IDianaOSCommand
    {
        public string Name => "resume";
        public bool IsReadOnly => false;
        public string Usage => "  resume | continue | c         resume emulation after an interactive halt (F4/breakpoint/single-step)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
            => DianaOSResult.Ok("Resuming.", new HostAction.Resume());
    }
}
