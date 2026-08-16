using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Finds write paths a live watch never saw executed - see §3.12.
    public class WritersCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "writers";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  writers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically writing to <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: writers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Write);
        }
    }
}
