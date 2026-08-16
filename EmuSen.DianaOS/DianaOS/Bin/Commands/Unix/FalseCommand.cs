using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Always fails, for the other branch of `true`'s control-flow use.
    public class FalseCommand : IDianaOSCommand
    {
        public string Name => "false";
        public bool IsReadOnly => true;
        public string Usage => "  false                         do nothing, fail (exit code 1) - for while/if conditions";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaOSResult.Fail("");
    }
}
