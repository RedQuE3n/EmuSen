using System;
using System.Collections.Generic;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    // Standalone SPC700 disassembler, separate from the interpreter's dispatch
    // table for the same reason Snes65816Disassembler is - see Venus_APU.md §8.
    public static class Spc700Disassembler
    {
        // Operand text uses byte-indexed placeholders - see Venus_APU.md §8.1.
        private readonly record struct Entry(string Mnemonic, string Operands, int Length);

        private static readonly Entry[] Table = BuildTable();

        public static IReadOnlyList<DisassembledInstruction> Disassemble(
            Func<int, byte> read, int address, int count)
        {
            var result = new List<DisassembledInstruction>(count);
            int pc = address & 0xFFFF;

            for (int i = 0; i < count; i++)
            {
                byte opcode = read(pc);
                var entry = Table[opcode];

                var bytes = new byte[entry.Length];
                for (int b = 0; b < entry.Length; b++) bytes[b] = read((pc + b) & 0xFFFF);

                result.Add(new DisassembledInstruction(pc, bytes, entry.Mnemonic, Format(entry, bytes, pc)));
                pc = (pc + entry.Length) & 0xFFFF;
            }

            return result;
        }

        private static string Format(Entry entry, byte[] bytes, int pc)
        {
            if (entry.Operands.Length == 0) return string.Empty;

            string text = entry.Operands;
            if (text.Contains("d0")) text = text.Replace("d0", $"${bytes[1]:X2}");
            if (text.Contains("d1")) text = text.Replace("d1", $"${bytes[2]:X2}");
            if (text.Contains("i0")) text = text.Replace("i0", $"#${bytes[1]:X2}");
            if (text.Contains("i1")) text = text.Replace("i1", $"#${bytes[2]:X2}");
            if (text.Contains("aa")) text = text.Replace("aa", $"${bytes[1] | (bytes[2] << 8):X4}");
            if (text.Contains("r1")) text = text.Replace("r1", $"${(pc + entry.Length + (sbyte)bytes[1]) & 0xFFFF:X4}");
            if (text.Contains("r2")) text = text.Replace("r2", $"${(pc + entry.Length + (sbyte)bytes[2]) & 0xFFFF:X4}");
            if (text.Contains("uu")) text = text.Replace("uu", $"$FF{bytes[1]:X2}");

            // A 13-bit address with the bit number in the top 3 bits.
            if (text.Contains("mm"))
            {
                int word = bytes[1] | (bytes[2] << 8);
                text = text.Replace("mm", $"${word & 0x1FFF:X4}.{word >> 13}");
            }

            return text;
        }

        private static Entry[] BuildTable()
        {
            var t = new Entry[256];
            for (int i = 0; i < 256; i++) t[i] = new Entry(".db", $"${i:X2}", 1);

            void Set(int op, string mnemonic, string operands = "", int length = 1)
                => t[op] = new Entry(mnemonic, operands, length);

            // The eight ALU rows share one operand shape - see Venus_APU.md §8.1.
            void AluRow(int baseOp, string mnemonic)
            {
                Set(baseOp + 0x04, mnemonic, "A, d0", 2);
                Set(baseOp + 0x05, mnemonic, "A, aa", 3);
                Set(baseOp + 0x06, mnemonic, "A, (X)");
                Set(baseOp + 0x07, mnemonic, "A, [d0+X]", 2);
                Set(baseOp + 0x08, mnemonic, "A, i0", 2);
                Set(baseOp + 0x09, mnemonic, "d1, d0", 3);
                Set(baseOp + 0x14, mnemonic, "A, d0+X", 2);
                Set(baseOp + 0x15, mnemonic, "A, aa+X", 3);
                Set(baseOp + 0x16, mnemonic, "A, aa+Y", 3);
                Set(baseOp + 0x17, mnemonic, "A, [d0]+Y", 2);
                Set(baseOp + 0x18, mnemonic, "d1, i0", 3);
                Set(baseOp + 0x19, mnemonic, "(X), (Y)");
            }

            AluRow(0x00, "OR");
            AluRow(0x20, "AND");
            AluRow(0x40, "EOR");
            AluRow(0x60, "CMP");
            AluRow(0x80, "ADC");
            AluRow(0xA0, "SBC");

            // TCALL n and the SET1/CLR1/BBS/BBC bit ladders are pure nibble patterns.
            for (int n = 0; n < 16; n++) Set(0x01 + n * 0x10, "TCALL", n.ToString());
            for (int bit = 0; bit < 8; bit++)
            {
                Set(0x02 + bit * 0x20, "SET1", $"d0.{bit}", 2);
                Set(0x12 + bit * 0x20, "CLR1", $"d0.{bit}", 2);
                Set(0x03 + bit * 0x20, "BBS", $"d0.{bit}, r2", 3);
                Set(0x13 + bit * 0x20, "BBC", $"d0.{bit}, r2", 3);
            }

            Set(0x00, "NOP");
            Set(0x0A, "OR1", "C, mm", 3);
            Set(0x0B, "ASL", "d0", 2);
            Set(0x0C, "ASL", "aa", 3);
            Set(0x0D, "PUSH", "PSW");
            Set(0x0E, "TSET1", "aa", 3);
            Set(0x0F, "BRK");

            Set(0x10, "BPL", "r1", 2);
            Set(0x1A, "DECW", "d0", 2);
            Set(0x1B, "ASL", "d0+X", 2);
            Set(0x1C, "ASL", "A");
            Set(0x1D, "DEC", "X");
            Set(0x1E, "CMP", "X, aa", 3);
            Set(0x1F, "JMP", "[aa+X]", 3);

            Set(0x20, "CLRP");
            Set(0x2A, "OR1", "C, /mm", 3);
            Set(0x2B, "ROL", "d0", 2);
            Set(0x2C, "ROL", "aa", 3);
            Set(0x2D, "PUSH", "A");
            Set(0x2E, "CBNE", "d0, r2", 3);
            Set(0x2F, "BRA", "r1", 2);

            Set(0x30, "BMI", "r1", 2);
            Set(0x3A, "INCW", "d0", 2);
            Set(0x3B, "ROL", "d0+X", 2);
            Set(0x3C, "ROL", "A");
            Set(0x3D, "INC", "X");
            Set(0x3E, "CMP", "X, d0", 2);
            Set(0x3F, "CALL", "aa", 3);

            Set(0x40, "SETP");
            Set(0x4A, "AND1", "C, mm", 3);
            Set(0x4B, "LSR", "d0", 2);
            Set(0x4C, "LSR", "aa", 3);
            Set(0x4D, "PUSH", "X");
            Set(0x4E, "TCLR1", "aa", 3);
            Set(0x4F, "PCALL", "uu", 2);

            Set(0x50, "BVC", "r1", 2);
            Set(0x5A, "CMPW", "YA, d0", 2);
            Set(0x5B, "LSR", "d0+X", 2);
            Set(0x5C, "LSR", "A");
            Set(0x5D, "MOV", "X, A");
            Set(0x5E, "CMP", "Y, aa", 3);
            Set(0x5F, "JMP", "aa", 3);

            Set(0x60, "CLRC");
            Set(0x6A, "AND1", "C, /mm", 3);
            Set(0x6B, "ROR", "d0", 2);
            Set(0x6C, "ROR", "aa", 3);
            Set(0x6D, "PUSH", "Y");
            Set(0x6E, "DBNZ", "d0, r2", 3);
            Set(0x6F, "RET");

            Set(0x70, "BVS", "r1", 2);
            Set(0x7A, "ADDW", "YA, d0", 2);
            Set(0x7B, "ROR", "d0+X", 2);
            Set(0x7C, "ROR", "A");
            Set(0x7D, "MOV", "A, X");
            Set(0x7E, "CMP", "Y, d0", 2);
            Set(0x7F, "RETI");

            Set(0x80, "SETC");
            Set(0x8A, "EOR1", "C, mm", 3);
            Set(0x8B, "DEC", "d0", 2);
            Set(0x8C, "DEC", "aa", 3);
            Set(0x8D, "MOV", "Y, i0", 2);
            Set(0x8E, "POP", "PSW");
            Set(0x8F, "MOV", "d1, i0", 3);

            Set(0x90, "BCC", "r1", 2);
            Set(0x9A, "SUBW", "YA, d0", 2);
            Set(0x9B, "DEC", "d0+X", 2);
            Set(0x9C, "DEC", "A");
            Set(0x9D, "MOV", "X, SP");
            Set(0x9E, "DIV", "YA, X");
            Set(0x9F, "XCN", "A");

            Set(0xA0, "EI");
            Set(0xAA, "MOV1", "C, mm", 3);
            Set(0xAB, "INC", "d0", 2);
            Set(0xAC, "INC", "aa", 3);
            Set(0xAD, "CMP", "Y, i0", 2);
            Set(0xAE, "POP", "A");
            Set(0xAF, "MOV", "(X)+, A");

            Set(0xB0, "BCS", "r1", 2);
            Set(0xBA, "MOVW", "YA, d0", 2);
            Set(0xBB, "INC", "d0+X", 2);
            Set(0xBC, "INC", "A");
            Set(0xBD, "MOV", "SP, X");
            Set(0xBE, "DAS", "A");
            Set(0xBF, "MOV", "A, (X)+");

            Set(0xC0, "DI");
            Set(0xC4, "MOV", "d0, A", 2);
            Set(0xC5, "MOV", "aa, A", 3);
            Set(0xC6, "MOV", "(X), A");
            Set(0xC7, "MOV", "[d0+X], A", 2);
            Set(0xC8, "CMP", "X, i0", 2);
            Set(0xC9, "MOV", "aa, X", 3);
            Set(0xCA, "MOV1", "mm, C", 3);
            Set(0xCB, "MOV", "d0, Y", 2);
            Set(0xCC, "MOV", "aa, Y", 3);
            Set(0xCD, "MOV", "X, i0", 2);
            Set(0xCE, "POP", "X");
            Set(0xCF, "MUL", "YA");

            Set(0xD0, "BNE", "r1", 2);
            Set(0xD4, "MOV", "d0+X, A", 2);
            Set(0xD5, "MOV", "aa+X, A", 3);
            Set(0xD6, "MOV", "aa+Y, A", 3);
            Set(0xD7, "MOV", "[d0]+Y, A", 2);
            Set(0xD8, "MOV", "d0, X", 2);
            Set(0xD9, "MOV", "d0+Y, X", 2);
            Set(0xDA, "MOVW", "d0, YA", 2);
            Set(0xDB, "MOV", "d0+X, Y", 2);
            Set(0xDC, "DEC", "Y");
            Set(0xDD, "MOV", "A, Y");
            Set(0xDE, "CBNE", "d0+X, r2", 3);
            Set(0xDF, "DAA", "A");

            Set(0xE0, "CLRV");
            Set(0xE4, "MOV", "A, d0", 2);
            Set(0xE5, "MOV", "A, aa", 3);
            Set(0xE6, "MOV", "A, (X)");
            Set(0xE7, "MOV", "A, [d0+X]", 2);
            Set(0xE8, "MOV", "A, i0", 2);
            Set(0xE9, "MOV", "X, aa", 3);
            Set(0xEA, "NOT1", "mm", 3);
            Set(0xEB, "MOV", "Y, d0", 2);
            Set(0xEC, "MOV", "Y, aa", 3);
            Set(0xED, "NOTC");
            Set(0xEE, "POP", "Y");
            Set(0xEF, "SLEEP");

            Set(0xF0, "BEQ", "r1", 2);
            Set(0xF4, "MOV", "A, d0+X", 2);
            Set(0xF5, "MOV", "A, aa+X", 3);
            Set(0xF6, "MOV", "A, aa+Y", 3);
            Set(0xF7, "MOV", "A, [d0]+Y", 2);
            Set(0xF8, "MOV", "X, d0", 2);
            Set(0xF9, "MOV", "X, d0+Y", 2);
            Set(0xFA, "MOV", "d1, d0", 3);
            Set(0xFB, "MOV", "Y, d0+X", 2);
            Set(0xFC, "INC", "Y");
            Set(0xFD, "MOV", "Y, A");
            Set(0xFE, "DBNZ", "Y, r1", 2);
            Set(0xFF, "STOP");

            return t;
        }
    }
}
