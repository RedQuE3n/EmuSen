using System.Collections.Generic;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
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
            target = EmuSen.Debug.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: callers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);

            int scanStart, scanLen;
            if (parts.Length >= 4)
            {
                scanStart = ParseHex(parts[2]);
                scanLen = ParseHex(parts[3]);
            }
            else
            {
                // Default to the target's own bank's upper 32KB - the
                // conventional LoROM code region (see MemoryBus's own
                // "offset >= 0x8000 is ROM" comment) and a reasonable
                // "just show me this bank's callers" starting point
                // without needing to already know a scan range.
                scanStart = (targetAddr & 0xFF0000) | 0x8000;
                scanLen = 0x8000;
            }

            // count = scanLen instructions is deliberately generous - every
            // 65816 instruction is at least 1 byte, so scanLen instructions
            // always covers at least scanLen bytes; the address check below
            // stops once the scanned range is actually exhausted. Same
            // "best-effort, may misalign through embedded data" caveat as
            // `disasm` applies here too - a linear disassembler has no way
            // to know which bytes are really code vs. data mixed into the
            // same range.
            var instrs = target.Disassemble("CpuBus", scanStart, scanLen);

            var matches = new List<string>();
            foreach (var instr in instrs)
            {
                if (instr.Address >= scanStart + scanLen) break;
                byte opcode = instr.Bytes[0];

                int? callTarget = opcode switch
                {
                    0x20 => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // JSR absolute
                    0x22 => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // JSL absolute long
                    0x4C => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // JMP absolute
                    0x5C => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // JMP absolute long
                    _ => null
                };

                if (callTarget.HasValue && callTarget.Value == targetAddr)
                {
                    matches.Add($"  ${instr.Address:X6}: {instr.Mnemonic} {instr.OperandText}");
                }
            }

            if (matches.Count == 0)
            {
                return $"No JSR/JSL/JMP/JML found targeting ${targetAddr:X6} in ${scanStart:X6}-${scanStart + scanLen - 1:X6}.";
            }
            return string.Join('\n', matches);
        }
    }
}
