using System;
using System.Text;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // The traffic between the CPU and a coprocessor's register window - see `man copflow`.
    public class CopFlowCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "copflow";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  copflow on [<size>]           start logging coprocessor register-window traffic",
            "  copflow off                   stop logging",
            "  copflow clear                 forget everything logged so far",
            "  copflow tail [<n>]            the last <n> accesses in order (default 20)",
            "  copflow stats                 per-register read/write tallies, busiest first",
            "  copflow poll                  the longest run of unchanging reads - a stuck handshake",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = RequireTarget(target);
            if (parts.Length < 2) return "Usage: copflow on|off|clear|tail|stats|poll";

            if (target.RegisterFlow is not { } flow)
            {
                return $"{target.CoreName} exposes no coprocessor register window to watch.";
            }

            switch (parts[1].ToLowerInvariant())
            {
                case "on":
                {
                    int size = parts.Length > 2 ? ParseHex(parts[2]) : 0x1000;
                    flow.Arm(flow.WindowName.Length > 0 ? flow.WindowName : "CoprocessorRegisters", size);
                    return $"Coprocessor register flow logging on ({size} registers, {flow.Capacity} retained).";
                }
                case "off":
                    flow.Disarm();
                    return "Coprocessor register flow logging off.";
                case "clear":
                    flow.Clear();
                    return "Coprocessor register flow log cleared.";
                case "tail":
                    return Tail(flow, parts.Length > 2 ? ParseHex(parts[2]) : 20);
                case "stats":
                    return Stats(flow);
                case "poll":
                    return Poll(flow);
                default:
                    return $"Unknown copflow subcommand '{parts[1]}'. Try on|off|clear|tail|stats|poll.";
            }
        }

        private static string Tail(RegisterFlowRegistry flow, int count)
        {
            var entries = flow.Tail(Math.Max(1, count));
            if (entries.Count == 0) return NothingLogged(flow);

            var sb = new StringBuilder();
            sb.AppendLine($"Last {entries.Count} of {flow.TotalAccesses} access(es):");
            sb.AppendLine("       seq  frame  dir  register  value  by");
            foreach (var e in entries)
            {
                sb.AppendLine($"  {e.Sequence,8}  {e.FrameNumber,5}  {(e.IsWrite ? "W " : " R"),3}  "
                    + $"${e.Register:X4}     0x{e.Value:X2}   {e.Actor}");
            }
            return sb.ToString().TrimEnd();
        }

        private static string Stats(RegisterFlowRegistry flow)
        {
            var stats = flow.Stats();
            if (stats.Count == 0) return NothingLogged(flow);

            var sb = new StringBuilder();
            sb.AppendLine($"{stats.Count} register(s) touched, {flow.TotalAccesses} access(es) total:");
            sb.AppendLine("  register    reads   writes  distinct  last");
            foreach (var s in stats)
            {
                sb.AppendLine($"  ${s.Register:X4}  {s.Reads,9}  {s.Writes,7}  {s.DistinctValues,8}  0x{s.LastValue:X2}");
            }
            return sb.ToString().TrimEnd();
        }

        // The "it wrote the parameters and then waited forever" report.
        private static string Poll(RegisterFlowRegistry flow)
        {
            if (flow.TotalAccesses == 0) return NothingLogged(flow);

            var sb = new StringBuilder();
            if (flow.LongestPollRegister < 0)
            {
                sb.AppendLine("No unchanging read run recorded - every read saw a value change or was followed by a write.");
            }
            else
            {
                sb.AppendLine($"Longest unchanging read run: ${flow.LongestPollRegister:X4} read "
                    + $"{flow.LongestPollRun} time(s) in a row, always 0x{flow.LongestPollValue:X2}.");
            }

            if (flow.CurrentPollRegister >= 0 && flow.CurrentPollRun > 1)
            {
                sb.AppendLine($"Right now: ${flow.CurrentPollRegister:X4} has read the same value "
                    + $"{flow.CurrentPollRun} time(s) with no intervening write.");
            }

            sb.Append("A long run means the CPU is spinning on a status bit the chip never changed.");
            return sb.ToString();
        }

        private static string NothingLogged(RegisterFlowRegistry flow)
            => flow.IsArmed
                ? "Nothing logged yet - the cartridge has not touched its coprocessor registers since `copflow on`."
                : "Coprocessor register flow logging is off. Turn it on with `copflow on`.";
    }
}
