using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // "Did this code ever run" (see CoverageRegistry.cs) - the whole-run
    // counterpart to `bp`, which can only answer "is it here right now". Pairs
    // with `callers`: walk up from a routine that never ran until a caller
    // that did, and the branch between them is the one that skipped it. See
    // EmuSen_Debugging_Tools_Reference_v5.md §3.24.
    public class CovCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "cov";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  cov on|off                    start/stop recording executed addresses",
            "  cov clear                     forget everything recorded so far",
            "  cov <addr> [<len>]            report which of <len> bytes from <addr> ever executed",
            "                                (default 16); 0 executed means the code never ran",
            "  cov cop on|off|clear|<addr>   same, on the coprocessor's own instruction stream -",
            "                                its addresses are a different address space",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: cov [cop] on|off|clear|<addr> [<len>]";

            // Optional scope word ahead of the subcommand, same shape as `bp sa1`.
            int at = 1;
            bool coprocessor = parts[at].Equals("cop", System.StringComparison.OrdinalIgnoreCase)
                            || parts[at].Equals("sa1", System.StringComparison.OrdinalIgnoreCase)
                            || parts[at].Equals("gsu", System.StringComparison.OrdinalIgnoreCase);
            if (coprocessor) at++;
            if (parts.Length <= at) return "Usage: cov cop on|off|clear|<addr> [<len>]";

            var coverage = coprocessor ? target.CoprocessorCoverage : target.Coverage;
            if (coverage == null)
            {
                return coprocessor
                    ? "This cartridge has no coprocessor to record coverage for."
                    : $"{target.CoreName} target records no execution coverage.";
            }

            string scope = coprocessor ? "Coprocessor coverage" : "Coverage";
            string sub = parts[at].ToLowerInvariant();

            switch (sub)
            {
                case "on":
                    coverage.Arm();
                    return $"{scope} recording on.";
                case "off":
                    coverage.Disarm();
                    return $"{scope} recording off, {coverage.InstructionsRecorded} instructions recorded.";
                case "clear":
                    coverage.Clear();
                    return $"{scope} cleared.";
            }

            int address = ParseHex(parts[at]);
            int length = parts.Length > at + 1 ? ParseHex(parts[at + 1]) : 16;
            if (length <= 0) return "Length must be positive.";

            int hits = coverage.CountExecuted(address, length);
            var lines = new System.Collections.Generic.List<string>
            {
                $"{scope} ${address:X6}-${address + length - 1:X6}: {hits}/{length} bytes executed"
                + (coverage.IsArmed ? string.Empty : " (recording is OFF)"),
            };

            if (hits == 0)
            {
                // The whole point of the command, so say it in words rather
                // than leaving a zero to be interpreted.
                lines.Add("  Never reached.");
            }
            else
            {
                var executed = coverage.ExecutedAddresses(address, length, 32);
                lines.Add("  " + string.Join(" ", executed.Select(a => $"${a:X6}")) + (hits > executed.Count ? " ..." : string.Empty));
            }

            return string.Join('\n', lines);
        }
    }
}
