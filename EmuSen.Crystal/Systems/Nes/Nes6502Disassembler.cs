using System;
using EmuSen.Crystal.Debugger;

namespace EmuSen.Crystal.Systems.Nes
{
    public enum Nes6502AddressMode : byte
    {
        Implied,
        Accumulator,
        Immediate,
        ZeroPage,
        ZeroPageX,
        ZeroPageY,
        Absolute,
        AbsoluteX,
        AbsoluteY,
        Indirect,
        IndexedIndirect,
        IndirectIndexed,
        Relative,
    }

    // The 2A03's instructions as text, independent of any engine's executing table - see Moon_Debug.md §5.
    public static class Nes6502Disassembler
    {
        // The 256 mnemonics, the undocumented ones by their common names - see Moon_CPU.md §6.
        public static readonly string[] OpcodeNames =
        {
            /* 0x0_ */ "BRK", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO", "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
            /* 0x1_ */ "BPL", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO", "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
            /* 0x2_ */ "JSR", "AND", "JAM", "RLA", "BIT", "AND", "ROL", "RLA", "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
            /* 0x3_ */ "BMI", "AND", "JAM", "RLA", "NOP", "AND", "ROL", "RLA", "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
            /* 0x4_ */ "RTI", "EOR", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE", "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
            /* 0x5_ */ "BVC", "EOR", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE", "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
            /* 0x6_ */ "RTS", "ADC", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA", "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
            /* 0x7_ */ "BVS", "ADC", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA", "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
            /* 0x8_ */ "NOP", "STA", "NOP", "SAX", "STY", "STA", "STX", "SAX", "DEY", "NOP", "TXA", "ANE", "STY", "STA", "STX", "SAX",
            /* 0x9_ */ "BCC", "STA", "JAM", "SHA", "STY", "STA", "STX", "SAX", "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "SHA",
            /* 0xA_ */ "LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX", "TAY", "LDA", "TAX", "LXA", "LDY", "LDA", "LDX", "LAX",
            /* 0xB_ */ "BCS", "LDA", "JAM", "LAX", "LDY", "LDA", "LDX", "LAX", "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
            /* 0xC_ */ "CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP", "INY", "CMP", "DEX", "SBX", "CPY", "CMP", "DEC", "DCP",
            /* 0xD_ */ "BNE", "CMP", "JAM", "DCP", "NOP", "CMP", "DEC", "DCP", "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
            /* 0xE_ */ "CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC", "INX", "SBC", "NOP", "USBC", "CPX", "SBC", "INC", "ISC",
            /* 0xF_ */ "BEQ", "SBC", "JAM", "ISC", "NOP", "SBC", "INC", "ISC", "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
        };

        private const byte Imp = (byte)Nes6502AddressMode.Implied;
        private const byte Acc = (byte)Nes6502AddressMode.Accumulator;
        private const byte Imm = (byte)Nes6502AddressMode.Immediate;
        private const byte Zp = (byte)Nes6502AddressMode.ZeroPage;
        private const byte Zpx = (byte)Nes6502AddressMode.ZeroPageX;
        private const byte Zpy = (byte)Nes6502AddressMode.ZeroPageY;
        private const byte Abs = (byte)Nes6502AddressMode.Absolute;
        private const byte Abx = (byte)Nes6502AddressMode.AbsoluteX;
        private const byte Aby = (byte)Nes6502AddressMode.AbsoluteY;
        private const byte Ind = (byte)Nes6502AddressMode.Indirect;
        private const byte Izx = (byte)Nes6502AddressMode.IndexedIndirect;
        private const byte Izy = (byte)Nes6502AddressMode.IndirectIndexed;
        private const byte Rel = (byte)Nes6502AddressMode.Relative;

        private static readonly byte[] Modes =
        {
            /* 0x0_ */ Imp, Izx, Imp, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
            /* 0x1_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
            /* 0x2_ */ Abs, Izx, Imp, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
            /* 0x3_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
            /* 0x4_ */ Imp, Izx, Imp, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Acc, Imm, Abs, Abs, Abs, Abs,
            /* 0x5_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
            /* 0x6_ */ Imp, Izx, Imp, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Acc, Imm, Ind, Abs, Abs, Abs,
            /* 0x7_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
            /* 0x8_ */ Imm, Izx, Imm, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
            /* 0x9_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpy, Zpy, Imp, Aby, Imp, Aby, Abx, Abx, Aby, Aby,
            /* 0xA_ */ Imm, Izx, Imm, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
            /* 0xB_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpy, Zpy, Imp, Aby, Imp, Aby, Abx, Abx, Aby, Aby,
            /* 0xC_ */ Imm, Izx, Imm, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
            /* 0xD_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
            /* 0xE_ */ Imm, Izx, Imm, Izx, Zp,  Zp,  Zp,  Zp,  Imp, Imm, Imp, Imm, Abs, Abs, Abs, Abs,
            /* 0xF_ */ Rel, Izy, Imp, Izy, Zpx, Zpx, Zpx, Zpx, Imp, Aby, Imp, Aby, Abx, Abx, Abx, Abx,
        };

        public static Nes6502AddressMode ModeOf(byte opcode) => (Nes6502AddressMode)Modes[opcode];

        public static string MnemonicOf(byte opcode) => OpcodeNames[opcode];

        public static int LengthOf(byte opcode) => (Nes6502AddressMode)Modes[opcode] switch
        {
            Nes6502AddressMode.Implied or Nes6502AddressMode.Accumulator => 1,
            Nes6502AddressMode.Absolute or Nes6502AddressMode.AbsoluteX or Nes6502AddressMode.AbsoluteY or Nes6502AddressMode.Indirect => 3,
            _ => 2,
        };

        // <address> is only used to resolve a branch target, which is relative to the next instruction.
        public static string FormatOperand(byte opcode, int address, byte operandLow, byte operandHigh)
        {
            int word = operandLow | (operandHigh << 8);

            return (Nes6502AddressMode)Modes[opcode] switch
            {
                Nes6502AddressMode.Implied => "",
                Nes6502AddressMode.Accumulator => "A",
                Nes6502AddressMode.Immediate => $"#${operandLow:X2}",
                Nes6502AddressMode.ZeroPage => $"${operandLow:X2}",
                Nes6502AddressMode.ZeroPageX => $"${operandLow:X2},X",
                Nes6502AddressMode.ZeroPageY => $"${operandLow:X2},Y",
                Nes6502AddressMode.Absolute => $"${word:X4}",
                Nes6502AddressMode.AbsoluteX => $"${word:X4},X",
                Nes6502AddressMode.AbsoluteY => $"${word:X4},Y",
                Nes6502AddressMode.Indirect => $"(${word:X4})",
                Nes6502AddressMode.IndexedIndirect => $"(${operandLow:X2},X)",
                Nes6502AddressMode.IndirectIndexed => $"(${operandLow:X2}),Y",
                Nes6502AddressMode.Relative => $"${(ushort)(address + 2 + (sbyte)operandLow):X4}",
                _ => "",
            };
        }

        // Only the forms whose target is knowable from the bytes alone - see IDebugTarget's own comment.
        public static (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(byte opcode, int operand)
        {
            var mode = (Nes6502AddressMode)Modes[opcode];
            if (mode != Nes6502AddressMode.Absolute && mode != Nes6502AddressMode.ZeroPage) return null;

            if (opcode == 0x20) return (StaticReferenceKind.Call, operand);
            if (opcode == 0x4C) return (StaticReferenceKind.Call, operand);

            return MnemonicOf(opcode) switch
            {
                "STA" or "STX" or "STY" or "SAX" => (StaticReferenceKind.Write, operand),
                "INC" or "DEC" or "ASL" or "LSR" or "ROL" or "ROR" => (StaticReferenceKind.Write, operand),
                "LDA" or "LDX" or "LDY" or "CMP" or "CPX" or "CPY" => (StaticReferenceKind.Read, operand),
                "ADC" or "SBC" or "AND" or "ORA" or "EOR" or "BIT" => (StaticReferenceKind.Read, operand),
                _ => null,
            };
        }
    }
}
