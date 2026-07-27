
namespace EmuSen.DianaOS.Commands.Unix
{
    // Unix `echo` - prints its arguments back, joined by single spaces.
    // Doesn't touch IDebugTarget at all - pure text plumbing, not a
    // debugging tool, unlike most of its neighbors here (mem/regs/watch/
    // etc.) - EmuSen.Debug.Commands and EmuSen.DianaOS.Commands used to be
    // two separate namespaces for exactly that distinction, merged into
    // one (EmuSen.DianaOS.Commands) since every one of them is a shell
    // command regardless of whether it happens to touch a debug target.
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
    public class EchoCommand : IDianaOSCommand
    {
        public string Name => "echo";
        public bool IsReadOnly => true;
        public string Usage => "  echo <text...>                print <text> back, joined by spaces";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin) =>
            args.Length > 1 ? string.Join(' ', args, 1, args.Length - 1) : "";
    }
}
