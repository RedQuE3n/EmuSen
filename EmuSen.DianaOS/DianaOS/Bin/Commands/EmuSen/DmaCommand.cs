using System;
using System.Text;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Block-transfer channel state, plus the transfer history behind it - see `man dma`.
    public class DmaCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "dma";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  dma                           every channel's current configuration",
            "  dma log on [<n>] | dma log off",
            "                                start/stop recording transfers (<n> retained, default 4096)",
            "  dma log [<n>] | dma log clear",
            "                                the last <n> transfers in order (default 20), or forget them",
            "  dma stats                     per-channel and per-destination totals, biggest first",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = RequireTarget(target);

            if (parts.Length < 2) return Channels(target);

            switch (parts[1].ToLowerInvariant())
            {
                case "log":
                    return Log(target, parts);
                case "stats":
                    return Stats(target);
                default:
                    return $"Unknown dma subcommand '{parts[1]}'. Try log|stats, or `dma` alone for the channel table.";
            }
        }

        private static DianaOSResult Channels(IDebugTarget target)
        {
            var channels = target.DmaChannels;
            if (channels.Count == 0) return DianaOSResult.Fail($"{target.CoreName} publishes no DMA channels.");

            var sb = new StringBuilder();
            sb.AppendLine("  ch  hdma  dir   mode  destination       source     length  hdma table");
            foreach (var c in channels)
            {
                string destination = Destination(target, c.DestinationRegister);
                // An idle channel still reports its last configuration, which is usually what you want.
                string hdma = c.HdmaEnabled ? (c.HdmaActive ? "on " : "arm") : "-  ";
                string table = c.HdmaEnabled
                    ? $"${c.TableAddress:X6} line {c.LineCounter:X2}" + ((c.Control & 0x40) != 0 ? $" -> ${c.IndirectAddress:X6}" : string.Empty)
                    : "-";
                sb.AppendLine($"  {c.Index}   {hdma}   {((c.Control & 0x80) != 0 ? "B->A" : "A->B")}  {c.Control & 0x07,4}  "
                    + $"{destination,-16}  ${c.SourceAddress:X6}  {c.TransferSize,7}  {table}");
            }
            sb.Append("  (mode is the B-bus write pattern; dir B->A reads the register instead)");
            return sb.ToString();
        }

        private static DianaOSResult Log(IDebugTarget target, string[] parts)
        {
            if (target.DmaLog is not { } log) return DianaOSResult.Fail($"{target.CoreName} records no DMA history.");

            string arg = parts.Length > 2 ? parts[2].ToLowerInvariant() : string.Empty;
            switch (arg)
            {
                case "on":
                {
                    int capacity = parts.Length > 3 ? ParseHex(parts[3]) : 4096;
                    log.Arm(capacity);
                    return $"DMA transfer logging on ({log.Capacity} retained).";
                }
                case "off":
                    log.Disarm();
                    return "DMA transfer logging off.";
                case "clear":
                    log.Clear();
                    return "DMA transfer log cleared.";
            }

            int count = arg.Length == 0 ? 20 : ParseHex(arg);
            var entries = log.Tail(Math.Max(1, count));
            if (entries.Count == 0) return NothingLogged(log);

            var sb = new StringBuilder();
            sb.AppendLine($"Last {entries.Count} of {log.TotalTransfers} transfer(s):");
            sb.AppendLine("       seq  frame  line  ch  kind  destination       source     bytes");
            foreach (var e in entries)
            {
                sb.AppendLine($"  {e.Sequence,8}  {e.FrameNumber,5}  {e.Scanline,4}  {e.Channel}   "
                    + $"{(e.Kind == DmaTransferKind.Hdma ? "hdma" : "gen "),4}  "
                    + $"{Destination(target, e.DestinationRegister),-16}  ${e.SourceAddress:X6}  {e.Length,6}");
            }
            return sb.ToString().TrimEnd();
        }

        private static DianaOSResult Stats(IDebugTarget target)
        {
            if (target.DmaLog is not { } log) return DianaOSResult.Fail($"{target.CoreName} records no DMA history.");

            var channels = log.ChannelStats();
            if (channels.Count == 0) return NothingLogged(log);

            var sb = new StringBuilder();
            long frames = log.LastFrame - log.FirstFrame + 1;
            sb.AppendLine($"{log.TotalTransfers} transfer(s), {log.TotalBytes} byte(s) over {frames} frame(s)"
                + (frames > 0 ? $" - {log.TotalBytes / frames} bytes/frame." : "."));

            sb.AppendLine("  ch   general    bytes       hdma    bytes");
            foreach (var c in channels)
            {
                sb.AppendLine($"  {c.Channel}  {c.GeneralTransfers,8}  {c.GeneralBytes,7}  {c.HdmaTransfers,9}  {c.HdmaBytes,7}");
            }

            // Where the bandwidth actually went, which is the question a channel table cannot answer.
            sb.AppendLine("  destination       transfers    bytes  channels");
            foreach (var d in log.DestinationStats())
            {
                sb.AppendLine($"  {Destination(target, d.DestinationRegister),-16}  {d.Transfers,9}  {d.Bytes,7}  {ChannelList(d.ChannelMask)}");
            }
            return sb.ToString().TrimEnd();
        }

        // The core names its own bus, since only it knows what base the register byte offsets.
        private static string Destination(IDebugTarget target, byte register)
            => target.NameDmaDestination(register) ?? $"reg 0x{register:X2}";

        private static string ChannelList(int mask)
        {
            var sb = new StringBuilder();
            for (int i = 0; i < 16; i++)
            {
                if ((mask & (1 << i)) != 0) sb.Append(sb.Length == 0 ? $"{i}" : $",{i}");
            }
            return sb.Length == 0 ? "-" : sb.ToString();
        }

        private static string NothingLogged(DmaLogRegistry log)
            => log.IsArmed
                ? "Nothing logged yet - no transfer has run since `dma log on`."
                : "DMA transfer logging is off. Turn it on with `dma log on`.";
    }
}
