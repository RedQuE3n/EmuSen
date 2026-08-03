using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.SuperFx
{
    // Standalone GSU disassembler, separate from SuperFx.Execute's dispatch
    // table for the same reason Snes65816Disassembler is separate from Cpu -
    // see Venus_SuperFX.md §8.1.
    public static class GsuDisassembler
    {
        // How many operand bytes follow the opcode, before any prefix applies.
        private static int OperandLength(byte opcode) => opcode switch
        {
            >= 0x05 and <= 0x0F => 1,   // branch displacement
            >= 0xA0 and <= 0xAF => 1,   // IBT/LMS/SMS
            >= 0xF0 and <= 0xFF => 2,   // IWT/LM/SM
            _ => 0,
        };

        // A prefix survives into the next instruction; everything else clears
        // it. Branches are the exception that §4.2 exists for.
        private static bool KeepsPrefix(byte opcode, bool with) => opcode switch
        {
            >= 0x05 and <= 0x0F => true,
            >= 0x10 and <= 0x1F => !with,
            >= 0x20 and <= 0x2F => true,
            >= 0x3D and <= 0x3F => true,
            >= 0xB0 and <= 0xBF => !with,
            _ => false,
        };

        // Walks forward from <address> tracking ALT1/ALT2/WITH/TO/FROM, so each
        // line is decoded under the prefix state its predecessors actually left
        // - see Venus_SuperFX.md §8.1 for why a static table cannot.
        public static IReadOnlyList<DisassembledInstruction> Disassemble(
            Func<int, byte> read, int address, int count)
        {
            var result = new List<DisassembledInstruction>(count);
            bool alt1 = false, alt2 = false, with = false;
            int sreg = 0, dreg = 0;

            for (int i = 0; i < count; i++)
            {
                int pc = address & 0xFFFFFF;
                byte opcode = read(pc);
                int operandBytes = OperandLength(opcode);

                var bytes = new byte[1 + operandBytes];
                bytes[0] = opcode;
                for (int b = 0; b < operandBytes; b++) bytes[b + 1] = read((pc & 0xFF0000) | ((pc + 1 + b) & 0xFFFF));

                var (mnemonic, operand) = Decode(opcode, bytes, pc, alt1, alt2, with, sreg, dreg);
                result.Add(new DisassembledInstruction(pc, bytes, mnemonic, operand));

                // Mirror the dispatcher's own prefix bookkeeping, in order.
                if (opcode >= 0x10 && opcode <= 0x1F && !with) dreg = opcode & 0x0F;
                else if (opcode >= 0x20 && opcode <= 0x2F) { sreg = dreg = opcode & 0x0F; with = true; }
                else if (opcode >= 0xB0 && opcode <= 0xBF && !with) sreg = opcode & 0x0F;

                if (opcode == 0x3D) alt1 = true;
                else if (opcode == 0x3E) alt2 = true;
                else if (opcode == 0x3F) { alt1 = true; alt2 = true; }

                if (!KeepsPrefix(opcode, with)) { alt1 = alt2 = with = false; sreg = dreg = 0; }

                address = (pc & 0xFF0000) | ((pc + bytes.Length) & 0xFFFF);
            }

            return result;
        }

        private static (string Mnemonic, string Operand) Decode(
            byte opcode, byte[] bytes, int pc, bool alt1, bool alt2, bool with, int sreg, int dreg)
        {
            int n = opcode & 0x0F;
            string rn = $"R{n}";

            // ALT3 is ALT1+ALT2; the four states select one mnemonic per slot.
            int alt = (alt1 ? 1 : 0) | (alt2 ? 2 : 0);

            switch (opcode)
            {
                case 0x00: return ("STOP", "");
                case 0x01: return ("NOP", "");
                case 0x02: return ("CACHE", "");
                case 0x03: return ("LSR", "");
                case 0x04: return ("ROL", "");

                case >= 0x05 and <= 0x0F:
                {
                    string[] names = { "BRA", "BGE", "BLT", "BNE", "BEQ", "BPL", "BMI", "BCC", "BCS", "BVC", "BVS" };
                    // The displacement is relative to the delay slot, i.e. to pc+2.
                    int target = (pc + 2 + (sbyte)bytes[1]) & 0xFFFF;
                    return (names[opcode - 0x05], $"${target:X4}");
                }

                case >= 0x10 and <= 0x1F:
                    return with ? ("MOVE", $"{rn},R{sreg}") : ("TO", rn);

                case >= 0x20 and <= 0x2F: return ("WITH", rn);
                case >= 0x30 and <= 0x3B: return (alt1 ? "STB" : "STW", $"({rn})");

                case 0x3C: return ("LOOP", "");
                case 0x3D: return ("ALT1", "");
                case 0x3E: return ("ALT2", "");
                case 0x3F: return ("ALT3", "");

                case >= 0x40 and <= 0x4B: return (alt1 ? "LDB" : "LDW", $"({rn})");

                case 0x4C: return (alt1 ? "RPIX" : "PLOT", "");
                case 0x4D: return ("SWAP", "");
                case 0x4E: return (alt1 ? "CMODE" : "COLOR", "");
                case 0x4F: return ("NOT", "");

                case >= 0x50 and <= 0x5F:
                    return (alt switch { 0 => "ADD", 1 => "ADC", 2 => "ADD", _ => "ADC" }, alt2 ? $"#{n}" : rn);

                case >= 0x60 and <= 0x6F:
                    return (alt switch { 0 => "SUB", 1 => "SBC", 2 => "SUB", _ => "CMP" },
                            alt == 2 ? $"#{n}" : rn);

                case 0x70: return ("MERGE", "");

                case >= 0x71 and <= 0x7F:
                    return (alt switch { 0 => "AND", 1 => "BIC", 2 => "AND", _ => "BIC" }, alt2 ? $"#{n}" : rn);

                case >= 0x80 and <= 0x8F:
                    return (alt switch { 0 => "MULT", 1 => "UMULT", 2 => "MULT", _ => "UMULT" }, alt2 ? $"#{n}" : rn);

                case 0x90: return ("SBK", "");
                case >= 0x91 and <= 0x94: return ("LINK", $"#{n}");
                case 0x95: return ("SEX", "");
                case 0x96: return (alt1 ? "DIV2" : "ASR", "");
                case 0x97: return ("ROR", "");
                case >= 0x98 and <= 0x9D: return (alt1 ? "LJMP" : "JMP", rn);
                case 0x9E: return ("LOB", "");
                case 0x9F: return (alt1 ? "LMULT" : "FMULT", "");

                case >= 0xA0 and <= 0xAF:
                    // ALT3 is undefined here and falls through to IBT - see §9.
                    return alt switch
                    {
                        1 => ("LMS", $"{rn},(${bytes[1] * 2:X4})"),
                        2 => ("SMS", $"(${bytes[1] * 2:X4}),{rn}"),
                        _ => ("IBT", $"{rn},#${bytes[1]:X2}"),
                    };

                case >= 0xB0 and <= 0xBF:
                    return with ? ("MOVES", $"R{dreg},{rn}") : ("FROM", rn);

                case 0xC0: return ("HIB", "");

                case >= 0xC1 and <= 0xCF:
                    return (alt switch { 0 => "OR", 1 => "XOR", 2 => "OR", _ => "XOR" }, alt2 ? $"#{n}" : rn);

                case >= 0xD0 and <= 0xDE: return ("INC", rn);
                case 0xDF: return (alt switch { 0 => "GETC", 1 => "GETC", 2 => "RAMB", _ => "ROMB" }, "");
                case >= 0xE0 and <= 0xEE: return ("DEC", rn);
                case 0xEF: return (alt switch { 0 => "GETB", 1 => "GETBH", 2 => "GETBL", _ => "GETBS" }, "");

                default:
                {
                    // $F0-$FF. ALT3 is undefined and falls through to IWT - see §9.
                    ushort imm = (ushort)(bytes[1] | (bytes[2] << 8));
                    return alt switch
                    {
                        1 => ("LM", $"{rn},(${imm:X4})"),
                        2 => ("SM", $"(${imm:X4}),{rn}"),
                        _ => ("IWT", $"{rn},#${imm:X4}"),
                    };
                }
            }
        }
    }
}
