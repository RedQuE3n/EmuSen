
namespace EmuSen.Shell.Commands
{
    // Unix `true` - does nothing, always succeeds (exit code 0). Its only
    // real use is in control flow: `while true; do ...; done`, or as a
    // harmless placeholder branch.
    public class TrueCommand : IShellCommand
    {
        public string Name => "true";
        public string Usage => "  true                          do nothing, succeed (exit code 0) - for while/if conditions";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin) => ShellResult.Ok("");
    }
}
