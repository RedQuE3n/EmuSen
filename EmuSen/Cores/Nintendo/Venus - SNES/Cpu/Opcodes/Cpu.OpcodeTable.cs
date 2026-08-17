using System;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    public partial class Cpu
    {
        // Mnemonics for the verbose trace only - see Venus_CPU.md §9.
        private static readonly string[] OpcodeNames =
        {
            /* 0x0_ */ "BRK", "ORA", "COP", "ORA", "TSB", "ORA", "ASL", "ORA", "PHP", "ORA", "ASL A", "PHD", "TSB", "ORA", "ASL", "ORA",
            /* 0x1_ */ "BPL", "ORA", "ORA", "ORA", "TRB", "ORA", "ASL", "ORA", "CLC", "ORA", "INC A", "TCS", "TRB", "ORA", "ASL", "ORA",
            /* 0x2_ */ "JSR", "AND", "JSL", "AND", "BIT", "AND", "ROL", "AND", "PLP", "AND", "ROL A", "PLD", "BIT", "AND", "ROL", "AND",
            /* 0x3_ */ "BMI", "AND", "AND", "AND", "BIT", "AND", "ROL", "AND", "SEC", "AND", "DEC A", "TSC", "BIT", "AND", "ROL", "AND",
            /* 0x4_ */ "RTI", "EOR", "WDM", "EOR", "MVP", "EOR", "LSR", "EOR", "PHA", "EOR", "LSR A", "PHK", "JMP", "EOR", "LSR", "EOR",
            /* 0x5_ */ "BVC", "EOR", "EOR", "EOR", "MVN", "EOR", "LSR", "EOR", "CLI", "EOR", "PHY", "TCD", "JML", "EOR", "LSR", "EOR",
            /* 0x6_ */ "RTS", "ADC", "PER", "ADC", "STZ", "ADC", "ROR", "ADC", "PLA", "ADC", "ROR A", "RTL", "JMP", "ADC", "ROR", "ADC",
            /* 0x7_ */ "BVS", "ADC", "ADC", "ADC", "STZ", "ADC", "ROR", "ADC", "SEI", "ADC", "PLY", "TDC", "JMP", "ADC", "ROR", "ADC",
            /* 0x8_ */ "BRA", "STA", "BRL", "STA", "STY", "STA", "STX", "STA", "DEY", "BIT", "TXA", "PHB", "STY", "STA", "STX", "STA",
            /* 0x9_ */ "BCC", "STA", "STA", "STA", "STY", "STA", "STX", "STA", "TYA", "STA", "TXS", "TXY", "STZ", "STA", "STZ", "STA",
            /* 0xA_ */ "LDY", "LDA", "LDX", "LDA", "LDY", "LDA", "LDX", "LDA", "TAY", "LDA", "TAX", "PLB", "LDY", "LDA", "LDX", "LDA",
            /* 0xB_ */ "BCS", "LDA", "LDA", "LDA", "LDY", "LDA", "LDX", "LDA", "CLV", "LDA", "TSX", "TYX", "LDY", "LDA", "LDX", "LDA",
            /* 0xC_ */ "CPY", "CMP", "REP", "CMP", "CPY", "CMP", "DEC", "CMP", "INY", "CMP", "DEX", "WAI", "CPY", "CMP", "DEC", "CMP",
            /* 0xD_ */ "BNE", "CMP", "CMP", "CMP", "PEI", "CMP", "DEC", "CMP", "CLD", "CMP", "PHX", "STP", "JML", "CMP", "DEC", "CMP",
            /* 0xE_ */ "CPX", "SBC", "SEP", "SBC", "CPX", "SBC", "INC", "SBC", "INX", "SBC", "NOP", "XBA", "CPX", "SBC", "INC", "SBC",
            /* 0xF_ */ "BEQ", "SBC", "SBC", "SBC", "PEA", "SBC", "INC", "SBC", "SED", "SBC", "PLX", "XCE", "JSR", "SBC", "INC", "SBC",
        };

        // Base access count per opcode, before _addrModeExtraCycles - see Venus_CPU.md §8.1.
        private static readonly byte[] OpcodeCycles =
        {
            /* 0x0_ */ 7, 6, 7, 4, 5, 3, 5, 6, 3, 2, 2, 4, 6, 4, 6, 5,
            /* 0x1_ */ 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 2, 2, 6, 4, 7, 5,
            /* 0x2_ */ 6, 6, 8, 4, 3, 3, 5, 6, 4, 2, 2, 5, 4, 4, 6, 5,
            /* 0x3_ */ 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 2, 2, 4, 4, 7, 5,
            /* 0x4_ */ 6, 6, 2, 4, 7, 3, 5, 6, 3, 2, 2, 3, 3, 4, 6, 5,
            /* 0x5_ */ 2, 5, 5, 7, 7, 4, 6, 6, 2, 4, 3, 2, 4, 4, 7, 5,
            /* 0x6_ */ 6, 6, 6, 4, 3, 3, 5, 6, 4, 2, 2, 6, 5, 4, 6, 5,
            /* 0x7_ */ 2, 5, 5, 7, 4, 4, 6, 6, 2, 4, 4, 2, 6, 4, 7, 5,
            /* 0x8_ */ 3, 6, 4, 4, 3, 3, 3, 6, 2, 2, 2, 3, 4, 4, 4, 5,
            /* 0x9_ */ 2, 6, 5, 7, 4, 4, 4, 6, 2, 5, 2, 2, 4, 5, 5, 5,
            /* 0xA_ */ 2, 6, 2, 4, 3, 3, 3, 6, 2, 2, 2, 4, 4, 4, 4, 5,
            /* 0xB_ */ 2, 5, 5, 7, 4, 4, 4, 6, 2, 4, 2, 2, 4, 4, 4, 5,
            /* 0xC_ */ 2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5,
            /* 0xD_ */ 2, 5, 5, 7, 6, 4, 6, 6, 2, 4, 3, 3, 6, 4, 7, 5,
            /* 0xE_ */ 2, 6, 3, 4, 3, 3, 5, 6, 2, 2, 2, 3, 4, 4, 6, 5,
            /* 0xF_ */ 2, 5, 5, 7, 5, 4, 6, 6, 2, 4, 4, 2, 8, 4, 7, 5,
        };

        // OpcodeCycles above counts an 8-bit operand - see Venus_CPU.md §8.8.
        public const byte WidthNone = 0, WidthM = 1, WidthX = 2, WidthMRmw = 3;

        private static readonly byte[] OpcodeWidth = BuildWidthTable();

        private static byte[] BuildWidthTable()
        {
            var w = new byte[256];

            // Follow the M flag: one extra cycle when the accumulator is 16-bit.
            foreach (byte op in new byte[]
            {
                0x01,0x03,0x05,0x07,0x09,0x0D,0x0F,0x11,0x12,0x13,0x15,0x17,0x19,0x1D,0x1F, // ORA
                0x21,0x23,0x25,0x27,0x29,0x2D,0x2F,0x31,0x32,0x33,0x35,0x37,0x39,0x3D,0x3F, // AND
                0x41,0x43,0x45,0x47,0x49,0x4D,0x4F,0x51,0x52,0x53,0x55,0x57,0x59,0x5D,0x5F, // EOR
                0x61,0x63,0x65,0x67,0x69,0x6D,0x6F,0x71,0x72,0x73,0x75,0x77,0x79,0x7D,0x7F, // ADC
                0x81,0x83,0x85,0x87,0x8D,0x8F,0x91,0x92,0x93,0x95,0x97,0x99,0x9D,0x9F,      // STA
                0xA1,0xA3,0xA5,0xA7,0xA9,0xAD,0xAF,0xB1,0xB2,0xB3,0xB5,0xB7,0xB9,0xBD,0xBF, // LDA
                0xC1,0xC3,0xC5,0xC7,0xC9,0xCD,0xCF,0xD1,0xD2,0xD3,0xD5,0xD7,0xD9,0xDD,0xDF, // CMP
                0xE1,0xE3,0xE5,0xE7,0xE9,0xED,0xEF,0xF1,0xF2,0xF3,0xF5,0xF7,0xF9,0xFD,0xFF, // SBC
                0x24,0x2C,0x34,0x3C,0x89,                                                   // BIT
                0x64,0x74,0x9C,0x9E,                                                        // STZ
                0x48,0x68,                                                                  // PHA/PLA
            }) w[op] = WidthM;

            // Read-modify-write reads and writes the operand, so 16-bit costs two.
            foreach (byte op in new byte[]
            {
                0x06,0x0E,0x16,0x1E, // ASL
                0x46,0x4E,0x56,0x5E, // LSR
                0x26,0x2E,0x36,0x3E, // ROL
                0x66,0x6E,0x76,0x7E, // ROR
                0xC6,0xCE,0xD6,0xDE, // DEC
                0xE6,0xEE,0xF6,0xFE, // INC
                0x04,0x0C,0x14,0x1C, // TSB/TRB
            }) w[op] = WidthMRmw;

            // Follow the X flag instead; the accumulator's width is irrelevant here.
            foreach (byte op in new byte[]
            {
                0xA2,0xA6,0xAE,0xB6,0xBE, // LDX
                0xA0,0xA4,0xAC,0xB4,0xBC, // LDY
                0x86,0x8E,0x96,           // STX
                0x84,0x8C,0x94,           // STY
                0xE0,0xE4,0xEC,           // CPX
                0xC0,0xC4,0xCC,           // CPY
                0xDA,0xFA,0x5A,0x7A,      // PHX/PLX/PHY/PLY
            }) w[op] = WidthX;

            return w;
        }

        // An internal cycle touches no bus, so it is always 6 master clocks whatever region the operand lives - see Venus_CPU.md §8.8.
        public const int InternalCycleClocks = 6;

        // Opcodes whose every non-fetch cycle is internal: register-only work and branches.
        private static readonly bool[] OpcodeInternalRemainder = BuildInternalRemainder();

        // A taken branch moves PC, so the usual "how far did PC advance" byte count is meaningless; these.
        private static readonly byte[] OpcodeFixedBytes = BuildFixedBytes();

        private static bool[] BuildInternalRemainder()
        {
            var t = new bool[256];
            foreach (byte op in new byte[]
            {
                0x18,0x38,0x58,0x78,0xB8,0xD8,0xF8,      // CLC SEC CLI SEI CLV CLD SED
                0xC2,0xE2,                               // REP SEP
                0xAA,0xA8,0xBA,0x8A,0x9A,0x98,0x9B,0xBB, // TAX TAY TSX TXA TXS TYA TXY TYX
                0x1B,0x3B,0x5B,0x7B,                     // TCS TSC TCD TDC
                0xE8,0xC8,0xCA,0x88,0x1A,0x3A,           // INX INY DEX DEY INC A DEC A
                0x0A,0x4A,0x2A,0x6A,                     // ASL A LSR A ROL A ROR A
                0xEB,0xFB,0xEA,0x42,                     // XBA XCE NOP WDM
                0x10,0x30,0x50,0x70,0x90,0xB0,0xD0,0xF0, // conditional branches
                0x80,0x82,                               // BRA BRL
            }) t[op] = true;
            return t;
        }

        private static byte[] BuildFixedBytes()
        {
            var t = new byte[256];
            foreach (byte op in new byte[] { 0x10,0x30,0x50,0x70,0x90,0xB0,0xD0,0xF0,0x80 }) t[op] = 2;
            t[0x82] = 3; // BRL's offset is 16-bit
            return t;
        }

        // Direct-call dispatch in opcode order - see Venus_CPU.md §9.
        private uint Dispatch(byte opcode)
        {
            switch (opcode)
            {
                case 0x00: { uint a = AddrImplied(); OpBRK(a); return a; } // BRK
                case 0x01: { uint a = AddrDirectIndirectX(); OpORA(a); return a; } // ORA
                case 0x02: { uint a = AddrImplied(); OpCOP(a); return a; } // COP
                case 0x03: { uint a = AddrStackRelative(); OpORA(a); return a; } // ORA
                case 0x04: { uint a = AddrDirectPage(); OpTSB(a); return a; } // TSB
                case 0x05: { uint a = AddrDirectPage(); OpORA(a); return a; } // ORA
                case 0x06: { uint a = AddrDirectPage(); OpASL(a); return a; } // ASL
                case 0x07: { uint a = AddrDirectIndirectLong(); OpORA(a); return a; } // ORA
                case 0x08: { uint a = AddrImplied(); OpPHP(a); return a; } // PHP
                case 0x09: { uint a = AddrImmediateM(); OpORA(a); return a; } // ORA
                case 0x0A: { uint a = AddrImplied(); OpASLA(a); return a; } // ASL A
                case 0x0B: { uint a = AddrImplied(); OpPHD(a); return a; } // PHD
                case 0x0C: { uint a = AddrAbsolute(); OpTSB(a); return a; } // TSB
                case 0x0D: { uint a = AddrAbsolute(); OpORA(a); return a; } // ORA
                case 0x0E: { uint a = AddrAbsolute(); OpASL(a); return a; } // ASL
                case 0x0F: { uint a = AddrAbsoluteLong(); OpORA(a); return a; } // ORA
                case 0x10: { uint a = AddrRelative(); OpBPL(a); return a; } // BPL
                case 0x11: { uint a = AddrDirectIndirectY(); OpORA(a); return a; } // ORA
                case 0x12: { uint a = AddrDirectIndirect(); OpORA(a); return a; } // ORA
                // (sr,S),Y cycle counts resolved against two sources - see Venus_CPU.md §9.1.
                case 0x13: { uint a = AddrStackRelativeIndirectY(); OpORA(a); return a; } // ORA
                case 0x14: { uint a = AddrDirectPage(); OpTRB(a); return a; } // TRB
                case 0x15: { uint a = AddrDirectPageX(); OpORA(a); return a; } // ORA
                case 0x16: { uint a = AddrDirectPageX(); OpASL(a); return a; } // ASL
                case 0x17: { uint a = AddrDirectIndirectLongY(); OpORA(a); return a; } // ORA
                case 0x18: { uint a = AddrImplied(); OpCLC(a); return a; } // CLC
                case 0x19: { uint a = AddrAbsoluteY(); OpORA(a); return a; } // ORA
                case 0x1A: { uint a = AddrImplied(); OpINCA(a); return a; } // INC A
                case 0x1B: { uint a = AddrImplied(); OpTCS(a); return a; } // TCS
                case 0x1C: { uint a = AddrAbsolute(); OpTRB(a); return a; } // TRB
                case 0x1D: { uint a = AddrAbsoluteX(); OpORA(a); return a; } // ORA
                case 0x1E: { uint a = AddrAbsoluteX(); OpASL(a); return a; } // ASL
                case 0x1F: { uint a = AddrAbsoluteLongX(); OpORA(a); return a; } // ORA
                case 0x20: { uint a = AddrAbsolutePB(); OpJSR(a); return a; } // JSR
                case 0x21: { uint a = AddrDirectIndirectX(); OpAND(a); return a; } // AND
                case 0x22: { uint a = AddrAbsoluteLong(); OpJSL(a); return a; } // JSL
                case 0x23: { uint a = AddrStackRelative(); OpAND(a); return a; } // AND
                case 0x24: { uint a = AddrDirectPage(); OpBIT(a); return a; } // BIT
                case 0x25: { uint a = AddrDirectPage(); OpAND(a); return a; } // AND
                case 0x26: { uint a = AddrDirectPage(); OpROLMem(a); return a; } // ROL
                case 0x27: { uint a = AddrDirectIndirectLong(); OpAND(a); return a; } // AND
                case 0x28: { uint a = AddrImplied(); OpPLP(a); return a; } // PLP
                case 0x29: { uint a = AddrImmediateM(); OpAND(a); return a; } // AND
                case 0x2A: { uint a = AddrImplied(); OpROLA(a); return a; } // ROL A
                case 0x2B: { uint a = AddrImplied(); OpPLD(a); return a; } // PLD
                case 0x2C: { uint a = AddrAbsolute(); OpBIT(a); return a; } // BIT
                case 0x2D: { uint a = AddrAbsolute(); OpAND(a); return a; } // AND
                case 0x2E: { uint a = AddrAbsolute(); OpROLMem(a); return a; } // ROL
                case 0x2F: { uint a = AddrAbsoluteLong(); OpAND(a); return a; } // AND
                case 0x30: { uint a = AddrRelative(); OpBMI(a); return a; } // BMI
                case 0x31: { uint a = AddrDirectIndirectY(); OpAND(a); return a; } // AND
                case 0x32: { uint a = AddrDirectIndirect(); OpAND(a); return a; } // AND
                case 0x33: { uint a = AddrStackRelativeIndirectY(); OpAND(a); return a; } // AND
                case 0x34: { uint a = AddrDirectPageX(); OpBIT(a); return a; } // BIT
                case 0x35: { uint a = AddrDirectPageX(); OpAND(a); return a; } // AND
                case 0x36: { uint a = AddrDirectPageX(); OpROLMem(a); return a; } // ROL
                case 0x37: { uint a = AddrDirectIndirectLongY(); OpAND(a); return a; } // AND
                case 0x38: { uint a = AddrImplied(); OpSEC(a); return a; } // SEC
                case 0x39: { uint a = AddrAbsoluteY(); OpAND(a); return a; } // AND
                case 0x3A: { uint a = AddrImplied(); OpDECA(a); return a; } // DEC A
                case 0x3B: { uint a = AddrImplied(); OpTSC(a); return a; } // TSC
                case 0x3C: { uint a = AddrAbsoluteX(); OpBIT(a); return a; } // BIT
                case 0x3D: { uint a = AddrAbsoluteX(); OpAND(a); return a; } // AND
                case 0x3E: { uint a = AddrAbsoluteX(); OpROLMem(a); return a; } // ROL
                case 0x3F: { uint a = AddrAbsoluteLongX(); OpAND(a); return a; } // AND
                case 0x40: { uint a = AddrImplied(); OpRTI(a); return a; } // RTI
                case 0x41: { uint a = AddrDirectIndirectX(); OpEOR(a); return a; } // EOR
                // WDM is a 2-byte NOP on real silicon - see Venus_CPU.md §9.1.
                case 0x42: { uint a = AddrImmediate8(); OpWDM(a); return a; } // WDM
                case 0x43: { uint a = AddrStackRelative(); OpEOR(a); return a; } // EOR
                case 0x44: { uint a = AddrBlockMove(); OpMVP(a); return a; } // MVP
                case 0x45: { uint a = AddrDirectPage(); OpEOR(a); return a; } // EOR
                case 0x46: { uint a = AddrDirectPage(); OpLSR(a); return a; } // LSR
                case 0x47: { uint a = AddrDirectIndirectLong(); OpEOR(a); return a; } // EOR
                case 0x48: { uint a = AddrImplied(); OpPHA(a); return a; } // PHA
                case 0x49: { uint a = AddrImmediateM(); OpEOR(a); return a; } // EOR
                case 0x4A: { uint a = AddrImplied(); OpLSRA(a); return a; } // LSR A
                case 0x4B: { uint a = AddrImplied(); OpPHK(a); return a; } // PHK
                case 0x4C: { uint a = AddrAbsolutePB(); OpJMP(a); return a; } // JMP
                case 0x4D: { uint a = AddrAbsolute(); OpEOR(a); return a; } // EOR
                case 0x4E: { uint a = AddrAbsolute(); OpLSR(a); return a; } // LSR
                case 0x4F: { uint a = AddrAbsoluteLong(); OpEOR(a); return a; } // EOR
                case 0x50: { uint a = AddrRelative(); OpBVC(a); return a; } // BVC
                case 0x51: { uint a = AddrDirectIndirectY(); OpEOR(a); return a; } // EOR
                case 0x52: { uint a = AddrDirectIndirect(); OpEOR(a); return a; } // EOR
                case 0x53: { uint a = AddrStackRelativeIndirectY(); OpEOR(a); return a; } // EOR
                case 0x54: { uint a = AddrBlockMove(); OpMVN(a); return a; } // MVN
                case 0x55: { uint a = AddrDirectPageX(); OpEOR(a); return a; } // EOR
                case 0x56: { uint a = AddrDirectPageX(); OpLSR(a); return a; } // LSR
                case 0x57: { uint a = AddrDirectIndirectLongY(); OpEOR(a); return a; } // EOR
                case 0x58: { uint a = AddrImplied(); OpCLI(a); return a; } // CLI
                case 0x59: { uint a = AddrAbsoluteY(); OpEOR(a); return a; } // EOR
                case 0x5A: { uint a = AddrImplied(); OpPHY(a); return a; } // PHY
                case 0x5B: { uint a = AddrImplied(); OpTCD(a); return a; } // TCD
                case 0x5C: { uint a = AddrAbsoluteLong(); OpJML(a); return a; } // JML
                case 0x5D: { uint a = AddrAbsoluteX(); OpEOR(a); return a; } // EOR
                case 0x5E: { uint a = AddrAbsoluteX(); OpLSR(a); return a; } // LSR
                case 0x5F: { uint a = AddrAbsoluteLongX(); OpEOR(a); return a; } // EOR
                case 0x60: { uint a = AddrImplied(); OpRTS(a); return a; } // RTS
                case 0x61: { uint a = AddrDirectIndirectX(); OpADC(a); return a; } // ADC
                case 0x62: { uint a = AddrImplied(); OpPER(a); return a; } // PER
                case 0x63: { uint a = AddrStackRelative(); OpADC(a); return a; } // ADC
                case 0x64: { uint a = AddrDirectPage(); OpSTZ(a); return a; } // STZ
                case 0x65: { uint a = AddrDirectPage(); OpADC(a); return a; } // ADC
                case 0x66: { uint a = AddrDirectPage(); OpRORMem(a); return a; } // ROR
                case 0x67: { uint a = AddrDirectIndirectLong(); OpADC(a); return a; } // ADC
                case 0x68: { uint a = AddrImplied(); OpPLA(a); return a; } // PLA
                case 0x69: { uint a = AddrImmediateM(); OpADC(a); return a; } // ADC
                case 0x6A: { uint a = AddrImplied(); OpRORA(a); return a; } // ROR A
                case 0x6B: { uint a = AddrImplied(); OpRTL(a); return a; } // RTL
                case 0x6C: { uint a = AddrAbsoluteIndirect(); OpJMP(a); return a; } // JMP
                case 0x6D: { uint a = AddrAbsolute(); OpADC(a); return a; } // ADC
                case 0x6E: { uint a = AddrAbsolute(); OpRORMem(a); return a; } // ROR
                case 0x6F: { uint a = AddrAbsoluteLong(); OpADC(a); return a; } // ADC
                case 0x70: { uint a = AddrRelative(); OpBVS(a); return a; } // BVS
                case 0x71: { uint a = AddrDirectIndirectY(); OpADC(a); return a; } // ADC
                case 0x72: { uint a = AddrDirectIndirect(); OpADC(a); return a; } // ADC
                case 0x73: { uint a = AddrStackRelativeIndirectY(); OpADC(a); return a; } // ADC
                case 0x74: { uint a = AddrDirectPageX(); OpSTZ(a); return a; } // STZ
                case 0x75: { uint a = AddrDirectPageX(); OpADC(a); return a; } // ADC
                case 0x76: { uint a = AddrDirectPageX(); OpRORMem(a); return a; } // ROR
                case 0x77: { uint a = AddrDirectIndirectLongY(); OpADC(a); return a; } // ADC
                case 0x78: { uint a = AddrImplied(); OpSEI(a); return a; } // SEI
                case 0x79: { uint a = AddrAbsoluteY(); OpADC(a); return a; } // ADC
                case 0x7A: { uint a = AddrImplied(); OpPLY(a); return a; } // PLY
                case 0x7B: { uint a = AddrImplied(); OpTDC(a); return a; } // TDC
                case 0x7C: { uint a = AddrAbsoluteIndexedIndirect(); OpJMP(a); return a; } // JMP
                case 0x7D: { uint a = AddrAbsoluteX(); OpADC(a); return a; } // ADC
                case 0x7E: { uint a = AddrAbsoluteX(); OpRORMem(a); return a; } // ROR
                case 0x7F: { uint a = AddrAbsoluteLongX(); OpADC(a); return a; } // ADC
                case 0x80: { uint a = AddrRelative(); OpBRA(a); return a; } // BRA
                case 0x81: { uint a = AddrDirectIndirectX(); OpSTA(a); return a; } // STA
                case 0x82: { uint a = AddrRelativeLong(); OpBRL(a); return a; } // BRL
                case 0x83: { uint a = AddrStackRelative(); OpSTA(a); return a; } // STA
                case 0x84: { uint a = AddrDirectPage(); OpSTY(a); return a; } // STY
                case 0x85: { uint a = AddrDirectPage(); OpSTA(a); return a; } // STA
                case 0x86: { uint a = AddrDirectPage(); OpSTX(a); return a; } // STX
                // STA [dp] is long-indirect, not (dp) - see Venus_CPU.md §9.1.
                case 0x87: { uint a = AddrDirectIndirectLong(); OpSTA(a); return a; } // STA
                case 0x88: { uint a = AddrImplied(); OpDEY(a); return a; } // DEY
                case 0x89: { uint a = AddrImmediateM(); OpBITImm(a); return a; } // BIT
                case 0x8A: { uint a = AddrImplied(); OpTXA(a); return a; } // TXA
                case 0x8B: { uint a = AddrImplied(); OpPHB(a); return a; } // PHB
                case 0x8C: { uint a = AddrAbsolute(); OpSTY(a); return a; } // STY
                case 0x8D: { uint a = AddrAbsolute(); OpSTA(a); return a; } // STA
                case 0x8E: { uint a = AddrAbsolute(); OpSTX(a); return a; } // STX
                case 0x8F: { uint a = AddrAbsoluteLong(); OpSTA(a); return a; } // STA
                case 0x90: { uint a = AddrRelative(); OpBCC(a); return a; } // BCC
                case 0x91: { uint a = AddrDirectIndirectY(); OpSTA(a); return a; } // STA
                case 0x92: { uint a = AddrDirectIndirect(); OpSTA(a); return a; } // STA
                case 0x93: { uint a = AddrStackRelativeIndirectY(); OpSTA(a); return a; } // STA
                case 0x94: { uint a = AddrDirectPageX(); OpSTY(a); return a; } // STY
                case 0x95: { uint a = AddrDirectPageX(); OpSTA(a); return a; } // STA
                case 0x96: { uint a = AddrDirectPageY(); OpSTX(a); return a; } // STX
                case 0x97: { uint a = AddrDirectIndirectLongY(); OpSTA(a); return a; } // STA
                case 0x98: { uint a = AddrImplied(); OpTYA(a); return a; } // TYA
                case 0x99: { uint a = AddrAbsoluteY(); OpSTA(a); return a; } // STA
                case 0x9A: { uint a = AddrImplied(); OpTXS(a); return a; } // TXS
                case 0x9B: { uint a = AddrImplied(); OpTXY(a); return a; } // TXY
                case 0x9C: { uint a = AddrAbsolute(); OpSTZ(a); return a; } // STZ
                case 0x9D: { uint a = AddrAbsoluteX(); OpSTA(a); return a; } // STA
                case 0x9E: { uint a = AddrAbsoluteX(); OpSTZ(a); return a; } // STZ
                case 0x9F: { uint a = AddrAbsoluteLongX(); OpSTA(a); return a; } // STA
                case 0xA0: { uint a = AddrImmediateX(); OpLDY(a); return a; } // LDY
                case 0xA1: { uint a = AddrDirectIndirectX(); OpLDA(a); return a; } // LDA
                case 0xA2: { uint a = AddrImmediateX(); OpLDX(a); return a; } // LDX
                case 0xA3: { uint a = AddrStackRelative(); OpLDA(a); return a; } // LDA
                case 0xA4: { uint a = AddrDirectPage(); OpLDY(a); return a; } // LDY
                case 0xA5: { uint a = AddrDirectPage(); OpLDA(a); return a; } // LDA
                case 0xA6: { uint a = AddrDirectPage(); OpLDX(a); return a; } // LDX
                case 0xA7: { uint a = AddrDirectIndirectLong(); OpLDA(a); return a; } // LDA
                case 0xA8: { uint a = AddrImplied(); OpTAY(a); return a; } // TAY
                case 0xA9: { uint a = AddrImmediateM(); OpLDA(a); return a; } // LDA
                case 0xAA: { uint a = AddrImplied(); OpTAX(a); return a; } // TAX
                case 0xAB: { uint a = AddrImplied(); OpPLB(a); return a; } // PLB
                case 0xAC: { uint a = AddrAbsolute(); OpLDY(a); return a; } // LDY
                case 0xAD: { uint a = AddrAbsolute(); OpLDA(a); return a; } // LDA
                case 0xAE: { uint a = AddrAbsolute(); OpLDX(a); return a; } // LDX
                case 0xAF: { uint a = AddrAbsoluteLong(); OpLDA(a); return a; } // LDA
                case 0xB0: { uint a = AddrRelative(); OpBCS(a); return a; } // BCS
                case 0xB1: { uint a = AddrDirectIndirectY(); OpLDA(a); return a; } // LDA
                case 0xB2: { uint a = AddrDirectIndirect(); OpLDA(a); return a; } // LDA
                case 0xB3: { uint a = AddrStackRelativeIndirectY(); OpLDA(a); return a; } // LDA
                case 0xB4: { uint a = AddrDirectPageX(); OpLDY(a); return a; } // LDY
                case 0xB5: { uint a = AddrDirectPageX(); OpLDA(a); return a; } // LDA
                case 0xB6: { uint a = AddrDirectPageY(); OpLDX(a); return a; } // LDX
                case 0xB7: { uint a = AddrDirectIndirectLongY(); OpLDA(a); return a; } // LDA
                case 0xB8: { uint a = AddrImplied(); OpCLV(a); return a; } // CLV
                case 0xB9: { uint a = AddrAbsoluteY(); OpLDA(a); return a; } // LDA
                case 0xBA: { uint a = AddrImplied(); OpTSX(a); return a; } // TSX
                case 0xBB: { uint a = AddrImplied(); OpTYX(a); return a; } // TYX
                case 0xBC: { uint a = AddrAbsoluteX(); OpLDY(a); return a; } // LDY
                case 0xBD: { uint a = AddrAbsoluteX(); OpLDA(a); return a; } // LDA
                case 0xBE: { uint a = AddrAbsoluteY(); OpLDX(a); return a; } // LDX
                case 0xBF: { uint a = AddrAbsoluteLongX(); OpLDA(a); return a; } // LDA
                case 0xC0: { uint a = AddrImmediateX(); OpCPY(a); return a; } // CPY
                case 0xC1: { uint a = AddrDirectIndirectX(); OpCMP(a); return a; } // CMP
                case 0xC2: { uint a = AddrImmediate8(); OpREP(a); return a; } // REP
                case 0xC3: { uint a = AddrStackRelative(); OpCMP(a); return a; } // CMP
                case 0xC4: { uint a = AddrDirectPage(); OpCPY(a); return a; } // CPY
                case 0xC5: { uint a = AddrDirectPage(); OpCMP(a); return a; } // CMP
                case 0xC6: { uint a = AddrDirectPage(); OpDECMem(a); return a; } // DEC
                case 0xC7: { uint a = AddrDirectIndirectLong(); OpCMP(a); return a; } // CMP
                case 0xC8: { uint a = AddrImplied(); OpINY(a); return a; } // INY
                case 0xC9: { uint a = AddrImmediateM(); OpCMP(a); return a; } // CMP
                case 0xCA: { uint a = AddrImplied(); OpDEX(a); return a; } // DEX
                case 0xCB: { uint a = AddrImplied(); OpWAI(a); return a; } // WAI
                case 0xCC: { uint a = AddrAbsolute(); OpCPY(a); return a; } // CPY
                case 0xCD: { uint a = AddrAbsolute(); OpCMP(a); return a; } // CMP
                case 0xCE: { uint a = AddrAbsolute(); OpDECMem(a); return a; } // DEC
                case 0xCF: { uint a = AddrAbsoluteLong(); OpCMP(a); return a; } // CMP
                case 0xD0: { uint a = AddrRelative(); OpBNE(a); return a; } // BNE
                case 0xD1: { uint a = AddrDirectIndirectY(); OpCMP(a); return a; } // CMP
                case 0xD2: { uint a = AddrDirectIndirect(); OpCMP(a); return a; } // CMP
                case 0xD3: { uint a = AddrStackRelativeIndirectY(); OpCMP(a); return a; } // CMP
                case 0xD4: { uint a = AddrImplied(); OpPEI(a); return a; } // PEI
                case 0xD5: { uint a = AddrDirectPageX(); OpCMP(a); return a; } // CMP
                case 0xD6: { uint a = AddrDirectPageX(); OpDECMem(a); return a; } // DEC
                case 0xD7: { uint a = AddrDirectIndirectLongY(); OpCMP(a); return a; } // CMP
                case 0xD8: { uint a = AddrImplied(); OpCLD(a); return a; } // CLD
                case 0xD9: { uint a = AddrAbsoluteY(); OpCMP(a); return a; } // CMP
                case 0xDA: { uint a = AddrImplied(); OpPHX(a); return a; } // PHX
                case 0xDB: { uint a = AddrImplied(); OpSTP(a); return a; } // STP
                case 0xDC: { uint a = AddrAbsoluteIndirectLong(); OpJML(a); return a; } // JML
                case 0xDD: { uint a = AddrAbsoluteX(); OpCMP(a); return a; } // CMP
                case 0xDE: { uint a = AddrAbsoluteX(); OpDECMem(a); return a; } // DEC
                case 0xDF: { uint a = AddrAbsoluteLongX(); OpCMP(a); return a; } // CMP
                case 0xE0: { uint a = AddrImmediateX(); OpCPX(a); return a; } // CPX
                case 0xE1: { uint a = AddrDirectIndirectX(); OpSBC(a); return a; } // SBC
                case 0xE2: { uint a = AddrImmediate8(); OpSEP(a); return a; } // SEP
                case 0xE3: { uint a = AddrStackRelative(); OpSBC(a); return a; } // SBC
                case 0xE4: { uint a = AddrDirectPage(); OpCPX(a); return a; } // CPX
                case 0xE5: { uint a = AddrDirectPage(); OpSBC(a); return a; } // SBC
                case 0xE6: { uint a = AddrDirectPage(); OpINCMem(a); return a; } // INC
                case 0xE7: { uint a = AddrDirectIndirectLong(); OpSBC(a); return a; } // SBC
                case 0xE8: { uint a = AddrImplied(); OpINX(a); return a; } // INX
                case 0xE9: { uint a = AddrImmediateM(); OpSBC(a); return a; } // SBC
                case 0xEA: { uint a = AddrImplied(); OpNOP(a); return a; } // NOP
                case 0xEB: { uint a = AddrImplied(); OpXBA(a); return a; } // XBA
                case 0xEC: { uint a = AddrAbsolute(); OpCPX(a); return a; } // CPX
                case 0xED: { uint a = AddrAbsolute(); OpSBC(a); return a; } // SBC
                case 0xEE: { uint a = AddrAbsolute(); OpINCMem(a); return a; } // INC
                case 0xEF: { uint a = AddrAbsoluteLong(); OpSBC(a); return a; } // SBC
                case 0xF0: { uint a = AddrRelative(); OpBEQ(a); return a; } // BEQ
                case 0xF1: { uint a = AddrDirectIndirectY(); OpSBC(a); return a; } // SBC
                case 0xF2: { uint a = AddrDirectIndirect(); OpSBC(a); return a; } // SBC
                case 0xF3: { uint a = AddrStackRelativeIndirectY(); OpSBC(a); return a; } // SBC
                case 0xF4: { uint a = AddrImplied(); OpPEA(a); return a; } // PEA
                case 0xF5: { uint a = AddrDirectPageX(); OpSBC(a); return a; } // SBC
                case 0xF6: { uint a = AddrDirectPageX(); OpINCMem(a); return a; } // INC
                case 0xF7: { uint a = AddrDirectIndirectLongY(); OpSBC(a); return a; } // SBC
                case 0xF8: { uint a = AddrImplied(); OpSED(a); return a; } // SED
                case 0xF9: { uint a = AddrAbsoluteY(); OpSBC(a); return a; } // SBC
                case 0xFA: { uint a = AddrImplied(); OpPLX(a); return a; } // PLX
                case 0xFB: { uint a = AddrImplied(); OpXCE(a); return a; } // XCE
                case 0xFC: { uint a = AddrAbsoluteIndexedIndirect(); OpJSR(a); return a; } // JSR
                case 0xFD: { uint a = AddrAbsoluteX(); OpSBC(a); return a; } // SBC
                case 0xFE: { uint a = AddrAbsoluteX(); OpINCMem(a); return a; } // INC
                case 0xFF: { uint a = AddrAbsoluteLongX(); OpSBC(a); return a; } // SBC
                default: throw new NotImplementedException($"Unimplemented Opcode: 0x{opcode:X2} at PC: 0x{PB:X2}{(PC - 1):X4}");
            }
        }
    }
}
