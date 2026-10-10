using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Pure text plumbing, and the natural source of a pipeline - see §3.17.
    public class EchoCommand : IDianaShCommand
    {
        public string Name => "echo";
        public bool IsReadOnly => true;
        public string Usage => "  echo <text...>                print <text> back, joined by spaces";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            args.Length > 1 ? string.Join(' ', args, 1, args.Length - 1) : "";
    }
}
