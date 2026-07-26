namespace EmuSen.Shell
{
    // A command's result: the text it produced, plus a bash-style exit
    // code (0 = success, nonzero = failure) driving '&&'/'||'/'if'/'while'
    // short-circuiting. Implicitly constructible from a plain string
    // (exit code 0) specifically so every pre-existing debug command -
    // which just does `return "some text";` - keeps compiling unchanged
    // after switching from IDebugCommand's `string Execute(...)` to
    // IShellCommand's `ShellResult Execute(...)`: success is still the
    // overwhelmingly common case, and only the handful of commands that
    // actually need to signal failure without throwing (true/false/test)
    // construct one explicitly.
    public readonly struct ShellResult
    {
        public readonly string Output;
        public readonly int ExitCode;

        public ShellResult(string output, int exitCode)
        {
            Output = output;
            ExitCode = exitCode;
        }

        public static implicit operator ShellResult(string output) => new ShellResult(output, 0);

        public static ShellResult Ok(string output) => new ShellResult(output, 0);
        public static ShellResult Fail(string output, int exitCode = 1) => new ShellResult(output, exitCode);
    }
}
