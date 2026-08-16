using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
namespace EmuSen.DianaOS.DianaOS.Lib
{
    // Implicitly built from a string so ordinary commands stay unchanged; Action is for §3.3b.
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
