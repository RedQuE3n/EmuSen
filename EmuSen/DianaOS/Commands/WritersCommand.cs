using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // "Who writes this address" - scans a range of code for instructions
    // statically storing to <addr>, the store-side counterpart to
    // `callers`. Motivated directly by the Yoshi/coin/block investigation
    // hitting a wall: watching $13C6/$1FFE live showed exactly one write
    // apiece, both from the same one-time init PC, with no write path ever
    // reached during the triggering event itself - which only tells us
    // what the traced *run* did, not what code exists in the ROM that's
    // capable of writing there. `writers` answers that regardless of
    // whether the write path was ever actually reached.
    //
    // Fully core-agnostic: reuses IDebugTarget.Disassemble AND
    // IDebugTarget.ClassifyStaticReference, so it has no idea what CPU or
    // opcode encoding it's scanning, or which addressing modes have a
    // statically-knowable target - see that method's own comment (and
    // SnesDebugTarget's implementation, for the SNES-specific
    // "absolute/absolute-long only, direct-page/indexed/indirect excluded"
    // reasoning that used to live directly in this file).
    public class WritersCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "writers";
        public string Usage => string.Join('\n', new[]
        {
            "  writers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically writing to <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: writers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Write);
        }
    }
}
