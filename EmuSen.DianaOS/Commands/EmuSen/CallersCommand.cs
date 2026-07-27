using static EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    // "Who calls this address" - scans a range of code for instructions
    // statically calling/jumping to <addr>, the static-analysis counterpart
    // to setting a write watch and waiting for something to happen: instead
    // of observing execution live, this reads the ROM/RAM code itself to
    // find every place that *could* jump there. Built the same way the
    // Yoshi investigation manually mined an already-captured CPU trace for
    // JSR/JSL targeting a known range (see EmuSen_Core_Gameplan.md's
    // current-status section) - generalizing that one-off grep into a real
    // command.
    //
    // Fully core-agnostic: reuses IDebugTarget.Disassemble AND
    // IDebugTarget.ClassifyStaticReference, so it has no idea what CPU or
    // opcode encoding it's scanning - see that method's own comment for
    // where the actual "which instructions are calls with a statically-
    // known target" knowledge lives (in each core's own implementation,
    // e.g. SnesDebugTarget's 65816 opcode table).
    public class CallersCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "callers";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  callers <addr> [<scanstart> <scanlen>]",
            "                                find instructions statically calling/jumping to <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: callers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            // count = scanLen instructions is deliberately generous - every
            // instruction on any real ISA is at least 1 byte, so scanLen
            // instructions always covers at least scanLen bytes; the shared
            // scan helper's address check stops once the scanned range is
            // actually exhausted. Same "best-effort, may misalign through
            // embedded data" caveat as `disasm` applies here too - a linear
            // disassembler has no way to know which bytes are really code
            // vs. data mixed into the same range.
            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, StaticReferenceKind.Call);
        }
    }
}
