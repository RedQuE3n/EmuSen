
namespace EmuSen.DianaOS.Commands
{
    // Unix `true` - does nothing, always succeeds (exit code 0). Its only
    // real use is in control flow: `while true; do ...; done`, or as a
    // harmless placeholder branch.
    public class TrueCommand : IDianaOSCommand
    {
        public string Name => "true";
        public string Usage => "  true                          do nothing, succeed (exit code 0) - for while/if conditions";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaOSResult.Ok("");
    }
}
