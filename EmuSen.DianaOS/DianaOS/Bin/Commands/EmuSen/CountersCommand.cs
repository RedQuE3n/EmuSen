using System;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Text;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Per-address read/write/execute tallies for one space - see `man counters`.
    public class CountersCommand : IDianaOSCommand
    {
        public string Name => "counters";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  counters on <space>           start tallying reads/writes/executes for <space>",
            "  counters off                  stop tallying (kept counts stay readable)",
            "  counters clear                zero every count, stay armed",
            "  counters <addr> [<len>]       totals over a range, default length 1",
            "  counters top [r|w|x|u] [<n>]  hottest addresses, default reads, default 20",
            "  counters cold <addr> <len>    addresses in the range nothing ever touched",
        });

        public DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            IDebugTarget resolved;
            try { resolved = RequireTarget(target); }
            catch (Exception ex) { return DianaOSResult.Fail(ex.Message); }

            var counters = resolved.AccessCounters;
            if (counters == null) return DianaOSResult.Fail("This core does not report access counters.");
            if (parts.Length < 2) return "Usage: counters on|off|clear|top|cold|<addr> ...";

            switch (parts[1].ToLowerInvariant())
            {
                case "on":
                {
                    if (parts.Length < 3) return "Usage: counters on <space>";
                    var space = FindSpace(resolved, parts[2]);
                    counters.Arm(space.Name, space.Size);
                    return $"Counting reads/writes/executes for {space.Name} ({space.Size} bytes).";
                }
                case "off":
                    counters.Disarm();
                    return "Access counting stopped.";
                case "clear":
                    counters.Clear();
                    return "Access counts cleared.";
                case "top":
                {
                    if (!counters.IsArmed && counters.Size == 0) return "Not counting - run `counters on <space>` first.";
                    var sortBy = parts.Length > 2 ? ParseSort(parts[2]) : AccessCounterRegistry.SortBy.Reads;
                    if (sortBy == null) return DianaOSResult.Fail($"Unknown sort '{parts[2]}'. Try r/w/x/u.");
                    int limit = parts.Length > 3 ? ParseHex(parts[3]) : 20;
                    var rows = counters.Hottest(sortBy.Value, limit);
                    if (rows.Count == 0) return "Nothing has been accessed yet.";
                    return Header(counters) + "\n" + string.Join('\n', rows.Select(r =>
                        $"  {Describe(resolved, r.Address)}  r={r.Reads,-10} w={r.Writes,-10} x={r.Executes,-10} uninit={r.UninitializedReads}"));
                }
                case "cold":
                {
                    if (parts.Length < 4) return "Usage: counters cold <addr> <len>";
                    int address = ParseHex(parts[2]);
                    int length = ParseHex(parts[3]);
                    var cold = counters.Untouched(address, length, 256);
                    if (cold.Count == 0) return $"Every byte in {counters.Space} 0x{address:X}-0x{address + length - 1:X} was touched.";
                    return $"{cold.Count} untouched address(es) in {counters.Space} 0x{address:X}-0x{address + length - 1:X}:\n"
                        + string.Join('\n', cold.Select(a => $"  0x{a:X}"));
                }
                default:
                {
                    int address;
                    try { address = ParseHex(parts[1]); }
                    catch (Exception)
                    {
                        string[] subcommands = { "on", "off", "clear", "top", "cold" };
                        return $"Unknown 'counters' subcommand '{parts[1]}'.{Suggestion.Hint(parts[1], subcommands)} Try {string.Join('/', subcommands)} or an address.";
                    }

                    int length = parts.Length > 2 ? ParseHex(parts[2]) : 1;
                    var totals = counters.Totals(address, length);
                    return Header(counters) + "\n"
                        + $"  {counters.Space} 0x{address:X}-0x{address + length - 1:X}\n"
                        + $"  reads {totals.Reads}, writes {totals.Writes}, executes {totals.Executes}\n"
                        + $"  uninitialized reads {totals.UninitializedReads}, bytes touched {totals.TouchedBytes}/{length}";
                }
            }
        }

        private static string Header(AccessCounterRegistry counters)
            => $"{counters.Space}{(counters.IsArmed ? string.Empty : " (counting stopped)")}:";

        private static string Describe(IDebugTarget target, int address)
            => target.Labels?.Describe(address) ?? $"${address:X6}";

        private static AccessCounterRegistry.SortBy? ParseSort(string text) => text.ToLowerInvariant() switch
        {
            "r" or "read" or "reads" => AccessCounterRegistry.SortBy.Reads,
            "w" or "write" or "writes" => AccessCounterRegistry.SortBy.Writes,
            "x" or "exec" or "executes" => AccessCounterRegistry.SortBy.Executes,
            "u" or "uninit" or "uninitialized" => AccessCounterRegistry.SortBy.Uninitialized,
            _ => null,
        };
    }
}
