using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Always succeeds; its use is `while true` and placeholder branches.
    public class TrueCommand : IDianaOSCommand
    {
        public string Name => "true";
        public bool IsReadOnly => true;
        public string Usage => "  true                          do nothing, succeed (exit code 0) - for while/if conditions";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaOSResult.Ok("");
    }
}
