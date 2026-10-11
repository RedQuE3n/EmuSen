using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Always fails, for the other branch of `true`'s control-flow use.
    public class FalseCommand : IDianaShCommand
    {
        public string Name => "false";
        public bool IsReadOnly => true;
        public string Usage => "  false                         do nothing, fail (exit code 1) - for while/if conditions";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaShResult.Fail("");
    }
}
