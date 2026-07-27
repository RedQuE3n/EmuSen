
namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `false` - does nothing, always fails (exit code 1). Real use is
    // the same as `true`'s, just the other branch (`until false; do ...
    // done` == an infinite loop, same as `while true`).
    public class FalseCommand : IDianaOSCommand
    {
        public string Name => "false";
        public bool IsReadOnly => true;
        public string Usage => "  false                         do nothing, fail (exit code 1) - for while/if conditions";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) => DianaOSResult.Fail("");
    }
}
