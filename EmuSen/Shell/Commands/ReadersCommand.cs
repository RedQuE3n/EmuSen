using static EmuSen.Shell.Commands.DebugCommandHelpers;

namespace EmuSen.Shell.Commands
{
    // "Who reads this address" - scans a range of code for LDA/LDX/LDY
    // instructions whose target matches <addr>, the read-side complement to
    // `writers`. Where `writers` finds every place a flag like $13C6 could
    // get set, `readers` finds every place that could be checking it - e.g.
    // every branch of gameplay logic gated on that flag's value, not just
    // the one we happened to spot manually while tracing the NMI handler.
    //
    // Same "only statically-knowable targets" rule as WritersCommand, for
    // the same reason - see that file's comment for the full rationale:
    //   - Absolute (LDA/LDX/LDY $nnnn) - bank assumed to equal the
    //     instruction's own bank (DBR-as-PB convention).
    //   - Absolute long (LDA $nnnnnn) - exact 24-bit target. Note only LDA
    //     has a long form on 65816; LDX/LDY don't, so they only ever
    //     contribute the absolute case.
    // Direct-page, indexed, indirect, stack-relative, and immediate forms
    // are excluded for the same reasons as in `writers` (immediate mode
    // additionally isn't a memory read at all - the operand IS the value).
    //
    // Core-agnostic like CallersCommand/WritersCommand: works purely off
    // IDebugTarget.Disassemble.
    public class ReadersCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "readers";
        public string Usage => string.Join('\n', new[]
        {
            "  readers <addr> [<scanstart> <scanlen>]",
            "                                find LDA/LDX/LDY instructions targeting <addr>",
            "                                (absolute/absolute-long only - see source comment",
            "                                for why direct-page/indexed/indirect/immediate are",
            "                                excluded), scanning CpuBus (default: <addr>'s own",
            "                                bank, $8000-$FFFF)",
        });

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: readers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, "LDA/LDX/LDY", (opcode, instr) => opcode switch
            {
                0xAD => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // LDA absolute
                0xAF => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // LDA absolute long
                0xAE => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // LDX absolute
                0xAC => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // LDY absolute
                _ => null
            }, notFoundSuffix: " (absolute/absolute-long only)");
        }
    }
}
