using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // The read-side complement to `writers` - see §3.12.
    public class ReadersCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "readers";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  readers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically reading <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: readers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Read);
        }
    }
}
