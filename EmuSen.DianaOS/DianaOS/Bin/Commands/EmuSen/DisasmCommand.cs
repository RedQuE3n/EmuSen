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
        public string Usage => "  disasm <space> <addr> [<n>]   disassemble <n> instructions (default 10)";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 3) return "Usage: disasm <space> <addr> [<count>]";
            FindSpace(target, parts[1]); // validates the space name, or throws a helpful error
            int addr = ParseHex(parts[2]);
            int count = parts.Length >= 4 ? ParseHex(parts[3]) : 10;

            var instructions = target.Disassemble(parts[1], addr, count);
            if (instructions.Count == 0) return $"{target.CoreName} target has no disassembler, or nothing was returned.";

            var sb = new StringBuilder();
            foreach (var instr in instructions)
            {
                string bytesHex = string.Join(' ', instr.Bytes.Select(b => b.ToString("X2")));
                sb.AppendLine($"  {instr.Address:X6}: {bytesHex,-9} {instr.Mnemonic} {instr.OperandText}".TrimEnd());
            }
            return sb.ToString().TrimEnd();
        }
    }
}
