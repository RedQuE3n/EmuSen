namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    // The whole 256-entry decode; cycle counts are emergent, not tabulated - see Moon_CPU.md §2.1.
    public partial class Cpu
    {
        // Mnemonics for tracing and the disassembler; undocumented opcodes use the common Oxyron names.
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

        private void Dispatch(byte opcode)
        {
            switch (opcode)
            {
                case 0x00: OpBRK(); break;
                case 0x01: OpORA(Read(AddrIndexedIndirect())); break;
                case 0x02: OpJAM(); break;
                case 0x03: OpSLO(AddrIndexedIndirect()); break;
                case 0x04: Read(AddrZeroPage()); break;
                case 0x05: OpORA(Read(AddrZeroPage())); break;
                case 0x06: OpASL(AddrZeroPage()); break;
                case 0x07: OpSLO(AddrZeroPage()); break;
                case 0x08: OpPHP(); break;
                case 0x09: OpORA(Read(AddrImmediate())); break;
                case 0x0A: ConsumeImplied(); A = ShiftLeft(A); break;
                case 0x0B: OpANC(Read(AddrImmediate())); break;
                case 0x0C: Read(AddrAbsolute()); break;
                case 0x0D: OpORA(Read(AddrAbsolute())); break;
                case 0x0E: OpASL(AddrAbsolute()); break;
                case 0x0F: OpSLO(AddrAbsolute()); break;

                case 0x10: Branch(!GetFlag(CpuFlags.N)); break;
                case 0x11: OpORA(Read(AddrIndirectIndexed(false))); break;
                case 0x12: OpJAM(); break;
                case 0x13: OpSLO(AddrIndirectIndexed(true)); break;
                case 0x14: Read(AddrZeroPageX()); break;
                case 0x15: OpORA(Read(AddrZeroPageX())); break;
                case 0x16: OpASL(AddrZeroPageX()); break;
                case 0x17: OpSLO(AddrZeroPageX()); break;
                case 0x18: SetFlagOpcode(CpuFlags.C, false); break;
                case 0x19: OpORA(Read(AddrAbsoluteY(false))); break;
                case 0x1A: ConsumeImplied(); break;
                case 0x1B: OpSLO(AddrAbsoluteY(true)); break;
                case 0x1C: Read(AddrAbsoluteX(false)); break;
                case 0x1D: OpORA(Read(AddrAbsoluteX(false))); break;
                case 0x1E: OpASL(AddrAbsoluteX(true)); break;
                case 0x1F: OpSLO(AddrAbsoluteX(true)); break;

                case 0x20: OpJSR(); break;
                case 0x21: OpAND(Read(AddrIndexedIndirect())); break;
                case 0x22: OpJAM(); break;
                case 0x23: OpRLA(AddrIndexedIndirect()); break;
                case 0x24: OpBIT(Read(AddrZeroPage())); break;
                case 0x25: OpAND(Read(AddrZeroPage())); break;
                case 0x26: OpROL(AddrZeroPage()); break;
                case 0x27: OpRLA(AddrZeroPage()); break;
                case 0x28: OpPLP(); break;
                case 0x29: OpAND(Read(AddrImmediate())); break;
                case 0x2A: ConsumeImplied(); A = RotateLeft(A); break;
                case 0x2B: OpANC(Read(AddrImmediate())); break;
                case 0x2C: OpBIT(Read(AddrAbsolute())); break;
                case 0x2D: OpAND(Read(AddrAbsolute())); break;
                case 0x2E: OpROL(AddrAbsolute()); break;
                case 0x2F: OpRLA(AddrAbsolute()); break;

                case 0x30: Branch(GetFlag(CpuFlags.N)); break;
                case 0x31: OpAND(Read(AddrIndirectIndexed(false))); break;
                case 0x32: OpJAM(); break;
                case 0x33: OpRLA(AddrIndirectIndexed(true)); break;
                case 0x34: Read(AddrZeroPageX()); break;
                case 0x35: OpAND(Read(AddrZeroPageX())); break;
                case 0x36: OpROL(AddrZeroPageX()); break;
                case 0x37: OpRLA(AddrZeroPageX()); break;
                case 0x38: SetFlagOpcode(CpuFlags.C, true); break;
                case 0x39: OpAND(Read(AddrAbsoluteY(false))); break;
                case 0x3A: ConsumeImplied(); break;
                case 0x3B: OpRLA(AddrAbsoluteY(true)); break;
                case 0x3C: Read(AddrAbsoluteX(false)); break;
                case 0x3D: OpAND(Read(AddrAbsoluteX(false))); break;
                case 0x3E: OpROL(AddrAbsoluteX(true)); break;
                case 0x3F: OpRLA(AddrAbsoluteX(true)); break;

                case 0x40: OpRTI(); break;
                case 0x41: OpEOR(Read(AddrIndexedIndirect())); break;
                case 0x42: OpJAM(); break;
                case 0x43: OpSRE(AddrIndexedIndirect()); break;
                case 0x44: Read(AddrZeroPage()); break;
                case 0x45: OpEOR(Read(AddrZeroPage())); break;
                case 0x46: OpLSR(AddrZeroPage()); break;
                case 0x47: OpSRE(AddrZeroPage()); break;
                case 0x48: OpPHA(); break;
                case 0x49: OpEOR(Read(AddrImmediate())); break;
                case 0x4A: ConsumeImplied(); A = ShiftRight(A); break;
                case 0x4B: OpALR(Read(AddrImmediate())); break;
                case 0x4C: PC = AddrAbsolute(); break;
                case 0x4D: OpEOR(Read(AddrAbsolute())); break;
                case 0x4E: OpLSR(AddrAbsolute()); break;
                case 0x4F: OpSRE(AddrAbsolute()); break;

                case 0x50: Branch(!GetFlag(CpuFlags.V)); break;
                case 0x51: OpEOR(Read(AddrIndirectIndexed(false))); break;
                case 0x52: OpJAM(); break;
                case 0x53: OpSRE(AddrIndirectIndexed(true)); break;
                case 0x54: Read(AddrZeroPageX()); break;
                case 0x55: OpEOR(Read(AddrZeroPageX())); break;
                case 0x56: OpLSR(AddrZeroPageX()); break;
                case 0x57: OpSRE(AddrZeroPageX()); break;
                case 0x58: SetInterruptDisable(false); break;
                case 0x59: OpEOR(Read(AddrAbsoluteY(false))); break;
                case 0x5A: ConsumeImplied(); break;
                case 0x5B: OpSRE(AddrAbsoluteY(true)); break;
                case 0x5C: Read(AddrAbsoluteX(false)); break;
                case 0x5D: OpEOR(Read(AddrAbsoluteX(false))); break;
                case 0x5E: OpLSR(AddrAbsoluteX(true)); break;
                case 0x5F: OpSRE(AddrAbsoluteX(true)); break;

                case 0x60: OpRTS(); break;
                case 0x61: OpADC(Read(AddrIndexedIndirect())); break;
                case 0x62: OpJAM(); break;
                case 0x63: OpRRA(AddrIndexedIndirect()); break;
                case 0x64: Read(AddrZeroPage()); break;
                case 0x65: OpADC(Read(AddrZeroPage())); break;
                case 0x66: OpROR(AddrZeroPage()); break;
                case 0x67: OpRRA(AddrZeroPage()); break;
                case 0x68: OpPLA(); break;
                case 0x69: OpADC(Read(AddrImmediate())); break;
                case 0x6A: ConsumeImplied(); A = RotateRight(A); break;
                case 0x6B: OpARR(Read(AddrImmediate())); break;
                case 0x6C: OpJMPIndirect(); break;
                case 0x6D: OpADC(Read(AddrAbsolute())); break;
                case 0x6E: OpROR(AddrAbsolute()); break;
                case 0x6F: OpRRA(AddrAbsolute()); break;

                case 0x70: Branch(GetFlag(CpuFlags.V)); break;
                case 0x71: OpADC(Read(AddrIndirectIndexed(false))); break;
                case 0x72: OpJAM(); break;
                case 0x73: OpRRA(AddrIndirectIndexed(true)); break;
                case 0x74: Read(AddrZeroPageX()); break;
                case 0x75: OpADC(Read(AddrZeroPageX())); break;
                case 0x76: OpROR(AddrZeroPageX()); break;
                case 0x77: OpRRA(AddrZeroPageX()); break;
                case 0x78: SetInterruptDisable(true); break;
                case 0x79: OpADC(Read(AddrAbsoluteY(false))); break;
                case 0x7A: ConsumeImplied(); break;
                case 0x7B: OpRRA(AddrAbsoluteY(true)); break;
                case 0x7C: Read(AddrAbsoluteX(false)); break;
                case 0x7D: OpADC(Read(AddrAbsoluteX(false))); break;
                case 0x7E: OpROR(AddrAbsoluteX(true)); break;
                case 0x7F: OpRRA(AddrAbsoluteX(true)); break;

                case 0x80: Read(AddrImmediate()); break;
                case 0x81: Write(AddrIndexedIndirect(), A); break;
                case 0x82: Read(AddrImmediate()); break;
                case 0x83: OpSAX(AddrIndexedIndirect()); break;
                case 0x84: Write(AddrZeroPage(), Y); break;
                case 0x85: Write(AddrZeroPage(), A); break;
                case 0x86: Write(AddrZeroPage(), X); break;
                case 0x87: OpSAX(AddrZeroPage()); break;
                case 0x88: ConsumeImplied(); Y = SetZeroNegative((byte)(Y - 1)); break;
                case 0x89: Read(AddrImmediate()); break;
                case 0x8A: ConsumeImplied(); A = SetZeroNegative(X); break;
                case 0x8B: OpANE(Read(AddrImmediate())); break;
                case 0x8C: Write(AddrAbsolute(), Y); break;
                case 0x8D: Write(AddrAbsolute(), A); break;
                case 0x8E: Write(AddrAbsolute(), X); break;
                case 0x8F: OpSAX(AddrAbsolute()); break;

                case 0x90: Branch(!GetFlag(CpuFlags.C)); break;
                case 0x91: Write(AddrIndirectIndexed(true), A); break;
                case 0x92: OpJAM(); break;
                case 0x93: OpSHA_IndirectY(); break;
                case 0x94: Write(AddrZeroPageX(), Y); break;
                case 0x95: Write(AddrZeroPageX(), A); break;
                case 0x96: Write(AddrZeroPageY(), X); break;
                case 0x97: OpSAX(AddrZeroPageY()); break;
                case 0x98: ConsumeImplied(); A = SetZeroNegative(Y); break;
                case 0x99: Write(AddrAbsoluteY(true), A); break;
                case 0x9A: OpTXS(); break;
                case 0x9B: OpTAS(); break;
                case 0x9C: OpSHY(); break;
                case 0x9D: Write(AddrAbsoluteX(true), A); break;
                case 0x9E: OpSHX(); break;
                case 0x9F: OpSHA_AbsoluteY(); break;

                case 0xA0: OpLDY(Read(AddrImmediate())); break;
                case 0xA1: OpLDA(Read(AddrIndexedIndirect())); break;
                case 0xA2: OpLDX(Read(AddrImmediate())); break;
                case 0xA3: OpLAX(Read(AddrIndexedIndirect())); break;
                case 0xA4: OpLDY(Read(AddrZeroPage())); break;
                case 0xA5: OpLDA(Read(AddrZeroPage())); break;
                case 0xA6: OpLDX(Read(AddrZeroPage())); break;
                case 0xA7: OpLAX(Read(AddrZeroPage())); break;
                case 0xA8: ConsumeImplied(); Y = SetZeroNegative(A); break;
                case 0xA9: OpLDA(Read(AddrImmediate())); break;
                case 0xAA: ConsumeImplied(); X = SetZeroNegative(A); break;
                case 0xAB: OpLXA(Read(AddrImmediate())); break;
                case 0xAC: OpLDY(Read(AddrAbsolute())); break;
                case 0xAD: OpLDA(Read(AddrAbsolute())); break;
                case 0xAE: OpLDX(Read(AddrAbsolute())); break;
                case 0xAF: OpLAX(Read(AddrAbsolute())); break;

                case 0xB0: Branch(GetFlag(CpuFlags.C)); break;
                case 0xB1: OpLDA(Read(AddrIndirectIndexed(false))); break;
                case 0xB2: OpJAM(); break;
                case 0xB3: OpLAX(Read(AddrIndirectIndexed(false))); break;
                case 0xB4: OpLDY(Read(AddrZeroPageX())); break;
                case 0xB5: OpLDA(Read(AddrZeroPageX())); break;
                case 0xB6: OpLDX(Read(AddrZeroPageY())); break;
                case 0xB7: OpLAX(Read(AddrZeroPageY())); break;
                case 0xB8: SetFlagOpcode(CpuFlags.V, false); break;
                case 0xB9: OpLDA(Read(AddrAbsoluteY(false))); break;
                case 0xBA: ConsumeImplied(); X = SetZeroNegative(S); break;
                case 0xBB: OpLAS(Read(AddrAbsoluteY(false))); break;
                case 0xBC: OpLDY(Read(AddrAbsoluteX(false))); break;
                case 0xBD: OpLDA(Read(AddrAbsoluteX(false))); break;
                case 0xBE: OpLDX(Read(AddrAbsoluteY(false))); break;
                case 0xBF: OpLAX(Read(AddrAbsoluteY(false))); break;

                case 0xC0: Compare(Y, Read(AddrImmediate())); break;
                case 0xC1: Compare(A, Read(AddrIndexedIndirect())); break;
                case 0xC2: Read(AddrImmediate()); break;
                case 0xC3: OpDCP(AddrIndexedIndirect()); break;
                case 0xC4: Compare(Y, Read(AddrZeroPage())); break;
                case 0xC5: Compare(A, Read(AddrZeroPage())); break;
                case 0xC6: OpDEC(AddrZeroPage()); break;
                case 0xC7: OpDCP(AddrZeroPage()); break;
                case 0xC8: ConsumeImplied(); Y = SetZeroNegative((byte)(Y + 1)); break;
                case 0xC9: Compare(A, Read(AddrImmediate())); break;
                case 0xCA: ConsumeImplied(); X = SetZeroNegative((byte)(X - 1)); break;
                case 0xCB: OpSBX(Read(AddrImmediate())); break;
                case 0xCC: Compare(Y, Read(AddrAbsolute())); break;
                case 0xCD: Compare(A, Read(AddrAbsolute())); break;
                case 0xCE: OpDEC(AddrAbsolute()); break;
                case 0xCF: OpDCP(AddrAbsolute()); break;

                case 0xD0: Branch(!GetFlag(CpuFlags.Z)); break;
                case 0xD1: Compare(A, Read(AddrIndirectIndexed(false))); break;
                case 0xD2: OpJAM(); break;
                case 0xD3: OpDCP(AddrIndirectIndexed(true)); break;
                case 0xD4: Read(AddrZeroPageX()); break;
                case 0xD5: Compare(A, Read(AddrZeroPageX())); break;
                case 0xD6: OpDEC(AddrZeroPageX()); break;
                case 0xD7: OpDCP(AddrZeroPageX()); break;
                case 0xD8: SetFlagOpcode(CpuFlags.D, false); break;
                case 0xD9: Compare(A, Read(AddrAbsoluteY(false))); break;
                case 0xDA: ConsumeImplied(); break;
                case 0xDB: OpDCP(AddrAbsoluteY(true)); break;
                case 0xDC: Read(AddrAbsoluteX(false)); break;
                case 0xDD: Compare(A, Read(AddrAbsoluteX(false))); break;
                case 0xDE: OpDEC(AddrAbsoluteX(true)); break;
                case 0xDF: OpDCP(AddrAbsoluteX(true)); break;

                case 0xE0: Compare(X, Read(AddrImmediate())); break;
                case 0xE1: OpSBC(Read(AddrIndexedIndirect())); break;
                case 0xE2: Read(AddrImmediate()); break;
                case 0xE3: OpISC(AddrIndexedIndirect()); break;
                case 0xE4: Compare(X, Read(AddrZeroPage())); break;
                case 0xE5: OpSBC(Read(AddrZeroPage())); break;
                case 0xE6: OpINC(AddrZeroPage()); break;
                case 0xE7: OpISC(AddrZeroPage()); break;
                case 0xE8: ConsumeImplied(); X = SetZeroNegative((byte)(X + 1)); break;
                case 0xE9: OpSBC(Read(AddrImmediate())); break;
                case 0xEA: ConsumeImplied(); break;
                case 0xEB: OpSBC(Read(AddrImmediate())); break;
                case 0xEC: Compare(X, Read(AddrAbsolute())); break;
                case 0xED: OpSBC(Read(AddrAbsolute())); break;
                case 0xEE: OpINC(AddrAbsolute()); break;
                case 0xEF: OpISC(AddrAbsolute()); break;

                case 0xF0: Branch(GetFlag(CpuFlags.Z)); break;
                case 0xF1: OpSBC(Read(AddrIndirectIndexed(false))); break;
                case 0xF2: OpJAM(); break;
                case 0xF3: OpISC(AddrIndirectIndexed(true)); break;
                case 0xF4: Read(AddrZeroPageX()); break;
                case 0xF5: OpSBC(Read(AddrZeroPageX())); break;
                case 0xF6: OpINC(AddrZeroPageX()); break;
                case 0xF7: OpISC(AddrZeroPageX()); break;
                case 0xF8: SetFlagOpcode(CpuFlags.D, true); break;
                case 0xF9: OpSBC(Read(AddrAbsoluteY(false))); break;
                case 0xFA: ConsumeImplied(); break;
                case 0xFB: OpISC(AddrAbsoluteY(true)); break;
                case 0xFC: Read(AddrAbsoluteX(false)); break;
                case 0xFD: OpSBC(Read(AddrAbsoluteX(false))); break;
                case 0xFE: OpINC(AddrAbsoluteX(true)); break;
                case 0xFF: OpISC(AddrAbsoluteX(true)); break;
            }
        }
    }
}
