namespace EmuSen.Debug.Commands
{
    // Unix `echo` - prints its arguments back, joined by single spaces.
    // Doesn't touch IDebugTarget at all, same as LogCommand/TraceCommand -
    // this is pure text plumbing, not a debugging tool. Its actual use is
    // scripting: a `--commands` file (EmuSen.HeadlessDebug) or a future
    // GUI console can drop an `echo` line in as a marker/separator without
    // needing a dedicated "print a comment" verb, and it's the natural
    // pipeline *source* for SedCommand ("echo <text> | sed s/.../.../").
    //
    // Note this project's command tokenizer (DebugCommandProcessor.Execute)
    // splits on whitespace with no quoting support - "echo  a   b" and
    // "echo a b" both come out as "a b", same collapsed-whitespace
    // behavior `printf '%s\n' $UNQUOTED_VAR` would give you in a shell
    // that isn't quoting either. Not worth adding quote parsing for.
    //
    // Implements ITextFilterCommand too so `<anything> | echo literal
    // text` doesn't error out just because echo happens to be mid-pipeline
    // - matches real `echo`, which ignores stdin entirely.
    public class EchoCommand : IDebugCommand, ITextFilterCommand
    {
        public string Name => "echo";
        public string Usage => "  echo <text...>                print <text> back, joined by spaces";

        public string Execute(IDebugTarget target, string[] parts) => Join(parts);

        public string Filter(IDebugTarget target, string input, string[] parts) => Join(parts);

        private static string Join(string[] parts) => parts.Length > 1 ? string.Join(' ', parts, 1, parts.Length - 1) : "";
    }
}
