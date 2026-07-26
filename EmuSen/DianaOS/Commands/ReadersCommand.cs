using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // "Who reads this address" - scans a range of code for instructions
    // statically reading <addr>, the read-side complement to `writers`.
    // Where `writers` finds every place a flag like $13C6 could get set,
    // `readers` finds every place that could be checking it - e.g. every
    // branch of gameplay logic gated on that flag's value, not just the one
    // we happened to spot manually while tracing the NMI handler.
    //
    // Fully core-agnostic: reuses IDebugTarget.Disassemble AND
    // IDebugTarget.ClassifyStaticReference, so it has no idea what CPU or
    // opcode encoding it's scanning, or which addressing modes have a
    // statically-knowable target - see that method's own comment (and
    // SnesDebugTarget's implementation, for the SNES-specific
    // "absolute/absolute-long only" reasoning that used to live directly in
    // this file).
    public class ReadersCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "readers";
        public string Usage => string.Join('\n', new[]
        {
            "  readers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically reading <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: readers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Read);
        }
    }
}
