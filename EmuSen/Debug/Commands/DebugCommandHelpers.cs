using System;
using System.Linq;

namespace EmuSen.Debug.Commands
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
    }
}
