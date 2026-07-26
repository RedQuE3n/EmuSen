using EmuSen.Debug;

namespace EmuSen.Shell.Commands
{
    // Unix `echo` - prints its arguments back, joined by single spaces.
    // Doesn't touch IDebugTarget at all - pure text plumbing, not a
    // debugging tool, which is why it lives here (EmuSen.Shell.Commands)
    // rather than alongside mem/regs/watch/etc. in EmuSen.Debug.Commands.
    // Its real use is scripting: a marker/separator line in a
    // `--commands` file, or the natural pipeline *source* for `sed`
    // ("echo <text> | sed s/.../.../").
    //
    // Real argument quoting (via the shell's own Lexer) is supported now -
    // "echo 'a b'" is one argument, not two - this is just the trivial
    // "print my own already-tokenized/expanded argument list" part.
    //
    // Ignores stdin entirely (matching real `echo`, which does too) so
    // `<anything> | echo literal text` doesn't error out just because
    // echo happens to be mid-pipeline.
    public class EchoCommand : IShellCommand
    {
        public string Name => "echo";
        public string Usage => "  echo <text...>                print <text> back, joined by spaces";

        public ShellResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            args.Length > 1 ? string.Join(' ', args, 1, args.Length - 1) : "";
    }
}
