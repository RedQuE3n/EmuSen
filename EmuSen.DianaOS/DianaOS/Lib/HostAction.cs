using EmuSen.Cauldron;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // For changing the caller's control flow; a side effect wants a delegate instead - see EmuSen_Debugging_Tools_Reference_v5.md §3.3b.
    public abstract record HostAction
    {
        private HostAction() { }

        // 'shutdown'/'quit' - terminate the whole process.
        public sealed record Shutdown : HostAction;

        // 'resume'/'continue'/'c' - stop blocking and let the emulation loop run.
        public sealed record Resume : HostAction;

        // 'step'/'s' - ArmSingleStep already armed it; this carries the "run again" half.
        public sealed record Step : HostAction;

        // Raw strings, because validating them was the issuing command's job - see §3.3b.
        public sealed record LoadCore(string CoreName, string RomPath) : HostAction;

        // 'tmux new/switch/kill' - see `man tmux`.
        public sealed record SwitchSession(string Name) : HostAction;
    }
}
