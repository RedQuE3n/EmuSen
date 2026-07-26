using System.Collections.Generic;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    // "Who writes this address" - scans a range of code for STA/STX/STY/STZ
    // instructions whose target matches <addr>, the store-side counterpart
    // to `callers`. Motivated directly by the Yoshi/coin/block investigation
    // hitting a wall: watching $13C6/$1FFE live showed exactly one write
    // apiece, both from the same one-time init PC, with no write path ever
    // reached during the triggering event itself - which only tells us
    // what the traced *run* did, not what code exists in the ROM that's
    // capable of writing there. `writers` answers that regardless of
    // whether the write path was ever actually reached.
    //
    // Deliberately only matches addressing modes whose target is knowable
    // from the instruction bytes alone:
    //   - Absolute (STA/STX/STY/STZ $nnnn) - bank assumed to equal the
    //     instruction's own bank, same DBR-as-PB convention `callers` already
    //     uses for JSR/JMP absolute. Not always true (DBR can differ from PB
    //     at runtime) but it's the same best-effort assumption, not a new one.
    //   - Absolute long (STA $nnnnnn) - exact 24-bit target, no assumption
    //     needed at all.
    // Everything else is excluded, not guessed at:
    //   - Direct page (STA/STX/STY/STZ <dp>) - target depends on the D
    //     register's runtime value, which this static scan has no way to
    //     know (unlike DBR-as-PB, there's no reasonable default for D).
    //   - Indexed forms (,X / ,Y / long,X) - target depends on the runtime
    //     value of the index register, not just the instruction bytes.
    //   - Indirect and stack-relative forms - target depends on a runtime
    //     pointer or the S register.
    // Same principle CallersCommand already applies to indirect JMP: if the
    // target isn't statically known from the bytes, leave it out rather than
    // report a guess as a fact.
    //
    // Core-agnostic like CallersCommand: works purely off
    // IDebugTarget.Disassemble, so any core that implements IDebugTarget
    // gets this command for free.
    public class WritersCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "writers";
        public string Usage => string.Join('\n', new[]
        {
            "  writers <addr> [<scanstart> <scanlen>]",
            "                                find STA/STX/STY/STZ instructions targeting <addr>",
            "                                (absolute/absolute-long only - see source comment",
            "                                for why direct-page/indexed/indirect are excluded),",
            "                                scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)",
        });

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Debug.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: writers <addr> [<scanstart> <scanlen>]";
            int targetAddr = ParseHex(parts[1]);

            int scanStart, scanLen;
            if (parts.Length >= 4)
            {
                scanStart = ParseHex(parts[2]);
                scanLen = ParseHex(parts[3]);
            }
            else
            {
                scanStart = (targetAddr & 0xFF0000) | 0x8000;
                scanLen = 0x8000;
            }

            var instrs = target.Disassemble("CpuBus", scanStart, scanLen);

            var matches = new List<string>();
            foreach (var instr in instrs)
            {
                if (instr.Address >= scanStart + scanLen) break;
                byte opcode = instr.Bytes[0];

                int? writeTarget = opcode switch
                {
                    0x8D => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // STA absolute
                    0x8F => instr.Bytes[1] | (instr.Bytes[2] << 8) | (instr.Bytes[3] << 16), // STA absolute long
                    0x8E => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // STX absolute
                    0x8C => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // STY absolute
                    0x9C => (instr.Address & 0xFF0000) | (instr.Bytes[1] | (instr.Bytes[2] << 8)), // STZ absolute
                    _ => null
                };

                if (writeTarget.HasValue && writeTarget.Value == targetAddr)
                {
                    matches.Add($"  ${instr.Address:X6}: {instr.Mnemonic} {instr.OperandText}");
                }
            }

            if (matches.Count == 0)
            {
                return $"No STA/STX/STY/STZ found targeting ${targetAddr:X6} in ${scanStart:X6}-${scanStart + scanLen - 1:X6} (absolute/absolute-long only).";
            }
            return string.Join('\n', matches);
        }
    }
}
