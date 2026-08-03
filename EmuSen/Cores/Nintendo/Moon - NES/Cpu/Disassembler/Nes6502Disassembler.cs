using System;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    public enum AddressMode : byte
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

    // A second opcode table, independent of the executing one - see Moon_Debug.md §5.
    public static class Nes6502Disassembler
    {
        private const byte Imp = (byte)AddressMode.Implied;
        private const byte Acc = (byte)AddressMode.Accumulator;
        private const byte Imm = (byte)AddressMode.Immediate;
        private const byte Zp = (byte)AddressMode.ZeroPage;
        private const byte Zpx = (byte)AddressMode.ZeroPageX;
        private const byte Zpy = (byte)AddressMode.ZeroPageY;
        private const byte Abs = (byte)AddressMode.Absolute;
        private const byte Abx = (byte)AddressMode.AbsoluteX;
        private const byte Aby = (byte)AddressMode.AbsoluteY;
        private const byte Ind = (byte)AddressMode.Indirect;
        private const byte Izx = (byte)AddressMode.IndexedIndirect;
        private const byte Izy = (byte)AddressMode.IndirectIndexed;
        private const byte Rel = (byte)AddressMode.Relative;

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

        public static AddressMode ModeOf(byte opcode) => (AddressMode)Modes[opcode];

        public static string MnemonicOf(byte opcode) => Cpu.OpcodeNames[opcode];

        public static int LengthOf(byte opcode) => (AddressMode)Modes[opcode] switch
        {
            AddressMode.Implied or AddressMode.Accumulator => 1,
            AddressMode.Absolute or AddressMode.AbsoluteX or AddressMode.AbsoluteY or AddressMode.Indirect => 3,
            _ => 2,
        };

        // <address> is only used to resolve a branch target, which is relative to the next instruction.
        public static string FormatOperand(byte opcode, int address, byte operandLow, byte operandHigh)
        {
            int word = operandLow | (operandHigh << 8);

            return (AddressMode)Modes[opcode] switch
            {
                AddressMode.Implied => "",
                AddressMode.Accumulator => "A",
                AddressMode.Immediate => $"#${operandLow:X2}",
                AddressMode.ZeroPage => $"${operandLow:X2}",
                AddressMode.ZeroPageX => $"${operandLow:X2},X",
                AddressMode.ZeroPageY => $"${operandLow:X2},Y",
                AddressMode.Absolute => $"${word:X4}",
                AddressMode.AbsoluteX => $"${word:X4},X",
                AddressMode.AbsoluteY => $"${word:X4},Y",
                AddressMode.Indirect => $"(${word:X4})",
                AddressMode.IndexedIndirect => $"(${operandLow:X2},X)",
                AddressMode.IndirectIndexed => $"(${operandLow:X2}),Y",
                AddressMode.Relative => $"${(ushort)(address + 2 + (sbyte)operandLow):X4}",
                _ => "",
            };
        }

        // Only the forms whose target is knowable from the bytes alone - see IDebugTarget's own comment.
        public static (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(byte opcode, int operand)
        {
            var mode = (AddressMode)Modes[opcode];
            if (mode != AddressMode.Absolute && mode != AddressMode.ZeroPage) return null;

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
