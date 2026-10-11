using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // The read-side complement to `writers` - see §3.12.
    public class ReadersCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "readers";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  readers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically reading <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: readers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Read);
        }
    }
}
