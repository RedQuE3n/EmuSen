using static EmuSen.Shell.Commands.DebugCommandHelpers;

namespace EmuSen.Shell.Commands
{
    // "Who calls this address" - scans a range of code for JSR/JSL/JMP/JML
    // instructions whose target matches <addr>, the static-analysis
    // counterpart to setting a write watch and waiting for something to
    // happen: instead of observing execution live, this reads the ROM/RAM
    // code itself to find every place that *could* jump there. Built the
    // same way the Yoshi investigation manually mined an already-captured
    // CPU trace for JSR/JSL targeting a known range (see
    // EmuSen_Core_Gameplan.md's current-status section) - generalizing
    // that one-off grep into a real command.
    //
    // Reuses IDebugTarget.Disassemble rather than re-decoding opcodes
    // itself - same disassembler, same bytes, so results always agree
    // with what `disasm` would show for the same range. Matches on the
    // raw opcode byte (Bytes[0]) rather than mnemonic text, since
    // mnemonic+length alone can't distinguish JMP $nnnn (absolute, direct)
    // from JMP ($nnnn) (indirect) or JMP ($nnnn,X) (indexed indirect) -
    // all three decode to the same "JMP" mnemonic and the same 3-byte
    // length, but only the first has a statically-known target. Indirect
    // forms are deliberately excluded rather than guessed at.
    public class CallersCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "callers";
        public string Usage => string.Join('\n', new[]
        {
            "  callers <addr> [<scanstart> <scanlen>]",
            "                                find JSR/JSL/JMP/JML instructions targeting <addr>,",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: callers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);
            var (scanStart, scanLen) = DefaultScanRange(parts, 2, targetAddr);

            // count = scanLen instructions is deliberately generous - every
            // 65816 instruction is at least 1 byte, so scanLen instructions
            // always covers at least scanLen bytes; the shared scan helper's
            // address check stops once the scanned range is actually
            // exhausted. Same "best-effort, may misalign through embedded
            // data" caveat as `disasm` applies here too - a linear
            // disassembler has no way to know which bytes are really code
            // vs. data mixed into the same range.
            return ScanForStaticReferences(target, scanStart, scanLen, targetAddr, "JSR/JSL/JMP/JML", (opcode, instr) => opcode switch
            {
                0x20 => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // JSR absolute
                0x22 => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // JSL absolute long
                0x4C => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // JMP absolute
                0x5C => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // JMP absolute long
                _ => null
            });
        }
    }
}
