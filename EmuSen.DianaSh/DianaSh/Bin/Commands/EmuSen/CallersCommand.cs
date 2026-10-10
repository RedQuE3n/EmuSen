using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // "Who calls this address", statically rather than by waiting - see EmuSen_Debugging_Tools_Reference_v5.md §3.12.
    public class CallersCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "callers";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  callers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically calling/jumping to <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: callers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            // Instructions are at least a byte, so scanLen instructions always covers scanLen bytes.
            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Call);
        }
    }
}
