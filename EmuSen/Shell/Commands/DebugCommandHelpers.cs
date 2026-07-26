using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Shell.Commands
{
    // Small, stateless helpers shared across multiple IShellCommand
    // implementations - the same role ShellInterpreter's private
    // static methods used to serve before commands moved into their own
    // classes. Kept as plain static methods (not an instance/DI thing)
    // since none of them need any state of their own beyond their
    // arguments, matching DebugTools.cs's existing shape for the same
    // kind of "generic, reusable, no state" helper.
    public static class DebugCommandHelpers
    {
        public static int ParseHex(string s)
        {
            s = s.Trim();
            if (s.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) s = s.Substring(2);
            else if (s.StartsWith("$")) s = s.Substring(1);
            return Convert.ToInt32(s, 16);
        }

        // Every debug command (mem, regs, watch, ...) fundamentally needs
        // a real emulator session to do anything - unlike the shell-level
        // commands (echo, sed, true/false...) that work with target ==
        // null just fine. Centralizes the null check so each command gets
        // a clean "no ROM loaded" error instead of a NullReferenceException,
        // without every single command file needing its own guard.
        public static IDebugTarget RequireTarget(IDebugTarget? target)
        {
            if (target == null) throw new InvalidOperationException("No ROM loaded - this command needs an active debug target.");
            return target;
        }

        public static IDebugMemorySpace FindSpace(IDebugTarget? target, string name)
        {
            IDebugTarget t = RequireTarget(target);
            var spaces = t.GetMemorySpaces();
            var match = spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));
            if (match == null)
            {
                string available = string.Join(", ", spaces.Select(s => s.Name));
                throw new ArgumentException($"No memory space named '{name}'. Available: {available}");
            }
            return match;
        }

        // Little-endian multi-byte read, matching the 65816/SNES
        // convention every other multi-byte value in this codebase uses
        // (see Snes65816Disassembler.FormatOperand's absolute-address
        // formatting for the same low-byte-first pattern). Shared by
        // SearchCommand's value comparisons - a future core's memory
        // spaces would read the same way for its own little-endian
        // values, or a core-specific command could read multi-byte
        // values its own way if it genuinely needed to.
        public static long ReadValue(IDebugMemorySpace space, int addr, int width)
        {
            long v = 0;
            for (int i = 0; i < width; i++) v |= (long)space.Read(addr + i) << (8 * i);
            return v;
        }

        // Shared by CallersCommand/WritersCommand/ReadersCommand: absent an
        // explicit <scanstart> <scanlen>, default to <addr>'s own bank's
        // upper 32KB - the conventional LoROM code region (see MemoryBus's
        // own "offset >= 0x8000 is ROM" comment) and a reasonable "just show
        // me this bank's hits" starting point without needing to already
        // know a scan range.
        public static (int ScanStart, int ScanLen) DefaultScanRange(string[] parts, int argOffset, int targetAddr)
        {
            if (parts.Length >= argOffset + 2)
            {
                return (ParseHex(parts[argOffset]), ParseHex(parts[argOffset + 1]));
            }
            return ((targetAddr & 0xFF0000) | 0x8000, 0x8000);
        }

        // Shared scan-and-match skeleton for CallersCommand/WritersCommand/
        // ReadersCommand: walk every disassembled instruction in
        // [scanStart, scanStart+scanLen), ask `resolveTarget` whether it has
        // a statically-known target address (returning null for anything it
        // doesn't recognize or can't resolve from the instruction bytes
        // alone), and collect the ones matching <targetAddr>. `resolveTarget`
        // is where each command's own opcode knowledge lives - this helper
        // only owns the walking/formatting/not-found-message part all three
        // share verbatim.
        //
        // NOTE: `resolveTarget` is necessarily ISA-specific (each command
        // passes its own 65816 opcode-byte switch) - this helper removes the
        // three commands' literal duplication of the scan loop, but does NOT
        // make the scan itself core-agnostic. A non-65816 core would need
        // its own opcode table here, or (better, if a second core ever
        // lands) this would want to move into IDebugTarget itself, the way
        // Disassemble()/DecodeTilemapEntry() already delegate their own
        // ISA-specific decoding to the core rather than hardcoding it here.
        public static string ScanForStaticReferences(
            IDebugTarget target,
            int scanStart,
            int scanLen,
            int targetAddr,
            string notFoundMnemonics,
            Func<byte, DisassembledInstruction, int?> resolveTarget,
            string notFoundSuffix = "")
        {
            var instrs = target.Disassemble("CpuBus", scanStart, scanLen);

            var matches = new List<string>();
            foreach (var instr in instrs)
            {
                if (instr.Address >= scanStart + scanLen) break;
                int? refTarget = resolveTarget(instr.Bytes[0], instr);
                if (refTarget.HasValue && refTarget.Value == targetAddr)
                {
                    matches.Add($"  ${instr.Address:X6}: {instr.Mnemonic} {instr.OperandText}");
                }
            }

            if (matches.Count == 0)
            {
                return $"No {notFoundMnemonics} found targeting ${targetAddr:X6} in ${scanStart:X6}-${scanStart + scanLen - 1:X6}{notFoundSuffix}.";
            }
            return string.Join('\n', matches);
        }
    }
}
