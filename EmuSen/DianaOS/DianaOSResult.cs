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
    public readonly struct DianaOSResult
    {
        public readonly string Output;
        public readonly int ExitCode;

        public DianaOSResult(string output, int exitCode)
        {
            Output = output;
            ExitCode = exitCode;
        }

        public static implicit operator DianaOSResult(string output) => new DianaOSResult(output, 0);

        public static DianaOSResult Ok(string output) => new DianaOSResult(output, 0);
        public static DianaOSResult Fail(string output, int exitCode = 1) => new DianaOSResult(output, exitCode);
    }
}
