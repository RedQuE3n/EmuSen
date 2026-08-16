using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // "Did this code ever run", whole-run, plus the ROM map built from it - see `man cov`.
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
            "  cov mark                      remember what has run so far, so `cov new` can subtract it",
            "  cov new <addr> [<len>]        only what has run SINCE the mark - do the thing, then ask",
            "  cov funcs [<count>]           routines discovered by watching calls land, busiest first",
            "  cov save|load <file>          persist the map to Logs/<CoreName>/<file> and merge it back",
            "  cov <cpu> on|off|clear|<addr> same, on another processor's own instruction stream -",
            "                                `cpus` lists them, and each has its own address space",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: cov [<cpu>] on|off|clear|mark|new|funcs|save|load|<addr> [<len>]";

            // Optional scope word ahead of the subcommand - see `man cpus`.
            var (cpu, at) = ResolveCpu(target, parts, 1);
            if (cpu == null && DebugCpus.IsScopeWord(target.DebugCpus, parts[1]))
            {
                return "This cartridge has no coprocessor to record coverage for.";
            }
            if (parts.Length <= at) return "Usage: cov <cpu> on|off|clear|<addr> [<len>]";

            bool coprocessor = cpu != null && cpu.Name != DebugCpus.MainName;
            var coverage = cpu != null ? cpu.Coverage : target.Coverage;
            if (coverage == null)
            {
                return coprocessor
                    ? $"{cpu!.Name} records no execution coverage on this core."
                    : $"{target.CoreName} target records no execution coverage.";
            }

            string scope = coprocessor ? $"{cpu!.Name.ToUpperInvariant()} coverage" : "Coverage";
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
                case "mark":
                    coverage.Mark();
                    return $"{scope} marked at {coverage.InstructionsRecorded} instructions - `cov new <addr>` now reports only what runs from here.";
                case "funcs":
                {
                    int limit = parts.Length > at + 1 ? ParseHex(parts[at + 1]) : 32;
                    var funcs = coverage.EntryPoints(limit);
                    if (funcs.Count == 0)
                    {
                        return coverage.IsArmed
                            ? "No routines discovered yet - nothing has been called since recording started."
                            : "No routines discovered (recording is OFF - `cov on` first).";
                    }
                    var labels = target.Labels;
                    var rows = funcs.Select(f =>
                        $"  {labels?.Describe(f.Address) ?? $"${f.Address:X6}"}  {f.Kind.ToString().ToLowerInvariant(),-4}  entered {f.Entries}x");
                    return $"{coverage.EntryPointCount} routine(s) discovered:\n" + string.Join('\n', rows);
                }
                case "save":
                case "load":
                {
                    if (parts.Length <= at + 1) return $"Usage: cov {sub} <file>";
                    string dir = Path.Combine(DianaOSSandbox.LogsDirectory, target.CoreName);
                    Directory.CreateDirectory(dir);
                    string path = Path.Combine(dir, parts[at + 1]);

                    if (sub == "save")
                    {
                        File.WriteAllBytes(path, coverage.Export());
                        return $"{scope} map saved to {path} ({coverage.EntryPointCount} routine(s)).";
                    }
                    if (!File.Exists(path)) return DianaOSResult.Fail($"No coverage map at {path}.");
                    if (!coverage.Import(File.ReadAllBytes(path)))
                        return DianaOSResult.Fail($"{path} is not a coverage map this build can read.");
                    return $"{scope} map merged from {path} ({coverage.EntryPointCount} routine(s) known now).";
                }
            }

            bool newOnly = sub == "new";
            if (newOnly)
            {
                if (!coverage.HasMark) return DianaOSResult.Fail("Nothing marked yet - run `cov mark`, do the thing, then ask again.");
                at++;
                if (parts.Length <= at) return "Usage: cov new <addr> [<len>]";
            }

            int address = ParseHex(parts[at]);
            int length = parts.Length > at + 1 ? ParseHex(parts[at + 1]) : 16;
            if (length <= 0) return "Length must be positive.";

            int hits = newOnly ? coverage.CountNewSinceMark(address, length) : coverage.CountExecuted(address, length);
            string what = newOnly ? "bytes executed since the mark" : "bytes executed";
            var lines = new System.Collections.Generic.List<string>
            {
                $"{scope} ${address:X6}-${address + length - 1:X6}: {hits}/{length} {what}"
                + (coverage.IsArmed ? string.Empty : " (recording is OFF)"),
            };

            if (hits == 0)
            {
                // The whole point of the command, so say it rather than leaving a zero to interpret.
                lines.Add(newOnly ? "  Nothing new here since the mark." : "  Never reached.");
            }
            else
            {
                var executed = newOnly
                    ? coverage.NewSinceMark(address, length, 32)
                    : coverage.ExecutedAddresses(address, length, 32);
                lines.Add("  " + string.Join(" ", executed.Select(a => $"${a:X6}")) + (hits > executed.Count ? " ..." : string.Empty));
            }

            return string.Join('\n', lines);
        }
    }
}
