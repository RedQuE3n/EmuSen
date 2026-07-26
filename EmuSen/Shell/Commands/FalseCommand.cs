
namespace EmuSen.Shell.Commands
{
    // Unix `false` - does nothing, always fails (exit code 1). Real use is
    // the same as `true`'s, just the other branch (`until false; do ...
    // done` == an infinite loop, same as `while true`).
    public class FalseCommand : IShellCommand
    {
        public string Name => "false";
        public string Usage => "  false                         do nothing, fail (exit code 1) - for while/if conditions";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin) => ShellResult.Fail("");
    }
}
