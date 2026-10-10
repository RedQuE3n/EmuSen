using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
namespace EmuSen.DianaSh.DianaSh.Lib
{
    // Implicitly built from a string so ordinary commands stay unchanged; Action is for §3.3b.
    public readonly struct DianaShResult
    {
        public readonly string Output;
        public readonly int ExitCode;
        public readonly HostAction? Action;

        public DianaShResult(string output, int exitCode, HostAction? action = null)
        {
            Output = output;
            ExitCode = exitCode;
            Action = action;
        }

        public static implicit operator DianaShResult(string output) => new DianaShResult(output, 0);

        public static DianaShResult Ok(string output, HostAction? action = null) => new DianaShResult(output, 0, action);
        public static DianaShResult Fail(string output, int exitCode = 1) => new DianaShResult(output, exitCode);
    }
}
