using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // Standalone NEC DSP disassembler - see Venus_NecDSP.md §8.
    public static class NecDspDisassembler
    {
        private static readonly string[] AluOps =
            { "NOP", "OR", "AND", "XOR", "SUB", "ADD", "SBC", "ADC", "DEC", "INC", "CMP", "SHR1", "SHL1", "SHL2", "SHL4", "XCHG" };

        // Index order matches GetSourceValue - see NecDsp.Execute.cs.
        private static readonly string[] Sources =
            { "TRB", "A", "B", "TR", "DP", "RP", "ROM", "SGN", "DR", "DRNF", "SR", "SIM", "SIL", "K", "L", "MEM" };

        // Index order matches Load - see NecDsp.Execute.cs.
        private static readonly string[] Dests =
            { "NON", "A", "B", "TR", "DP", "RP", "DR", "SR", "SOL", "SOM", "K", "KLR", "KLM", "L", "TRB", "MEM" };

        private static readonly string[] DataPointerOps = { "DPL:NOP", "DPL:INC", "DPL:DEC", "DPL:CLR" };

        // <read> takes a program-word index and returns the whole 24-bit word.
        public static IReadOnlyList<DisassembledInstruction> Disassemble(
            Func<int, int> read, int address, int count)
        {
            var result = new List<DisassembledInstruction>(count);

            for (int i = 0; i < count; i++)
            {
                int pc = address + i;
                int opcode = read(pc) & 0xFFFFFF;
                var bytes = new byte[] { (byte)opcode, (byte)(opcode >> 8), (byte)(opcode >> 16) };

                var (mnemonic, operands) = Decode(opcode, pc);
                result.Add(new DisassembledInstruction(pc, bytes, mnemonic, operands));
            }

            return result;
        }

        private static (string Mnemonic, string Operands) Decode(int opcode, int pc)
        {
            return ((opcode >> 22) & 0x03) switch
            {
                0 or 1 => DecodeOp(opcode, isReturn: ((opcode >> 22) & 0x03) == 1),
                2 => DecodeJump(opcode, pc),
                _ => ("LDI", $"{Dests[opcode & 0x0F]}, #${(opcode >> 6) & 0xFFFF:X4}"),
            };
        }

        // One word can do an ALU op, a move, a DP step and an RP step at once.
        private static (string, string) DecodeOp(int opcode, bool isReturn)
        {
            int alu = (opcode >> 16) & 0x0F;
            int source = (opcode >> 4) & 0x0F;
            int dest = opcode & 0x0F;
            bool accB = ((opcode >> 15) & 0x01) != 0;

            var parts = new List<string>();

            if (alu != 0)
            {
                string operand = alu <= 7
                    ? ((opcode >> 20) & 0x03) switch
                    {
                        0 => "RAM",
                        1 => Sources[source],
                        2 => "M",
                        _ => "N",
                    } + ", "
                    : string.Empty;
                parts.Add($"{AluOps[alu]} {operand}{(accB ? "B" : "A")}");
            }

            if (dest != 0) parts.Add($"MOV {Sources[source]}, {Dests[dest]}");

            int dpOp = (opcode >> 13) & 0x03;
            if (dpOp != 0) parts.Add(DataPointerOps[dpOp]);

            int dpLow = (opcode >> 9) & 0x0F;
            if (dpLow != 0) parts.Add($"DPH:${dpLow:X1}");

            if (((opcode >> 8) & 0x01) != 0) parts.Add("RP--");

            if (parts.Count == 0) parts.Add("NOP");

            string mnemonic = isReturn ? "RT" : "OP";
            return (mnemonic, string.Join(" | ", parts));
        }

        private static (string, string) DecodeJump(int opcode, int pc)
        {
            int target = (pc & 0x2000) | ((opcode & 0x03) << 11) | ((opcode >> 2) & 0x7FF);
            int jumpType = (opcode >> 13) & 0x1FF;

            // Only the unconditional forms pick their own bank - see Venus_NecDSP.md §5.4.
            if (jumpType is 0x100 or 0x140) target &= ~0x2000;
            else if (jumpType is 0x101 or 0x141) target |= 0x2000;

            string condition = jumpType switch
            {
                0x000 => "JMPSO",
                0x080 => "JNCA", 0x082 => "JCA", 0x084 => "JNCB", 0x086 => "JCB",
                0x088 => "JNZA", 0x08A => "JZA", 0x08C => "JNZB", 0x08E => "JZB",
                0x090 => "JNOVA0", 0x092 => "JOVA0", 0x094 => "JNOVB0", 0x096 => "JOVB0",
                0x098 => "JNOVA1", 0x09A => "JOVA1", 0x09C => "JNOVB1", 0x09E => "JOVB1",
                0x0A0 => "JNSA0", 0x0A2 => "JSA0", 0x0A4 => "JNSB0", 0x0A6 => "JSB0",
                0x0A8 => "JNSA1", 0x0AA => "JSA1", 0x0AC => "JNSB1", 0x0AE => "JSB1",
                0x0B0 => "JDPL0", 0x0B1 => "JDPLN0", 0x0B2 => "JDPLF", 0x0B3 => "JDPLNF",
                0x0B4 => "JNSIAK", 0x0B6 => "JSIAK", 0x0B8 => "JNSOAK", 0x0BA => "JSOAK",
                0x0BC => "JNRQM", 0x0BE => "JRQM",
                0x100 or 0x101 => "JMP", 0x140 or 0x141 => "CALL",
                _ => $"JMP?{jumpType:X3}",
            };

            return (condition, $"${target:X4}");
        }
    }
}
