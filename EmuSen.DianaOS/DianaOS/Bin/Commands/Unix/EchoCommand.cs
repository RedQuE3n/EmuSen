using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Pure text plumbing, and the natural source of a pipeline - see §3.17.
    public class EchoCommand : IDianaOSCommand
    {
        public string Name => "echo";
        public bool IsReadOnly => true;
        public string Usage => "  echo <text...>                print <text> back, joined by spaces";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            args.Length > 1 ? string.Join(' ', args, 1, args.Length - 1) : "";
    }
}
