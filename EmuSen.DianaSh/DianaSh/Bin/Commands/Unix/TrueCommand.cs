using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Always succeeds; its use is `while true` and placeholder branches.
    public class TrueCommand : IDianaShCommand
    {
        public string Name => "true";
        public bool IsReadOnly => true;
        public string Usage => "  true                          do nothing, succeed (exit code 0) - for while/if conditions";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaShResult.Ok("");
    }
}
