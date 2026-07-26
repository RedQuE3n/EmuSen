using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    public partial class Cpu
    {
        private void BuildOpcodeTable()
        {
            _instructions = new Instruction[256];

            for (int i = 0; i < 256; i++)
            {
                _instructions[i] = new Instruction 
                { 
                    Name = "NOP/UNK", 
                    AddrMode = AddrImplied, 
                    Operate = OpUnknown, 
                    Cycles = 2 
                };
            }

            _instructions[0x08] = new Instruction { Name = "PHP", AddrMode = AddrImplied, Operate = OpPHP, Cycles = 3 };
            _instructions[0x10] = new Instruction { Name = "BPL", AddrMode = AddrRelative, Operate = OpBPL, Cycles = 2 };
            _instructions[0x18] = new Instruction { Name = "CLC", AddrMode = AddrImplied, Operate = OpCLC, Cycles = 2 };
            _instructions[0x1A] = new Instruction { Name = "INC A", AddrMode = AddrImplied, Operate = OpINCA, Cycles = 2 };
            _instructions[0x1B] = new Instruction { Name = "TCS", AddrMode = AddrImplied, Operate = OpTCS, Cycles = 2 };
            _instructions[0x20] = new Instruction { Name = "JSR", AddrMode = AddrAbsolutePB, Operate = OpJSR, Cycles = 6 };
            _instructions[0x28] = new Instruction { Name = "PLP", AddrMode = AddrImplied, Operate = OpPLP, Cycles = 4 };
            _instructions[0x2A] = new Instruction { Name = "ROL A", AddrMode = AddrImplied, Operate = OpROLA, Cycles = 2 };
            _instructions[0x26] = new Instruction { Name = "ROL", AddrMode = AddrDirectPage, Operate = OpROLMem, Cycles = 5 };
            _instructions[0x36] = new Instruction { Name = "ROL", AddrMode = AddrDirectPageX, Operate = OpROLMem, Cycles = 6 };
            _instructions[0x2E] = new Instruction { Name = "ROL", AddrMode = AddrAbsolute, Operate = OpROLMem, Cycles = 6 };
            _instructions[0x3E] = new Instruction { Name = "ROL", AddrMode = AddrAbsoluteX, Operate = OpROLMem, Cycles = 7 };
            _instructions[0x38] = new Instruction { Name = "SEC", AddrMode = AddrImplied, Operate = OpSEC, Cycles = 2 };
            _instructions[0x40] = new Instruction { Name = "RTI", AddrMode = AddrImplied, Operate = OpRTI, Cycles = 6 };
            _instructions[0x48] = new Instruction { Name = "PHA", AddrMode = AddrImplied, Operate = OpPHA, Cycles = 3 };
            
            _instructions[0x58] = new Instruction { Name = "CLI", AddrMode = AddrImplied, Operate = OpCLI, Cycles = 2 };
            _instructions[0x5B] = new Instruction { Name = "TCD", AddrMode = AddrImplied, Operate = OpTCD, Cycles = 2 };
            _instructions[0x60] = new Instruction { Name = "RTS", AddrMode = AddrImplied, Operate = OpRTS, Cycles = 6 };
            _instructions[0x68] = new Instruction { Name = "PLA", AddrMode = AddrImplied, Operate = OpPLA, Cycles = 4 };
            _instructions[0x69] = new Instruction { Name = "ADC", AddrMode = AddrImmediateM, Operate = OpADC, Cycles = 2 };
            _instructions[0x70] = new Instruction { Name = "BVS", AddrMode = AddrRelative, Operate = OpBVS, Cycles = 2 };
            _instructions[0x50] = new Instruction { Name = "BVC", AddrMode = AddrRelative, Operate = OpBVC, Cycles = 2 };
            
            _instructions[0x64] = new Instruction { Name = "STZ", AddrMode = AddrDirectPage, Operate = OpSTZ, Cycles = 3 };
            _instructions[0x74] = new Instruction { Name = "STZ", AddrMode = AddrDirectPageX, Operate = OpSTZ, Cycles = 4 };
            
            _instructions[0x78] = new Instruction { Name = "SEI", AddrMode = AddrImplied, Operate = OpSEI, Cycles = 2 };
            _instructions[0x80] = new Instruction { Name = "BRA", AddrMode = AddrRelative, Operate = OpBRA, Cycles = 3 };
            _instructions[0x82] = new Instruction { Name = "BRL", AddrMode = AddrRelativeLong, Operate = OpBRL, Cycles = 4 };
            _instructions[0x8D] = new Instruction { Name = "STA", AddrMode = AddrAbsolute, Operate = OpSTA, Cycles = 4 };
            _instructions[0x8F] = new Instruction { Name = "STA", AddrMode = AddrAbsoluteLong, Operate = OpSTA, Cycles = 5 };
            // --- Absolute Long ALU Family ---
            _instructions[0xAF] = new Instruction { Name = "LDA", AddrMode = AddrAbsoluteLong, Operate = OpLDA, Cycles = 5 };
            _instructions[0x6F] = new Instruction { Name = "ADC", AddrMode = AddrAbsoluteLong, Operate = OpADC, Cycles = 5 };
            _instructions[0xEF] = new Instruction { Name = "SBC", AddrMode = AddrAbsoluteLong, Operate = OpSBC, Cycles = 5 };
            _instructions[0x2F] = new Instruction { Name = "AND", AddrMode = AddrAbsoluteLong, Operate = OpAND, Cycles = 5 };
            _instructions[0x0F] = new Instruction { Name = "ORA", AddrMode = AddrAbsoluteLong, Operate = OpORA, Cycles = 5 };
            _instructions[0x4F] = new Instruction { Name = "EOR", AddrMode = AddrAbsoluteLong, Operate = OpEOR, Cycles = 5 }; 
            _instructions[0x98] = new Instruction { Name = "TYA", AddrMode = AddrImplied, Operate = OpTYA, Cycles = 2 };
            _instructions[0x9C] = new Instruction { Name = "STZ", AddrMode = AddrAbsolute, Operate = OpSTZ, Cycles = 4 };
            _instructions[0x9F] = new Instruction { Name = "STA", AddrMode = AddrAbsoluteLongX, Operate = OpSTA, Cycles = 5 };
            _instructions[0x97] = new Instruction { Name = "STA", AddrMode = AddrDirectIndirectLongY, Operate = OpSTA, Cycles = 6 };
            // --- Direct Indirect Long Indexed Y Family ( [dp],Y ) ---
            _instructions[0xD7] = new Instruction { Name = "CMP", AddrMode = AddrDirectIndirectLongY, Operate = OpCMP, Cycles = 6 };
            _instructions[0x77] = new Instruction { Name = "ADC", AddrMode = AddrDirectIndirectLongY, Operate = OpADC, Cycles = 6 };
            _instructions[0xF7] = new Instruction { Name = "SBC", AddrMode = AddrDirectIndirectLongY, Operate = OpSBC, Cycles = 6 };
            _instructions[0x37] = new Instruction { Name = "AND", AddrMode = AddrDirectIndirectLongY, Operate = OpAND, Cycles = 6 };
            _instructions[0x17] = new Instruction { Name = "ORA", AddrMode = AddrDirectIndirectLongY, Operate = OpORA, Cycles = 6 };
            _instructions[0x57] = new Instruction { Name = "EOR", AddrMode = AddrDirectIndirectLongY, Operate = OpEOR, Cycles = 6 };
            _instructions[0xA0] = new Instruction { Name = "LDY", AddrMode = AddrImmediateX, Operate = OpLDY, Cycles = 2 };
            _instructions[0xA2] = new Instruction { Name = "LDX", AddrMode = AddrImmediateX, Operate = OpLDX, Cycles = 2 };
            _instructions[0xA8] = new Instruction { Name = "TAY", AddrMode = AddrImplied, Operate = OpTAY, Cycles = 2 };
            _instructions[0xA9] = new Instruction { Name = "LDA", AddrMode = AddrImmediateM, Operate = OpLDA, Cycles = 2 };
            _instructions[0xAA] = new Instruction { Name = "TAX", AddrMode = AddrImplied, Operate = OpTAX, Cycles = 2 };
            _instructions[0xB7] = new Instruction { Name = "LDA", AddrMode = AddrDirectIndirectLongY, Operate = OpLDA, Cycles = 6 };
            _instructions[0xC2] = new Instruction { Name = "REP", AddrMode = AddrImmediate8, Operate = OpREP, Cycles = 3 };
            _instructions[0xC8] = new Instruction { Name = "INY", AddrMode = AddrImplied, Operate = OpINY, Cycles = 2 };
            _instructions[0xCA] = new Instruction { Name = "DEX", AddrMode = AddrImplied, Operate = OpDEX, Cycles = 2 };
            _instructions[0xCD] = new Instruction { Name = "CMP", AddrMode = AddrAbsolute, Operate = OpCMP, Cycles = 4 };
            // --- CMP Memory Family ---
            _instructions[0xC5] = new Instruction { Name = "CMP", AddrMode = AddrDirectPage, Operate = OpCMP, Cycles = 3 };
            _instructions[0xD5] = new Instruction { Name = "CMP", AddrMode = AddrDirectPageX, Operate = OpCMP, Cycles = 4 };
            _instructions[0xCF] = new Instruction { Name = "CMP", AddrMode = AddrAbsoluteLong, Operate = OpCMP, Cycles = 5 };
            _instructions[0xD0] = new Instruction { Name = "BNE", AddrMode = AddrRelative, Operate = OpBNE, Cycles = 2 };
            _instructions[0xE0] = new Instruction { Name = "CPX", AddrMode = AddrImmediateX, Operate = OpCPX, Cycles = 2 };
            // --- CPX / CPY Memory Additions ---
            _instructions[0xE4] = new Instruction { Name = "CPX", AddrMode = AddrDirectPage, Operate = OpCPX, Cycles = 3 };
            _instructions[0xC4] = new Instruction { Name = "CPY", AddrMode = AddrDirectPage, Operate = OpCPY, Cycles = 3 };
            _instructions[0xEC] = new Instruction { Name = "CPX", AddrMode = AddrAbsolute, Operate = OpCPX, Cycles = 4 };
            _instructions[0xE2] = new Instruction { Name = "SEP", AddrMode = AddrImmediate8, Operate = OpSEP, Cycles = 3 };
            _instructions[0xE9] = new Instruction { Name = "SBC", AddrMode = AddrImmediateM, Operate = OpSBC, Cycles = 2 };
            _instructions[0xEB] = new Instruction { Name = "XBA", AddrMode = AddrImplied, Operate = OpXBA, Cycles = 3 };
            _instructions[0xF0] = new Instruction { Name = "BEQ", AddrMode = AddrRelative, Operate = OpBEQ, Cycles = 2 };
            _instructions[0xFB] = new Instruction { Name = "XCE", AddrMode = AddrImplied, Operate = OpXCE, Cycles = 2 };

            // --- Added: fills gaps that virtually any 65816 program needs early on ---
            _instructions[0x09] = new Instruction { Name = "ORA", AddrMode = AddrImmediateM, Operate = OpORA, Cycles = 2 };
            _instructions[0x0A] = new Instruction { Name = "ASL A", AddrMode = AddrImplied, Operate = OpASLA, Cycles = 2 };
            // --- LSR Memory Family ---
            _instructions[0x46] = new Instruction { Name = "LSR", AddrMode = AddrDirectPage, Operate = OpLSR, Cycles = 5 };
            _instructions[0x4E] = new Instruction { Name = "LSR", AddrMode = AddrAbsolute, Operate = OpLSR, Cycles = 6 };
            _instructions[0x56] = new Instruction { Name = "LSR", AddrMode = AddrDirectPageX, Operate = OpLSR, Cycles = 6 };
            _instructions[0x5E] = new Instruction { Name = "LSR", AddrMode = AddrAbsoluteX, Operate = OpLSR, Cycles = 7 };
            _instructions[0x0B] = new Instruction { Name = "PHD", AddrMode = AddrImplied, Operate = OpPHD, Cycles = 4 };
            _instructions[0x0D] = new Instruction { Name = "ORA", AddrMode = AddrAbsolute, Operate = OpORA, Cycles = 4 };
            _instructions[0x05] = new Instruction { Name = "ORA", AddrMode = AddrDirectPage, Operate = OpORA, Cycles = 3 };
            _instructions[0x07] = new Instruction { Name = "ORA", AddrMode = AddrDirectIndirectLong, Operate = OpORA, Cycles = 6 };
            _instructions[0x22] = new Instruction { Name = "JSL", AddrMode = AddrAbsoluteLong, Operate = OpJSL, Cycles = 8 };
            _instructions[0x29] = new Instruction { Name = "AND", AddrMode = AddrImmediateM, Operate = OpAND, Cycles = 2 };
            _instructions[0x2B] = new Instruction { Name = "PLD", AddrMode = AddrImplied, Operate = OpPLD, Cycles = 5 };
            _instructions[0x2D] = new Instruction { Name = "AND", AddrMode = AddrAbsolute, Operate = OpAND, Cycles = 4 };
            _instructions[0x25] = new Instruction { Name = "AND", AddrMode = AddrDirectPage, Operate = OpAND, Cycles = 3 };
            _instructions[0x30] = new Instruction { Name = "BMI", AddrMode = AddrRelative, Operate = OpBMI, Cycles = 2 };
            _instructions[0x4A] = new Instruction { Name = "LSR A", AddrMode = AddrImplied, Operate = OpLSRA, Cycles = 2 };
            _instructions[0x4B] = new Instruction { Name = "PHK", AddrMode = AddrImplied, Operate = OpPHK, Cycles = 3 };
            _instructions[0x4C] = new Instruction { Name = "JMP", AddrMode = AddrAbsolutePB, Operate = OpJMP, Cycles = 3 };
            _instructions[0x4D] = new Instruction { Name = "EOR", AddrMode = AddrAbsolute, Operate = OpEOR, Cycles = 4 };
            _instructions[0x49] = new Instruction { Name = "EOR", AddrMode = AddrImmediateM, Operate = OpEOR, Cycles = 2 };
            _instructions[0x5A] = new Instruction { Name = "PHY", AddrMode = AddrImplied, Operate = OpPHY, Cycles = 3 };
            _instructions[0x5C] = new Instruction { Name = "JML", AddrMode = AddrAbsoluteLong, Operate = OpJML, Cycles = 4 };
            _instructions[0x6B] = new Instruction { Name = "RTL", AddrMode = AddrImplied, Operate = OpRTL, Cycles = 6 };
            _instructions[0x7A] = new Instruction { Name = "PLY", AddrMode = AddrImplied, Operate = OpPLY, Cycles = 4 };
            _instructions[0x85] = new Instruction { Name = "STA", AddrMode = AddrDirectPage, Operate = OpSTA, Cycles = 3 };
            _instructions[0x88] = new Instruction { Name = "DEY", AddrMode = AddrImplied, Operate = OpDEY, Cycles = 2 };
            _instructions[0x8B] = new Instruction { Name = "PHB", AddrMode = AddrImplied, Operate = OpPHB, Cycles = 3 };
            _instructions[0x90] = new Instruction { Name = "BCC", AddrMode = AddrRelative, Operate = OpBCC, Cycles = 2 };
            _instructions[0xA5] = new Instruction { Name = "LDA", AddrMode = AddrDirectPage, Operate = OpLDA, Cycles = 3 };
            _instructions[0xA7] = new Instruction { Name = "LDA", AddrMode = AddrDirectIndirectLong, Operate = OpLDA, Cycles = 6 };
            _instructions[0xAB] = new Instruction { Name = "PLB", AddrMode = AddrImplied, Operate = OpPLB, Cycles = 4 };
            _instructions[0xAD] = new Instruction { Name = "LDA", AddrMode = AddrAbsolute, Operate = OpLDA, Cycles = 4 };
            _instructions[0xB2] = new Instruction { Name = "LDA", AddrMode = AddrDirectIndirect, Operate = OpLDA, Cycles = 5 };
            _instructions[0xB0] = new Instruction { Name = "BCS", AddrMode = AddrRelative, Operate = OpBCS, Cycles = 2 };
            _instructions[0xC0] = new Instruction { Name = "CPY", AddrMode = AddrImmediateX, Operate = OpCPY, Cycles = 2 };
            _instructions[0xCC] = new Instruction { Name = "CPY", AddrMode = AddrAbsolute, Operate = OpCPY, Cycles = 4 };
            _instructions[0xDA] = new Instruction { Name = "PHX", AddrMode = AddrImplied, Operate = OpPHX, Cycles = 3 };
            _instructions[0xE8] = new Instruction { Name = "INX", AddrMode = AddrImplied, Operate = OpINX, Cycles = 2 };
            _instructions[0xEA] = new Instruction { Name = "NOP", AddrMode = AddrImplied, Operate = OpNOP, Cycles = 2 };
            _instructions[0x00] = new Instruction { Name = "BRK", AddrMode = AddrImplied, Operate = OpBRK, Cycles = 7 };
            _instructions[0x02] = new Instruction { Name = "COP", AddrMode = AddrImplied, Operate = OpCOP, Cycles = 7 };
            _instructions[0xFA] = new Instruction { Name = "PLX", AddrMode = AddrImplied, Operate = OpPLX, Cycles = 4 };
            _instructions[0xBF] = new Instruction { Name = "LDA", AddrMode = AddrAbsoluteLongX, Operate = OpLDA, Cycles = 5 };

            // --- Added: Absolute,X / Absolute,Y indexed family ---
            // 0x9E (STZ abs,X) is what the ROM halted on after the APU upload finished;
            // the rest share the same new addressing modes and are among the most common
            // 65816 instructions, so they're near-certain to be needed immediately after.
            _instructions[0x9E] = new Instruction { Name = "STZ", AddrMode = AddrAbsoluteX, Operate = OpSTZ, Cycles = 5 };
            _instructions[0x9D] = new Instruction { Name = "STA", AddrMode = AddrAbsoluteX, Operate = OpSTA, Cycles = 5 };
            _instructions[0x99] = new Instruction { Name = "STA", AddrMode = AddrAbsoluteY, Operate = OpSTA, Cycles = 5 };
            _instructions[0xBD] = new Instruction { Name = "LDA", AddrMode = AddrAbsoluteX, Operate = OpLDA, Cycles = 4 };
            _instructions[0xB9] = new Instruction { Name = "LDA", AddrMode = AddrAbsoluteY, Operate = OpLDA, Cycles = 4 };
            _instructions[0xBC] = new Instruction { Name = "LDY", AddrMode = AddrAbsoluteX, Operate = OpLDY, Cycles = 4 };
            _instructions[0xBE] = new Instruction { Name = "LDX", AddrMode = AddrAbsoluteY, Operate = OpLDX, Cycles = 4 };
            _instructions[0xDD] = new Instruction { Name = "CMP", AddrMode = AddrAbsoluteX, Operate = OpCMP, Cycles = 4 };
            _instructions[0xC9] = new Instruction { Name = "CMP", AddrMode = AddrImmediateM, Operate = OpCMP, Cycles = 2 };
            _instructions[0xD9] = new Instruction { Name = "CMP", AddrMode = AddrAbsoluteY, Operate = OpCMP, Cycles = 4 };
            _instructions[0x7D] = new Instruction { Name = "ADC", AddrMode = AddrAbsoluteX, Operate = OpADC, Cycles = 4 };
            _instructions[0x6D] = new Instruction { Name = "ADC", AddrMode = AddrAbsolute, Operate = OpADC, Cycles = 4 };
            _instructions[0x65] = new Instruction { Name = "ADC", AddrMode = AddrDirectPage, Operate = OpADC, Cycles = 3 };
            _instructions[0x75] = new Instruction { Name = "ADC", AddrMode = AddrDirectPageX, Operate = OpADC, Cycles = 4 };
            _instructions[0x15] = new Instruction { Name = "ORA", AddrMode = AddrDirectPageX, Operate = OpORA, Cycles = 4 };
            _instructions[0x35] = new Instruction { Name = "AND", AddrMode = AddrDirectPageX, Operate = OpAND, Cycles = 4 };
            _instructions[0x79] = new Instruction { Name = "ADC", AddrMode = AddrAbsoluteY, Operate = OpADC, Cycles = 4 };
            _instructions[0xFD] = new Instruction { Name = "SBC", AddrMode = AddrAbsoluteX, Operate = OpSBC, Cycles = 4 };
            _instructions[0xF9] = new Instruction { Name = "SBC", AddrMode = AddrAbsoluteY, Operate = OpSBC, Cycles = 4 };
            _instructions[0x3D] = new Instruction { Name = "AND", AddrMode = AddrAbsoluteX, Operate = OpAND, Cycles = 4 };
            _instructions[0x39] = new Instruction { Name = "AND", AddrMode = AddrAbsoluteY, Operate = OpAND, Cycles = 4 };
            _instructions[0x1D] = new Instruction { Name = "ORA", AddrMode = AddrAbsoluteX, Operate = OpORA, Cycles = 4 };
            _instructions[0x19] = new Instruction { Name = "ORA", AddrMode = AddrAbsoluteY, Operate = OpORA, Cycles = 4 };
            _instructions[0x5D] = new Instruction { Name = "EOR", AddrMode = AddrAbsoluteX, Operate = OpEOR, Cycles = 4 };
            _instructions[0x59] = new Instruction { Name = "EOR", AddrMode = AddrAbsoluteY, Operate = OpEOR, Cycles = 4 };
            // 0x87 is STA [dp] - the 24-bit long-indirect store (3-byte
            // pointer read from the direct page, no DBR involved at all) -
            // NOT the same addressing mode as 0x92's STA (dp) (16-bit
            // pointer + current DBR). Was wired to AddrDirectIndirect (the
            // short/DBR-relative mode) instead of AddrDirectIndirectLong,
            // the same long-pointer mode LDA's own 0xA7 already uses
            // correctly. Real-world effect: any code doing STA [dp] with
            // DBR != the pointer's own bank byte (common - e.g. ALTTP's
            // AddReceivedItem sets DBR to its own bank via PHK:PLB, then
            // writes an item's equipment-table byte through a pointer
            // whose bank byte is $7E) silently wrote to the wrong bank
            // instead, with the real target address never touched -
            // found via the LttP "lamp appears then never enters
            // inventory" investigation.
            _instructions[0x87] = new Instruction { Name = "STA", AddrMode = AddrDirectIndirectLong, Operate = OpSTA, Cycles = 6 };
            // --- Added: read-modify-write memory family + close relatives ---
            // 0xE6 (INC dp) is what the ROM halted on; INC/DEC on memory in all four
            // common addressing modes come as a set, and DEC A / ROR A / BIT round out
            // the same tier of ubiquitous instructions.
            _instructions[0xE6] = new Instruction { Name = "INC", AddrMode = AddrDirectPage, Operate = OpINCMem, Cycles = 5 };
            _instructions[0xEE] = new Instruction { Name = "INC", AddrMode = AddrAbsolute, Operate = OpINCMem, Cycles = 6 };
            _instructions[0xF6] = new Instruction { Name = "INC", AddrMode = AddrDirectPageX, Operate = OpINCMem, Cycles = 6 };
            _instructions[0xFE] = new Instruction { Name = "INC", AddrMode = AddrAbsoluteX, Operate = OpINCMem, Cycles = 7 };
            _instructions[0xC6] = new Instruction { Name = "DEC", AddrMode = AddrDirectPage, Operate = OpDECMem, Cycles = 5 };
            _instructions[0xCE] = new Instruction { Name = "DEC", AddrMode = AddrAbsolute, Operate = OpDECMem, Cycles = 6 };
            _instructions[0xD6] = new Instruction { Name = "DEC", AddrMode = AddrDirectPageX, Operate = OpDECMem, Cycles = 6 };
            _instructions[0xDE] = new Instruction { Name = "DEC", AddrMode = AddrAbsoluteX, Operate = OpDECMem, Cycles = 7 };
            _instructions[0x3A] = new Instruction { Name = "DEC A", AddrMode = AddrImplied, Operate = OpDECA, Cycles = 2 };
            _instructions[0x6A] = new Instruction { Name = "ROR A", AddrMode = AddrImplied, Operate = OpRORA, Cycles = 2 };
            _instructions[0x24] = new Instruction { Name = "BIT", AddrMode = AddrDirectPage, Operate = OpBIT, Cycles = 3 };
            _instructions[0x2C] = new Instruction { Name = "BIT", AddrMode = AddrAbsolute, Operate = OpBIT, Cycles = 4 };
            _instructions[0x34] = new Instruction { Name = "BIT", AddrMode = AddrDirectPageX, Operate = OpBIT, Cycles = 4 };
            _instructions[0x89] = new Instruction { Name = "BIT", AddrMode = AddrImmediateM, Operate = OpBITImm, Cycles = 2 };
            // --- Added: STX/STY store family, dp-indexed loads/stores, register transfers ---
            // 0x84 (STY dp) is what the ROM halted on; STX/STY were entirely missing.
            _instructions[0x84] = new Instruction { Name = "STY", AddrMode = AddrDirectPage, Operate = OpSTY, Cycles = 3 };
            _instructions[0x8C] = new Instruction { Name = "STY", AddrMode = AddrAbsolute, Operate = OpSTY, Cycles = 4 };
            _instructions[0x94] = new Instruction { Name = "STY", AddrMode = AddrDirectPageX, Operate = OpSTY, Cycles = 4 };
            _instructions[0x86] = new Instruction { Name = "STX", AddrMode = AddrDirectPage, Operate = OpSTX, Cycles = 3 };
            _instructions[0x8E] = new Instruction { Name = "STX", AddrMode = AddrAbsolute, Operate = OpSTX, Cycles = 4 };
            _instructions[0x96] = new Instruction { Name = "STX", AddrMode = AddrDirectPageY, Operate = OpSTX, Cycles = 4 };
            _instructions[0x95] = new Instruction { Name = "STA", AddrMode = AddrDirectPageX, Operate = OpSTA, Cycles = 4 };
            _instructions[0xB5] = new Instruction { Name = "LDA", AddrMode = AddrDirectPageX, Operate = OpLDA, Cycles = 4 };
            _instructions[0xB4] = new Instruction { Name = "LDY", AddrMode = AddrDirectPageX, Operate = OpLDY, Cycles = 4 };
            _instructions[0xB6] = new Instruction { Name = "LDX", AddrMode = AddrDirectPageY, Operate = OpLDX, Cycles = 4 };
            _instructions[0xA4] = new Instruction { Name = "LDY", AddrMode = AddrDirectPage, Operate = OpLDY, Cycles = 3 };
            _instructions[0xA6] = new Instruction { Name = "LDX", AddrMode = AddrDirectPage, Operate = OpLDX, Cycles = 3 };
            _instructions[0xAC] = new Instruction { Name = "LDY", AddrMode = AddrAbsolute, Operate = OpLDY, Cycles = 4 };
            _instructions[0xAE] = new Instruction { Name = "LDX", AddrMode = AddrAbsolute, Operate = OpLDX, Cycles = 4 };
            _instructions[0x8A] = new Instruction { Name = "TXA", AddrMode = AddrImplied, Operate = OpTXA, Cycles = 2 };
            _instructions[0x9B] = new Instruction { Name = "TXY", AddrMode = AddrImplied, Operate = OpTXY, Cycles = 2 };
            _instructions[0xBB] = new Instruction { Name = "TYX", AddrMode = AddrImplied, Operate = OpTYX, Cycles = 2 };
            _instructions[0x7B] = new Instruction { Name = "TDC", AddrMode = AddrImplied, Operate = OpTDC, Cycles = 2 };
            _instructions[0x3B] = new Instruction { Name = "TSC", AddrMode = AddrImplied, Operate = OpTSC, Cycles = 2 };
            _instructions[0x9A] = new Instruction { Name = "TXS", AddrMode = AddrImplied, Operate = OpTXS, Cycles = 2 };
            _instructions[0xBA] = new Instruction { Name = "TSX", AddrMode = AddrImplied, Operate = OpTSX, Cycles = 2 };
            // --- Push Effective Address Family ---
            _instructions[0xF4] = new Instruction { Name = "PEA", AddrMode = AddrImplied, Operate = OpPEA, Cycles = 5 };
            _instructions[0xD4] = new Instruction { Name = "PEI", AddrMode = AddrImplied, Operate = OpPEI, Cycles = 6 };
            _instructions[0x62] = new Instruction { Name = "PER", AddrMode = AddrImplied, Operate = OpPER, Cycles = 6 };
            // --- TRB / TSB Memory Family ---
            _instructions[0x14] = new Instruction { Name = "TRB", AddrMode = AddrDirectPage, Operate = OpTRB, Cycles = 5 };
            _instructions[0x1C] = new Instruction { Name = "TRB", AddrMode = AddrAbsolute, Operate = OpTRB, Cycles = 6 };
            _instructions[0x04] = new Instruction { Name = "TSB", AddrMode = AddrDirectPage, Operate = OpTSB, Cycles = 5 };
            _instructions[0x0C] = new Instruction { Name = "TSB", AddrMode = AddrAbsolute, Operate = OpTSB, Cycles = 6 };
            // --- Added: indirect jump family ---
            // 0xDC (JML [abs]) is what the ROM halted on - jump-table dispatch, which
            // SMW's game-mode engine uses heavily. Its siblings come as a set.
            _instructions[0xDC] = new Instruction { Name = "JML", AddrMode = AddrAbsoluteIndirectLong, Operate = OpJML, Cycles = 6 };
            _instructions[0x6C] = new Instruction { Name = "JMP", AddrMode = AddrAbsoluteIndirect, Operate = OpJMP, Cycles = 5 };
            _instructions[0x7C] = new Instruction { Name = "JMP", AddrMode = AddrAbsoluteIndexedIndirect, Operate = OpJMP, Cycles = 6 };
            _instructions[0xFC] = new Instruction { Name = "JSR", AddrMode = AddrAbsoluteIndexedIndirect, Operate = OpJSR, Cycles = 8 };
            // --- Absolute Long, X Family ---
            _instructions[0xDF] = new Instruction { Name = "CMP", AddrMode = AddrAbsoluteLongX, Operate = OpCMP, Cycles = 5 };
            _instructions[0x1F] = new Instruction { Name = "ORA", AddrMode = AddrAbsoluteLongX, Operate = OpORA, Cycles = 5 };
            _instructions[0x3F] = new Instruction { Name = "AND", AddrMode = AddrAbsoluteLongX, Operate = OpAND, Cycles = 5 };
            _instructions[0x5F] = new Instruction { Name = "EOR", AddrMode = AddrAbsoluteLongX, Operate = OpEOR, Cycles = 5 };
            _instructions[0x7F] = new Instruction { Name = "ADC", AddrMode = AddrAbsoluteLongX, Operate = OpADC, Cycles = 5 };
            _instructions[0xFF] = new Instruction { Name = "SBC", AddrMode = AddrAbsoluteLongX, Operate = OpSBC, Cycles = 5 };
            // --- Direct Indirect Indexed Y Family ( (dp),Y ) ---
            _instructions[0xB1] = new Instruction { Name = "LDA", AddrMode = AddrDirectIndirectY, Operate = OpLDA, Cycles = 5 };
            _instructions[0x91] = new Instruction { Name = "STA", AddrMode = AddrDirectIndirectY, Operate = OpSTA, Cycles = 6 };
            _instructions[0xD1] = new Instruction { Name = "CMP", AddrMode = AddrDirectIndirectY, Operate = OpCMP, Cycles = 5 };
            _instructions[0x71] = new Instruction { Name = "ADC", AddrMode = AddrDirectIndirectY, Operate = OpADC, Cycles = 5 };
            _instructions[0xF1] = new Instruction { Name = "SBC", AddrMode = AddrDirectIndirectY, Operate = OpSBC, Cycles = 5 };
            _instructions[0x31] = new Instruction { Name = "AND", AddrMode = AddrDirectIndirectY, Operate = OpAND, Cycles = 5 };
            _instructions[0x11] = new Instruction { Name = "ORA", AddrMode = AddrDirectIndirectY, Operate = OpORA, Cycles = 5 };
            _instructions[0x51] = new Instruction { Name = "EOR", AddrMode = AddrDirectIndirectY, Operate = OpEOR, Cycles = 5 };
            // --- Direct Page Indexed Indirect X Family ( (dp,X) ) ---
            _instructions[0xA1] = new Instruction { Name = "LDA", AddrMode = AddrDirectIndirectX, Operate = OpLDA, Cycles = 6 };
            _instructions[0x81] = new Instruction { Name = "STA", AddrMode = AddrDirectIndirectX, Operate = OpSTA, Cycles = 6 };
            _instructions[0xC1] = new Instruction { Name = "CMP", AddrMode = AddrDirectIndirectX, Operate = OpCMP, Cycles = 6 };
            _instructions[0x61] = new Instruction { Name = "ADC", AddrMode = AddrDirectIndirectX, Operate = OpADC, Cycles = 6 };
            _instructions[0xE1] = new Instruction { Name = "SBC", AddrMode = AddrDirectIndirectX, Operate = OpSBC, Cycles = 6 };
            _instructions[0x21] = new Instruction { Name = "AND", AddrMode = AddrDirectIndirectX, Operate = OpAND, Cycles = 6 };
            _instructions[0x01] = new Instruction { Name = "ORA", AddrMode = AddrDirectIndirectX, Operate = OpORA, Cycles = 6 };
            _instructions[0x41] = new Instruction { Name = "EOR", AddrMode = AddrDirectIndirectX, Operate = OpEOR, Cycles = 6 };
            // --- Block Move Family ---
            _instructions[0x54] = new Instruction { Name = "MVN", AddrMode = AddrBlockMove, Operate = OpMVN, Cycles = 7 };
            _instructions[0x44] = new Instruction { Name = "MVP", AddrMode = AddrBlockMove, Operate = OpMVP, Cycles = 7 };
            // --- Stack Relative (sr,S) ALU Family ---
            _instructions[0x03] = new Instruction { Name = "ORA", AddrMode = AddrStackRelative, Operate = OpORA, Cycles = 4 };
            _instructions[0x23] = new Instruction { Name = "AND", AddrMode = AddrStackRelative, Operate = OpAND, Cycles = 4 };
            _instructions[0x43] = new Instruction { Name = "EOR", AddrMode = AddrStackRelative, Operate = OpEOR, Cycles = 4 };
            _instructions[0x63] = new Instruction { Name = "ADC", AddrMode = AddrStackRelative, Operate = OpADC, Cycles = 4 };
            _instructions[0x83] = new Instruction { Name = "STA", AddrMode = AddrStackRelative, Operate = OpSTA, Cycles = 4 };
            _instructions[0xA3] = new Instruction { Name = "LDA", AddrMode = AddrStackRelative, Operate = OpLDA, Cycles = 4 };
            _instructions[0xC3] = new Instruction { Name = "CMP", AddrMode = AddrStackRelative, Operate = OpCMP, Cycles = 4 };
            _instructions[0xE3] = new Instruction { Name = "SBC", AddrMode = AddrStackRelative, Operate = OpSBC, Cycles = 4 };
            // --- ASL Memory Family ---
            _instructions[0x06] = new Instruction { Name = "ASL", AddrMode = AddrDirectPage, Operate = OpASL, Cycles = 5 };
            _instructions[0x0E] = new Instruction { Name = "ASL", AddrMode = AddrAbsolute, Operate = OpASL, Cycles = 6 };
            _instructions[0x16] = new Instruction { Name = "ASL", AddrMode = AddrDirectPageX, Operate = OpASL, Cycles = 6 };
            _instructions[0x1E] = new Instruction { Name = "ASL", AddrMode = AddrAbsoluteX, Operate = OpASL, Cycles = 7 };
            // --- SBC Memory Family ---
            _instructions[0xE5] = new Instruction { Name = "SBC", AddrMode = AddrDirectPage, Operate = OpSBC, Cycles = 3 };
            _instructions[0xF5] = new Instruction { Name = "SBC", AddrMode = AddrDirectPageX, Operate = OpSBC, Cycles = 4 };
            _instructions[0xED] = new Instruction { Name = "SBC", AddrMode = AddrAbsolute, Operate = OpSBC, Cycles = 4 };
            // --- EOR Memory Family ---
            _instructions[0x45] = new Instruction { Name = "EOR", AddrMode = AddrDirectPage, Operate = OpEOR, Cycles = 3 };
            _instructions[0x55] = new Instruction { Name = "EOR", AddrMode = AddrDirectPageX, Operate = OpEOR, Cycles = 4 };
            // --- ROR Memory Family ---
            _instructions[0x66] = new Instruction { Name = "ROR", AddrMode = AddrDirectPage, Operate = OpRORMem, Cycles = 5 };
            _instructions[0x6E] = new Instruction { Name = "ROR", AddrMode = AddrAbsolute, Operate = OpRORMem, Cycles = 6 };
            _instructions[0x76] = new Instruction { Name = "ROR", AddrMode = AddrDirectPageX, Operate = OpRORMem, Cycles = 6 };
            _instructions[0x7E] = new Instruction { Name = "ROR", AddrMode = AddrAbsoluteX, Operate = OpRORMem, Cycles = 7 };
            // --- Direct Indirect Family ( (dp) ) ---
            _instructions[0x12] = new Instruction { Name = "ORA", AddrMode = AddrDirectIndirect, Operate = OpORA, Cycles = 5 };
            _instructions[0x32] = new Instruction { Name = "AND", AddrMode = AddrDirectIndirect, Operate = OpAND, Cycles = 5 };
            _instructions[0x52] = new Instruction { Name = "EOR", AddrMode = AddrDirectIndirect, Operate = OpEOR, Cycles = 5 };
            _instructions[0x72] = new Instruction { Name = "ADC", AddrMode = AddrDirectIndirect, Operate = OpADC, Cycles = 5 };
            _instructions[0x92] = new Instruction { Name = "STA", AddrMode = AddrDirectIndirect, Operate = OpSTA, Cycles = 5 };
            _instructions[0xD2] = new Instruction { Name = "CMP", AddrMode = AddrDirectIndirect, Operate = OpCMP, Cycles = 5 };
            _instructions[0xF2] = new Instruction { Name = "SBC", AddrMode = AddrDirectIndirect, Operate = OpSBC, Cycles = 5 };

            // --- Stack Relative Indirect Indexed,Y ( (sr,S),Y ) Family ---
            // Verified against the documented 65816 opcode matrix (oxyron.de,
            // cross-checked against softpixel's independent table since
            // oxyron's own page lists EOR's isy cycle count as 6 where every
            // other opcode in this family - and softpixel's table - list 7;
            // treating that as a typo in that one source rather than a real
            // asymmetry, since nothing about EOR's addressing differs from
            // the others here).
            _instructions[0x13] = new Instruction { Name = "ORA", AddrMode = AddrStackRelativeIndirectY, Operate = OpORA, Cycles = 7 };
            _instructions[0x33] = new Instruction { Name = "AND", AddrMode = AddrStackRelativeIndirectY, Operate = OpAND, Cycles = 7 };
            _instructions[0x53] = new Instruction { Name = "EOR", AddrMode = AddrStackRelativeIndirectY, Operate = OpEOR, Cycles = 7 };
            _instructions[0x73] = new Instruction { Name = "ADC", AddrMode = AddrStackRelativeIndirectY, Operate = OpADC, Cycles = 7 };
            _instructions[0x93] = new Instruction { Name = "STA", AddrMode = AddrStackRelativeIndirectY, Operate = OpSTA, Cycles = 7 };
            _instructions[0xB3] = new Instruction { Name = "LDA", AddrMode = AddrStackRelativeIndirectY, Operate = OpLDA, Cycles = 7 };
            _instructions[0xD3] = new Instruction { Name = "CMP", AddrMode = AddrStackRelativeIndirectY, Operate = OpCMP, Cycles = 7 };
            _instructions[0xF3] = new Instruction { Name = "SBC", AddrMode = AddrStackRelativeIndirectY, Operate = OpSBC, Cycles = 7 };

            // --- Direct Page Indirect Long ( [dp] ) Family - remaining members ---
            // (0x07 ORA [dp] was already implemented; these five complete the family.)
            _instructions[0x27] = new Instruction { Name = "AND", AddrMode = AddrDirectIndirectLong, Operate = OpAND, Cycles = 6 };
            _instructions[0x47] = new Instruction { Name = "EOR", AddrMode = AddrDirectIndirectLong, Operate = OpEOR, Cycles = 6 };
            _instructions[0x67] = new Instruction { Name = "ADC", AddrMode = AddrDirectIndirectLong, Operate = OpADC, Cycles = 6 };
            _instructions[0xC7] = new Instruction { Name = "CMP", AddrMode = AddrDirectIndirectLong, Operate = OpCMP, Cycles = 6 };
            _instructions[0xE7] = new Instruction { Name = "SBC", AddrMode = AddrDirectIndirectLong, Operate = OpSBC, Cycles = 6 };

            // --- Remaining misc opcodes ---
            _instructions[0x3C] = new Instruction { Name = "BIT", AddrMode = AddrAbsoluteX, Operate = OpBIT, Cycles = 4 };
            _instructions[0xB8] = new Instruction { Name = "CLV", AddrMode = AddrImplied, Operate = OpCLV, Cycles = 2 };
            _instructions[0xD8] = new Instruction { Name = "CLD", AddrMode = AddrImplied, Operate = OpCLD, Cycles = 2 };
            _instructions[0xF8] = new Instruction { Name = "SED", AddrMode = AddrImplied, Operate = OpSED, Cycles = 2 };
            _instructions[0xCB] = new Instruction { Name = "WAI", AddrMode = AddrImplied, Operate = OpWAI, Cycles = 3 };
            _instructions[0xDB] = new Instruction { Name = "STP", AddrMode = AddrImplied, Operate = OpSTP, Cycles = 3 };
            // WDM: officially reserved for future expansion. Every real 65816
            // treats it as a 2-byte NOP (fetch the opcode, fetch and discard
            // one operand byte, do nothing) - that's what every documented
            // source agrees on, and it's what SNES games rely on if they hit
            // it at all (they don't intentionally use it, but some
            // copy-protection/anti-emulation checks have historically probed
            // for correct WDM handling).
            _instructions[0x42] = new Instruction { Name = "WDM", AddrMode = AddrImmediate8, Operate = OpNOP, Cycles = 2 };
        }
    }
}
