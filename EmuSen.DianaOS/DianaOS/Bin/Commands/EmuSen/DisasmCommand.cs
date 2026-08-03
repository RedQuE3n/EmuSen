using System.Linq;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    public class DisasmCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "disasm";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  disasm <space> <addr> [<n>]   disassemble <n> instructions (default 10)",
            "  disasm <cpu> [<addr>] [<n>]   same, in that processor's own code space and ISA -",
            "                                with no <addr>, starts at where it is executing now",
            "  ... [m8|m16] [x8|x16]         65816 only: force the starting operand widths",
        });

        private static bool IsWidthHint(string s) =>
            s is "m8" or "m16" or "x8" or "x16" or "M8" or "M16" or "X8" or "X16";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: disasm <space>|<cpu> <addr> [<count>]";

            // Pulled out first so they never read as <addr>/<count>.
            var hints = parts.Where(IsWidthHint).ToArray();
            if (hints.Length > 0) parts = parts.Where(p => !IsWidthHint(p)).ToArray();

            // A chip name stands in for its own code space - see `man cpus`.
            var named = DebugCpus.Find(target.DebugCpus, parts[1]);
            string space;
            int at;
            if (named?.CodeSpace != null) { space = named.CodeSpace; at = 2; }
            else { space = parts[1]; at = 2; named = null; }

            if (named == null && parts.Length < 3) return "Usage: disasm <space> <addr> [<count>]";

            FindSpace(target, space); // validates the space name, or throws a helpful error

            int addr;
            if (parts.Length > at) addr = ParseHex(parts[at]);
            else if (named?.ProgramCounter is { } pc) addr = pc();
            else return $"Usage: disasm {parts[1]} <addr> [<count>]";

            int count = parts.Length >= at + 2 ? ParseHex(parts[at + 1]) : 10;

            var instructions = target.Disassemble(space, addr, count, hints);
            if (instructions.Count == 0) return $"{target.CoreName} target has no disassembler, or nothing was returned.";

            var labels = target.Labels;
            var sb = new StringBuilder();
            // Echoed so a scripted run records which widths produced the listing.
            if (hints.Length > 0) sb.AppendLine($"  ; decoding with {string.Join(' ', hints).ToLowerInvariant()}");
            foreach (var instr in instructions)
            {
                // A label gets its own line - see `man label`.
                if (labels != null && labels.TryGetName(instr.Address, out string name))
                {
                    string comment = labels.CommentAt(instr.Address) is { } c ? $"  ; {c}" : string.Empty;
                    sb.AppendLine($"{name}:{comment}");
                }
                string bytesHex = string.Join(' ', instr.Bytes.Select(b => b.ToString("X2")));
                string target24 = ReferenceLabel(target, labels, instr);
                sb.AppendLine($"  {instr.Address:X6}: {bytesHex,-9} {instr.Mnemonic} {instr.OperandText}{target24}".TrimEnd());
            }
            return sb.ToString().TrimEnd();
        }

        // Annotates an operand whose static target happens to be labelled.
        private static string ReferenceLabel(IDebugTarget target, LabelRegistry? labels, DisassembledInstruction instr)
        {
            if (labels == null) return string.Empty;
            if (target.ClassifyStaticReference(instr) is not { } reference) return string.Empty;
            return labels.TryGetName(reference.Target, out string name) ? $"  ; <{name}>" : string.Empty;
        }
    }
}
