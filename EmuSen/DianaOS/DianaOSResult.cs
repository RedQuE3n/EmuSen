namespace EmuSen.DianaOS
{
    // A command's result: the text it produced, plus a bash-style exit
    // code (0 = success, nonzero = failure) driving '&&'/'||'/'if'/'while'
    // short-circuiting. Implicitly constructible from a plain string
    // (exit code 0) specifically so every pre-existing debug command -
    // which just does `return "some text";` - keeps compiling unchanged
    // after switching from IDebugCommand's `string Execute(...)` to
    // IDianaOSCommand's `DianaOSResult Execute(...)`: success is still the
    // overwhelmingly common case, and only the handful of commands that
    // actually need to signal failure without throwing (true/false/test)
    // construct one explicitly.
    //
    // Action is a third, optional member for the same reason - null for
    // every ordinary command, exactly like the two-arg constructor and
    // implicit string conversion already produce. Only a command that
    // genuinely needs to change the CALLER's control flow (see
    // HostAction's own comment) ever sets it.
    public readonly struct DianaOSResult
    {
        public readonly string Output;
        public readonly int ExitCode;
        public readonly HostAction? Action;

        public DianaOSResult(string output, int exitCode, HostAction? action = null)
        {
            Output = output;
            ExitCode = exitCode;
            Action = action;
        }

        public static implicit operator DianaOSResult(string output) => new DianaOSResult(output, 0);

        public static DianaOSResult Ok(string output, HostAction? action = null) => new DianaOSResult(output, 0, action);
        public static DianaOSResult Fail(string output, int exitCode = 1) => new DianaOSResult(output, exitCode);
    }
}
