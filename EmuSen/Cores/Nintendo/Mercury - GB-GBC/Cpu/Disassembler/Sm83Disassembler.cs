using System;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Mercury.Cpu.Disassembler
{
    // A second opcode table, independent of the executing one - see Mercury_Debug.md §4.
    public static class Sm83Disassembler
    {
        // The placeholders a template can carry; each one also fixes the instruction's length.
        private const string Byte = "d8";
        private const string Word = "d16";
        private const string HighPage = "a8";
        private const string Absolute = "a16";
        private const string Relative = "r8";
        private const string Signed = "s8";

        // What an opcode with no instruction behind it prints as; the CPU throws on these - see Mercury_Cpu.md §6.1.
        public const string Illegal = "???";

        private static readonly string[] Templates =
        {
            /* 0x0_ */
            "NOP", "LD BC,d16", "LD (BC),A", "INC BC", "INC B", "DEC B", "LD B,d8", "RLCA",
            "LD (a16),SP", "ADD HL,BC", "LD A,(BC)", "DEC BC", "INC C", "DEC C", "LD C,d8", "RRCA",

            /* 0x1_ */
            "STOP d8", "LD DE,d16", "LD (DE),A", "INC DE", "INC D", "DEC D", "LD D,d8", "RLA",
            "JR r8", "ADD HL,DE", "LD A,(DE)", "DEC DE", "INC E", "DEC E", "LD E,d8", "RRA",

            /* 0x2_ */
            "JR NZ,r8", "LD HL,d16", "LD (HL+),A", "INC HL", "INC H", "DEC H", "LD H,d8", "DAA",
            "JR Z,r8", "ADD HL,HL", "LD A,(HL+)", "DEC HL", "INC L", "DEC L", "LD L,d8", "CPL",

            /* 0x3_ */
            "JR NC,r8", "LD SP,d16", "LD (HL-),A", "INC SP", "INC (HL)", "DEC (HL)", "LD (HL),d8", "SCF",
            "JR C,r8", "ADD HL,SP", "LD A,(HL-)", "DEC SP", "INC A", "DEC A", "LD A,d8", "CCF",

            /* 0x4_ */
            "LD B,B", "LD B,C", "LD B,D", "LD B,E", "LD B,H", "LD B,L", "LD B,(HL)", "LD B,A",
            "LD C,B", "LD C,C", "LD C,D", "LD C,E", "LD C,H", "LD C,L", "LD C,(HL)", "LD C,A",

            /* 0x5_ */
            "LD D,B", "LD D,C", "LD D,D", "LD D,E", "LD D,H", "LD D,L", "LD D,(HL)", "LD D,A",
            "LD E,B", "LD E,C", "LD E,D", "LD E,E", "LD E,H", "LD E,L", "LD E,(HL)", "LD E,A",

            /* 0x6_ */
            "LD H,B", "LD H,C", "LD H,D", "LD H,E", "LD H,H", "LD H,L", "LD H,(HL)", "LD H,A",
            "LD L,B", "LD L,C", "LD L,D", "LD L,E", "LD L,H", "LD L,L", "LD L,(HL)", "LD L,A",

            /* 0x7_ */
            "LD (HL),B", "LD (HL),C", "LD (HL),D", "LD (HL),E", "LD (HL),H", "LD (HL),L", "HALT", "LD (HL),A",
            "LD A,B", "LD A,C", "LD A,D", "LD A,E", "LD A,H", "LD A,L", "LD A,(HL)", "LD A,A",

            /* 0x8_ */
            "ADD A,B", "ADD A,C", "ADD A,D", "ADD A,E", "ADD A,H", "ADD A,L", "ADD A,(HL)", "ADD A,A",
            "ADC A,B", "ADC A,C", "ADC A,D", "ADC A,E", "ADC A,H", "ADC A,L", "ADC A,(HL)", "ADC A,A",

            /* 0x9_ */
            "SUB B", "SUB C", "SUB D", "SUB E", "SUB H", "SUB L", "SUB (HL)", "SUB A",
            "SBC A,B", "SBC A,C", "SBC A,D", "SBC A,E", "SBC A,H", "SBC A,L", "SBC A,(HL)", "SBC A,A",

            /* 0xA_ */
            "AND B", "AND C", "AND D", "AND E", "AND H", "AND L", "AND (HL)", "AND A",
            "XOR B", "XOR C", "XOR D", "XOR E", "XOR H", "XOR L", "XOR (HL)", "XOR A",

            /* 0xB_ */
            "OR B", "OR C", "OR D", "OR E", "OR H", "OR L", "OR (HL)", "OR A",
            "CP B", "CP C", "CP D", "CP E", "CP H", "CP L", "CP (HL)", "CP A",

            /* 0xC_ */
            "RET NZ", "POP BC", "JP NZ,a16", "JP a16", "CALL NZ,a16", "PUSH BC", "ADD A,d8", "RST 00H",
            "RET Z", "RET", "JP Z,a16", "PREFIX CB", "CALL Z,a16", "CALL a16", "ADC A,d8", "RST 08H",

            /* 0xD_ */
            "RET NC", "POP DE", "JP NC,a16", "???", "CALL NC,a16", "PUSH DE", "SUB d8", "RST 10H",
            "RET C", "RETI", "JP C,a16", "???", "CALL C,a16", "???", "SBC A,d8", "RST 18H",

            /* 0xE_ */
            "LDH (a8),A", "POP HL", "LD (C),A", "???", "???", "PUSH HL", "AND d8", "RST 20H",
            "ADD SP,s8", "JP (HL)", "LD (a16),A", "???", "???", "???", "XOR d8", "RST 28H",

            /* 0xF_ */
            "LDH A,(a8)", "POP AF", "LD A,(C)", "DI", "???", "PUSH AF", "OR d8", "RST 30H",
            "LD HL,SPs8", "LD SP,HL", "LD A,(a16)", "EI", "???", "???", "CP d8", "RST 38H",
        };

        private static readonly string[] CbOperations =
        {
            "RLC", "RRC", "RL", "RR", "SLA", "SRA", "SWAP", "SRL",
        };

        private static readonly string[] CbTargets =
        {
            "B", "C", "D", "E", "H", "L", "(HL)", "A",
        };

        // The whole $CB page is two bytes: the prefix and the operation - see Mercury_Cpu.md §6.2.
        public const int CbLength = 2;

        public static int LengthOf(byte opcode)
        {
            if (opcode == 0xCB) return CbLength;

            string template = Templates[opcode];

            if (template.Contains(Word, StringComparison.Ordinal)) return 3;
            if (template.Contains(Absolute, StringComparison.Ordinal)) return 3;

            return template.Contains(Byte, StringComparison.Ordinal)
                || template.Contains(HighPage, StringComparison.Ordinal)
                || template.Contains(Relative, StringComparison.Ordinal)
                || template.Contains(Signed, StringComparison.Ordinal) ? 2 : 1;
        }

        public static string MnemonicOf(byte opcode, byte suffix)
        {
            if (opcode == 0xCB) return CbMnemonic(suffix);

            string template = Templates[opcode];
            int space = template.IndexOf(' ');
            return space < 0 ? template : template[..space];
        }

        // <address> is only needed to resolve a relative branch, which is measured from the next instruction.
        public static string FormatOperand(byte opcode, int address, byte low, byte high)
        {
            if (opcode == 0xCB) return CbOperand(low);

            string template = Templates[opcode];
            int space = template.IndexOf(' ');
            if (space < 0) return "";

            string operands = template[(space + 1)..];
            int word = low | (high << 8);

            operands = operands.Replace(Word, $"${word:X4}", StringComparison.Ordinal);
            operands = operands.Replace(Absolute, $"${word:X4}", StringComparison.Ordinal);
            operands = operands.Replace(HighPage, $"$FF{low:X2}", StringComparison.Ordinal);
            operands = operands.Replace(Relative, $"${(ushort)(address + 2 + (sbyte)low):X4}", StringComparison.Ordinal);

            // A displacement, not a target: printed signed so "SP-2" does not read as "SP+254".
            operands = operands.Replace(Signed, $"{(sbyte)low:+0;-0;+0}", StringComparison.Ordinal);
            operands = operands.Replace(Byte, $"${low:X2}", StringComparison.Ordinal);

            return operands;
        }

        private static string CbMnemonic(byte suffix)
        {
            if (suffix < 0x40) return CbOperations[suffix >> 3];
            return suffix < 0x80 ? "BIT" : suffix < 0xC0 ? "RES" : "SET";
        }

        private static string CbOperand(byte suffix)
        {
            string target = CbTargets[suffix & 0x07];
            return suffix < 0x40 ? target : $"{(suffix >> 3) & 0x07},{target}";
        }

        // Only the forms whose target is knowable from the bytes alone - see IDebugTarget's own comment.
        public static (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(byte opcode, int operand, int address)
        {
            switch (opcode)
            {
                // CALL and JP with an absolute operand, plus every conditional form of both.
                case 0xC4 or 0xCC or 0xCD or 0xD4 or 0xDC:
                case 0xC2 or 0xC3 or 0xCA or 0xD2 or 0xDA:
                    return (StaticReferenceKind.Call, operand & 0xFFFF);

                // RST's target is in the opcode itself, so there is nothing runtime about it.
                case 0xC7 or 0xCF or 0xD7 or 0xDF or 0xE7 or 0xEF or 0xF7 or 0xFF:
                    return (StaticReferenceKind.Call, opcode & 0x38);

                case 0x18 or 0x20 or 0x28 or 0x30 or 0x38:
                    return (StaticReferenceKind.Call, (ushort)(address + 2 + (sbyte)operand));

                case 0x08 or 0xEA:
                    return (StaticReferenceKind.Write, operand & 0xFFFF);

                case 0xE0:
                    return (StaticReferenceKind.Write, 0xFF00 | (operand & 0xFF));

                case 0xFA:
                    return (StaticReferenceKind.Read, operand & 0xFFFF);

                case 0xF0:
                    return (StaticReferenceKind.Read, 0xFF00 | (operand & 0xFF));

                default:
                    return null;
            }
        }
    }
}
