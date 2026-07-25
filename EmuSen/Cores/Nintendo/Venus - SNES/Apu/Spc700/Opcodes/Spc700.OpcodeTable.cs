using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    public partial class Spc700
    {
        // --- Opcode Setup ---

        private void BuildOpcodeTable()
        {
            _instructions = new SpcInstruction[256];

            for (int i = 0; i < 256; i++)
            {
                _instructions[i] = new SpcInstruction 
                { 
                    Name = "NOP/UNK", 
                    AddrMode = AddrImplied, 
                    Operate = OpUnknown, 
                    Cycles = 2 
                };
            }

            // --- IPL ROM Boot & RAM Clear ---
            _instructions[0xCD] = new SpcInstruction { Name = "MOV X, #imm", AddrMode = AddrImmediate, Operate = OpMOV_X_Imm, Cycles = 2 };
            _instructions[0xBD] = new SpcInstruction { Name = "MOV SP, X", AddrMode = AddrImplied, Operate = OpMOV_SP_X, Cycles = 2 };
            _instructions[0xE8] = new SpcInstruction { Name = "MOV A, #imm", AddrMode = AddrImmediate, Operate = OpMOV_A_Imm, Cycles = 2 };
            _instructions[0xC6] = new SpcInstruction { Name = "MOV (X), A", AddrMode = AddrIndirectX, Operate = OpMOV_IndX_A, Cycles = 4 };
            _instructions[0x1D] = new SpcInstruction { Name = "DEC X", AddrMode = AddrImplied, Operate = OpDEC_X, Cycles = 2 };
            _instructions[0x8F] = new SpcInstruction { Name = "MOV dp, #imm", AddrMode = AddrImmediateToDirectPage, Operate = OpMOV_dp_imm, Cycles = 5 };
            _instructions[0x78] = new SpcInstruction { Name = "CMP dp, #imm", AddrMode = AddrImmediateToDirectPage, Operate = OpCMP_dp_imm, Cycles = 5 };
            _instructions[0x5A] = new SpcInstruction { Name = "CMPW YA, dp", AddrMode = AddrDirectPage, Operate = OpCMPW_YA_dp, Cycles = 4 };
            
            // --- Core Branches ---
            _instructions[0x2F] = new SpcInstruction { Name = "BRA rel", AddrMode = AddrRelative, Operate = OpBRA, Cycles = 4 };
            _instructions[0xF0] = new SpcInstruction { Name = "BEQ rel", AddrMode = AddrRelative, Operate = OpBEQ, Cycles = 2 };
            _instructions[0x10] = new SpcInstruction { Name = "BPL rel", AddrMode = AddrRelative, Operate = OpBPL, Cycles = 2 };
            _instructions[0x30] = new SpcInstruction { Name = "BMI rel", AddrMode = AddrRelative, Operate = OpBMI, Cycles = 2 };
            _instructions[0x3F] = new SpcInstruction { Name = "CALL abs", AddrMode = AddrAbsolute, Operate = OpCALL, Cycles = 8 };
            _instructions[0x6F] = new SpcInstruction { Name = "RET", AddrMode = AddrImplied, Operate = OpRET, Cycles = 5 };
            _instructions[0x90] = new SpcInstruction { Name = "BCC rel", AddrMode = AddrRelative, Operate = OpBCC, Cycles = 2 };
            _instructions[0xDB] = new SpcInstruction { Name = "MOV dp+X, Y", AddrMode = AddrDirectPageX, Operate = OpMOV_dp_Y, Cycles = 5 };
            _instructions[0xDE] = new SpcInstruction { Name = "CBNE dp+X, rel", AddrMode = AddrDirectPageX, Operate = OpCBNE_dp_rel, Cycles = 6 };
            _instructions[0x2E] = new SpcInstruction { Name = "CBNE dp, rel", AddrMode = AddrDirectPage, Operate = OpCBNE_dp_rel, Cycles = 5 };
            _instructions[0x08] = new SpcInstruction { Name = "OR A, #imm", AddrMode = AddrImmediate, Operate = OpOR_A_dp, Cycles = 2 };
            _instructions[0xB0] = new SpcInstruction { Name = "BCS rel", AddrMode = AddrRelative, Operate = OpBCS, Cycles = 2 };
            
            // --- Port Polling & Memory Mapping ---
            _instructions[0xBA] = new SpcInstruction { Name = "MOVW YA, dp", AddrMode = AddrDirectPage, Operate = OpMOVW_YA_dp, Cycles = 5 };
            _instructions[0xDA] = new SpcInstruction { Name = "MOVW dp, YA", AddrMode = AddrDirectPage, Operate = OpMOVW_dp_YA, Cycles = 5 };
            _instructions[0xC4] = new SpcInstruction { Name = "MOV dp, A", AddrMode = AddrDirectPage, Operate = OpMOV_dp_A, Cycles = 4 };
            _instructions[0xDD] = new SpcInstruction { Name = "MOV A, Y", AddrMode = AddrImplied, Operate = OpMOV_A_Y, Cycles = 2 };
            _instructions[0x5D] = new SpcInstruction { Name = "MOV X, A", AddrMode = AddrImplied, Operate = OpMOV_X_A, Cycles = 2 };
            _instructions[0xEB] = new SpcInstruction { Name = "MOV Y, dp", AddrMode = AddrDirectPage, Operate = OpMOV_Y_mem, Cycles = 3 };
            _instructions[0xFB] = new SpcInstruction { Name = "MOV Y, dp+X", AddrMode = AddrDirectPageX, Operate = OpMOV_Y_mem, Cycles = 4 };
            _instructions[0x7E] = new SpcInstruction { Name = "CMP Y, dp", AddrMode = AddrDirectPage, Operate = OpCMP_Y_dp, Cycles = 4 };
            _instructions[0xE4] = new SpcInstruction { Name = "MOV A, dp", AddrMode = AddrDirectPage, Operate = OpMOV_A_dp, Cycles = 3 };
            _instructions[0xCB] = new SpcInstruction { Name = "MOV dp, Y", AddrMode = AddrDirectPage, Operate = OpMOV_dp_Y, Cycles = 4 };
            _instructions[0xC5] = new SpcInstruction { Name = "MOV abs, A", AddrMode = AddrAbsolute, Operate = OpMOV_abs_A, Cycles = 5 };
            _instructions[0xCC] = new SpcInstruction { Name = "MOV abs, Y", AddrMode = AddrAbsolute, Operate = OpMOV_abs_Y, Cycles = 5 };
            _instructions[0xEC] = new SpcInstruction { Name = "MOV Y, abs", AddrMode = AddrAbsolute, Operate = OpMOV_Y_abs, Cycles = 5 };
            _instructions[0xE5] = new SpcInstruction { Name = "MOV A, abs", AddrMode = AddrAbsolute, Operate = OpMOV_A_abs, Cycles = 5 };
            _instructions[0xAF] = new SpcInstruction { Name = "MOV (X)+, A", AddrMode = AddrIndirectX, Operate = OpMOV_IndXInc_A, Cycles = 4 };
            _instructions[0xBC] = new SpcInstruction { Name = "INC A", AddrMode = AddrImplied, Operate = OpINC_A, Cycles = 2 };
            _instructions[0xD5] = new SpcInstruction { Name = "MOV abs+X, A", AddrMode = AddrAbsoluteIndexedX, Operate = OpMOV_absX_A, Cycles = 6 };
            _instructions[0xF5] = new SpcInstruction { Name = "MOV A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpMOV_A_absX, Cycles = 5 };
            _instructions[0xFD] = new SpcInstruction { Name = "MOV Y, A", AddrMode = AddrImplied, Operate = OpMOV_Y_A, Cycles = 2 };
            _instructions[0xCF] = new SpcInstruction { Name = "MUL YA", AddrMode = AddrImplied, Operate = OpMUL_YA, Cycles = 9 };
            _instructions[0x84] = new SpcInstruction { Name = "ADC A, dp", AddrMode = AddrDirectPage, Operate = OpADC_A, Cycles = 3 };
            _instructions[0x85] = new SpcInstruction { Name = "ADC A, abs", AddrMode = AddrAbsolute, Operate = OpADC_A, Cycles = 4 };
            _instructions[0x86] = new SpcInstruction { Name = "ADC A, (X)", AddrMode = AddrIndirectX, Operate = OpADC_A, Cycles = 3 };
            _instructions[0x87] = new SpcInstruction { Name = "ADC A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpADC_A, Cycles = 6 };
            _instructions[0x94] = new SpcInstruction { Name = "ADC A, dp+X", AddrMode = AddrDirectPageX, Operate = OpADC_A, Cycles = 4 };
            _instructions[0x95] = new SpcInstruction { Name = "ADC A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpADC_A, Cycles = 5 };
            _instructions[0x96] = new SpcInstruction { Name = "ADC A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpADC_A, Cycles = 5 };
            _instructions[0x97] = new SpcInstruction { Name = "ADC A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpADC_A, Cycles = 6 };
            _instructions[0x98] = new SpcInstruction { Name = "ADC dp, #imm", AddrMode = AddrImmediate, Operate = OpADC_dp_imm, Cycles = 5 };
            _instructions[0x7D] = new SpcInstruction { Name = "MOV A, X", AddrMode = AddrImplied, Operate = OpMOV_A_X, Cycles = 2 };
            _instructions[0xF4] = new SpcInstruction { Name = "MOV A, dp+X", AddrMode = AddrDirectPageX, Operate = OpMOV_A_dp, Cycles = 4 };
            _instructions[0x75] = new SpcInstruction { Name = "CMP A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpCMP_A, Cycles = 5 };
            _instructions[0x8D] = new SpcInstruction { Name = "MOV Y, #imm", AddrMode = AddrImmediate, Operate = OpMOV_Y_Imm, Cycles = 2 };
            _instructions[0x68] = new SpcInstruction { Name = "CMP A, #imm", AddrMode = AddrImmediate, Operate = OpCMP_A, Cycles = 2 };
            _instructions[0x1C] = new SpcInstruction { Name = "ASL A", AddrMode = AddrImplied, Operate = OpASL_A, Cycles = 2 };
            _instructions[0x0B] = new SpcInstruction { Name = "ASL dp", AddrMode = AddrDirectPage, Operate = OpASL_dp, Cycles = 4 };
            _instructions[0x0C] = new SpcInstruction { Name = "ASL abs", AddrMode = AddrAbsolute, Operate = OpASL_dp, Cycles = 5 };
            _instructions[0x6B] = new SpcInstruction { Name = "ROR dp", AddrMode = AddrDirectPage, Operate = OpROR_dp, Cycles = 4 };
            _instructions[0x38] = new SpcInstruction { Name = "AND dp, #imm", AddrMode = AddrImmediate, Operate = OpAND_dp_imm, Cycles = 5 };
            _instructions[0x58] = new SpcInstruction { Name = "EOR dp, #imm", AddrMode = AddrImmediate, Operate = OpEOR_dp_imm, Cycles = 5 };
            _instructions[0xB8] = new SpcInstruction { Name = "SBC dp, #imm", AddrMode = AddrImmediate, Operate = OpSBC_dp_imm, Cycles = 5 };
            _instructions[0x6C] = new SpcInstruction { Name = "ROR abs", AddrMode = AddrAbsolute, Operate = OpROR_dp, Cycles = 5 };
            _instructions[0x7C] = new SpcInstruction { Name = "ROR A", AddrMode = AddrImplied, Operate = OpROR_A, Cycles = 2 };
            _instructions[0x05] = new SpcInstruction { Name = "OR A, abs", AddrMode = AddrAbsolute, Operate = OpOR_A_abs, Cycles = 4 };
            _instructions[0x18] = new SpcInstruction { Name = "OR dp, #imm", AddrMode = AddrImmediate, Operate = OpOR_dp_imm, Cycles = 5 };
            _instructions[0x09] = new SpcInstruction { Name = "OR dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpOR_dp_dp, Cycles = 6 };
            _instructions[0x28] = new SpcInstruction { Name = "AND A, #imm", AddrMode = AddrImmediate, Operate = OpAND_A, Cycles = 2 };
            _instructions[0x48] = new SpcInstruction { Name = "EOR A, #imm", AddrMode = AddrImmediate, Operate = OpEOR_A, Cycles = 2 };
            _instructions[0xD4] = new SpcInstruction { Name = "MOV dp+X, A", AddrMode = AddrDirectPageX, Operate = OpMOV_dp_A, Cycles = 5 };
            _instructions[0xA8] = new SpcInstruction { Name = "SBC A, #imm", AddrMode = AddrImmediate, Operate = OpSBC_A, Cycles = 2 };
            _instructions[0xA4] = new SpcInstruction { Name = "SBC A, dp", AddrMode = AddrDirectPage, Operate = OpSBC_A, Cycles = 3 };
            _instructions[0xB4] = new SpcInstruction { Name = "SBC A, dp+X", AddrMode = AddrDirectPageX, Operate = OpSBC_A, Cycles = 4 };
            _instructions[0xA5] = new SpcInstruction { Name = "SBC A, abs", AddrMode = AddrAbsolute, Operate = OpSBC_A, Cycles = 4 };
            _instructions[0xB5] = new SpcInstruction { Name = "SBC A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpSBC_A, Cycles = 5 };
            _instructions[0xB6] = new SpcInstruction { Name = "SBC A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpSBC_A, Cycles = 5 };
            _instructions[0xA6] = new SpcInstruction { Name = "SBC A, (X)", AddrMode = AddrIndirectX, Operate = OpSBC_A, Cycles = 3 };
            _instructions[0xA7] = new SpcInstruction { Name = "SBC A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpSBC_A, Cycles = 6 };
            _instructions[0xB7] = new SpcInstruction { Name = "SBC A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpSBC_A, Cycles = 6 };
            _instructions[0x7A] = new SpcInstruction { Name = "ADDW YA, dp", AddrMode = AddrDirectPage, Operate = OpADDW_YA_dp, Cycles = 5 };
            _instructions[0x9E] = new SpcInstruction { Name = "DIV YA, X", AddrMode = AddrImplied, Operate = OpDIV_YA_X, Cycles = 12 };
            _instructions[0x4B] = new SpcInstruction { Name = "LSR dp", AddrMode = AddrDirectPage, Operate = OpLSR_dp, Cycles = 4 };
            _instructions[0x0D] = new SpcInstruction { Name = "PUSH PSW", AddrMode = AddrImplied, Operate = OpPUSH_PSW, Cycles = 4 };
            _instructions[0x8E] = new SpcInstruction { Name = "POP PSW", AddrMode = AddrImplied, Operate = OpPOP_PSW, Cycles = 4 };
            _instructions[0x7F] = new SpcInstruction { Name = "RETI", AddrMode = AddrImplied, Operate = OpRETI, Cycles = 6 };
            _instructions[0x9F] = new SpcInstruction { Name = "XCN A", AddrMode = AddrImplied, Operate = OpXCN_A, Cycles = 5 };
            _instructions[0x4F] = new SpcInstruction { Name = "PCALL upage", AddrMode = AddrImmediate, Operate = OpPCALL, Cycles = 6 };
            _instructions[0x9A] = new SpcInstruction { Name = "SUBW YA, dp", AddrMode = AddrDirectPage, Operate = OpSUBW_YA_dp, Cycles = 5 };
            _instructions[0x5C] = new SpcInstruction { Name = "LSR A", AddrMode = AddrImplied, Operate = OpLSR_A, Cycles = 2 };
            _instructions[0x24] = new SpcInstruction { Name = "AND A, dp", AddrMode = AddrDirectPage, Operate = OpAND_A, Cycles = 3 };
            _instructions[0x34] = new SpcInstruction { Name = "AND A, dp+X", AddrMode = AddrDirectPageX, Operate = OpAND_A, Cycles = 4 };
            _instructions[0x25] = new SpcInstruction { Name = "AND A, abs", AddrMode = AddrAbsolute, Operate = OpAND_A, Cycles = 4 };
            _instructions[0x35] = new SpcInstruction { Name = "AND A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpAND_A, Cycles = 5 };
            _instructions[0x36] = new SpcInstruction { Name = "AND A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpAND_A, Cycles = 5 };
            _instructions[0x27] = new SpcInstruction { Name = "AND A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpAND_A, Cycles = 6 };
            _instructions[0x37] = new SpcInstruction { Name = "AND A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpAND_A, Cycles = 6 };
            _instructions[0x0E] = new SpcInstruction { Name = "TSET1 abs", AddrMode = AddrAbsolute, Operate = OpTSET1_abs, Cycles = 6 };
            _instructions[0x4E] = new SpcInstruction { Name = "TCLR1 abs", AddrMode = AddrAbsolute, Operate = OpTCLR1_abs, Cycles = 6 };
            
            // Decrement Memory Family
            _instructions[0x8B] = new SpcInstruction { Name = "DEC dp", AddrMode = AddrDirectPage, Operate = OpDEC_mem, Cycles = 4 };
            _instructions[0x8C] = new SpcInstruction { Name = "DEC abs", AddrMode = AddrAbsolute, Operate = OpDEC_mem, Cycles = 5 };
            _instructions[0x9B] = new SpcInstruction { Name = "DEC dp+X", AddrMode = AddrDirectPageX, Operate = OpDEC_mem, Cycles = 5 };
            
            // Decrement Register Family
            _instructions[0x9C] = new SpcInstruction { Name = "DEC A", AddrMode = AddrImplied, Operate = OpDEC_A, Cycles = 2 };
            _instructions[0xDC] = new SpcInstruction { Name = "DEC Y", AddrMode = AddrImplied, Operate = OpDEC_Y, Cycles = 2 };
            

            // --- Payload Storage ---
            _instructions[0xD7] = new SpcInstruction { Name = "MOV [dp]+Y, A", AddrMode = AddrDirectIndirectIndexedY, Operate = OpMOV_IndY_A, Cycles = 7 };
            _instructions[0xAB] = new SpcInstruction { Name = "INC dp", AddrMode = AddrDirectPage, Operate = OpINC_dp, Cycles = 4 };
            // --- Increment Memory Family ---
            _instructions[0xBB] = new SpcInstruction { Name = "INC dp+X", AddrMode = AddrDirectPageX, Operate = OpINC_dp, Cycles = 5 };
            _instructions[0xAC] = new SpcInstruction { Name = "INC abs", AddrMode = AddrAbsolute, Operate = OpINC_dp, Cycles = 5 };
            _instructions[0x3A] = new SpcInstruction { Name = "INCW dp", AddrMode = AddrDirectPage, Operate = OpINCW_dp, Cycles = 6 };
            _instructions[0x1A] = new SpcInstruction { Name = "DECW dp", AddrMode = AddrDirectPage, Operate = OpDECW_dp, Cycles = 6 };

            // --- Iteration & Stack Control ---
            _instructions[0xFC] = new SpcInstruction { Name = "INC Y", AddrMode = AddrImplied, Operate = OpINC_Y, Cycles = 2 };
            _instructions[0x3D] = new SpcInstruction { Name = "INC X", AddrMode = AddrImplied, Operate = OpINC_X, Cycles = 2 };
            _instructions[0x2D] = new SpcInstruction { Name = "PUSH A", AddrMode = AddrImplied, Operate = OpPUSH_A, Cycles = 4 };
            _instructions[0xAE] = new SpcInstruction { Name = "POP A", AddrMode = AddrImplied, Operate = OpPOP_A, Cycles = 4 };
            _instructions[0xCE] = new SpcInstruction { Name = "POP X", AddrMode = AddrImplied, Operate = OpPOP_X, Cycles = 4 };
            _instructions[0xC8] = new SpcInstruction { Name = "CMP X, #imm", AddrMode = AddrImmediate, Operate = OpCMP_X_Imm, Cycles = 2 };
            _instructions[0x6D] = new SpcInstruction { Name = "PUSH Y", AddrMode = AddrImplied, Operate = OpPUSH_Y, Cycles = 4 };
            _instructions[0x4D] = new SpcInstruction { Name = "PUSH X", AddrMode = AddrImplied, Operate = OpPUSH_X, Cycles = 4 };
            _instructions[0xEE] = new SpcInstruction { Name = "POP Y", AddrMode = AddrImplied, Operate = OpPOP_Y, Cycles = 4 };

            // --- Boot Exit ---
            _instructions[0x1F] = new SpcInstruction { Name = "JMP [abs+X]", AddrMode = AddrAbsoluteIndirectX, Operate = OpJMP, Cycles = 6 };
            _instructions[0x5F] = new SpcInstruction { Name = "JMP abs", AddrMode = AddrAbsolute, Operate = OpJMP_abs, Cycles = 3 };

            // --- Driver Initialization & Flag Control ---
            _instructions[0x20] = new SpcInstruction { Name = "CLRP", AddrMode = AddrImplied, Operate = OpCLRP, Cycles = 2 };
            _instructions[0x40] = new SpcInstruction { Name = "SETP", AddrMode = AddrImplied, Operate = OpSETP, Cycles = 2 };
            _instructions[0xA0] = new SpcInstruction { Name = "EI", AddrMode = AddrImplied, Operate = OpEI, Cycles = 3 };
            _instructions[0xC0] = new SpcInstruction { Name = "DI", AddrMode = AddrImplied, Operate = OpDI, Cycles = 3 };
            _instructions[0xE0] = new SpcInstruction { Name = "CLRV", AddrMode = AddrImplied, Operate = OpCLRV, Cycles = 2 };
            _instructions[0x60] = new SpcInstruction { Name = "CLRC", AddrMode = AddrImplied, Operate = OpCLRC, Cycles = 2 };
            _instructions[0x80] = new SpcInstruction { Name = "SETC", AddrMode = AddrImplied, Operate = OpSETC, Cycles = 2 };
            _instructions[0x5E] = new SpcInstruction { Name = "CMP Y, abs", AddrMode = AddrAbsolute, Operate = OpCMP_Y_dp, Cycles = 4 };
            _instructions[0xAD] = new SpcInstruction { Name = "CMP Y, #imm", AddrMode = AddrImmediate, Operate = OpCMP_Y_dp, Cycles = 2 };
            _instructions[0x1E] = new SpcInstruction { Name = "CMP X, abs", AddrMode = AddrAbsolute, Operate = OpCMP_X_Imm, Cycles = 4 };
            _instructions[0x3E] = new SpcInstruction { Name = "CMP X, dp", AddrMode = AddrDirectPage, Operate = OpCMP_X_Imm, Cycles = 3 };
            _instructions[0x64] = new SpcInstruction { Name = "CMP A, dp", AddrMode = AddrDirectPage, Operate = OpCMP_A, Cycles = 3 };
            _instructions[0x65] = new SpcInstruction { Name = "CMP A, abs", AddrMode = AddrAbsolute, Operate = OpCMP_A, Cycles = 4 };
            _instructions[0x74] = new SpcInstruction { Name = "CMP A, dp+X", AddrMode = AddrDirectPageX, Operate = OpCMP_A, Cycles = 4 };
            _instructions[0xFE] = new SpcInstruction { Name = "DBNZ Y, rel", AddrMode = AddrRelative, Operate = OpDBNZ_Y, Cycles = 4 };
            _instructions[0x6E] = new SpcInstruction { Name = "DBNZ dp, rel", AddrMode = AddrDirectPage, Operate = OpDBNZ_dp, Cycles = 5 };
            _instructions[0xC9] = new SpcInstruction { Name = "MOV abs, X", AddrMode = AddrAbsolute, Operate = OpMOV_mem_X, Cycles = 5 };
            _instructions[0xD8] = new SpcInstruction { Name = "MOV dp, X", AddrMode = AddrDirectPage, Operate = OpMOV_mem_X, Cycles = 4 };
            _instructions[0xE9] = new SpcInstruction { Name = "MOV X, abs", AddrMode = AddrAbsolute, Operate = OpMOV_X_Imm, Cycles = 4 };
            _instructions[0xE6] = new SpcInstruction { Name = "MOV A, (X)", AddrMode = AddrIndX, Operate = OpMOV_A_indX, Cycles = 3 };
            _instructions[0xF8] = new SpcInstruction { Name = "MOV X, dp", AddrMode = AddrDirectPage, Operate = OpMOV_X_Imm, Cycles = 3 };
            _instructions[0xF6] = new SpcInstruction { Name = "MOV A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpMOV_A_abs, Cycles = 5 };
            _instructions[0xD6] = new SpcInstruction { Name = "MOV abs+Y, A", AddrMode = AddrAbsoluteIndexedY, Operate = OpMOV_abs_A, Cycles = 6 };
            _instructions[0xF7] = new SpcInstruction { Name = "MOV A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpMOV_A_abs, Cycles = 6 };
            _instructions[0xE7] = new SpcInstruction { Name = "MOV A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpMOV_A_dp, Cycles = 6 };
            _instructions[0x88] = new SpcInstruction { Name = "ADC A, #imm", AddrMode = AddrImmediate, Operate = OpADC_A, Cycles = 2 };
            _instructions[0x76] = new SpcInstruction { Name = "CMP A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpCMP_A, Cycles = 5 };
            _instructions[0xD0] = new SpcInstruction { Name = "BNE rel", AddrMode = AddrRelative, Operate = OpBNE, Cycles = 2 };
            _instructions[0x69] = new SpcInstruction { Name = "CMP dp(d), dp(s)", AddrMode = AddrImplied, Operate = OpCMP_dp_dp, Cycles = 6 };
            _instructions[0x44] = new SpcInstruction { Name = "EOR A, dp", AddrMode = AddrDirectPage, Operate = OpEOR_A, Cycles = 3 };
            _instructions[0xED] = new SpcInstruction { Name = "NOTC", AddrMode = AddrImplied, Operate = OpNOTC, Cycles = 3 };
            _instructions[0x4C] = new SpcInstruction { Name = "LSR abs", AddrMode = AddrAbsolute, Operate = OpLSR_dp, Cycles = 5 };
            _instructions[0x26] = new SpcInstruction { Name = "AND A, (X)", AddrMode = AddrIndirectX, Operate = OpAND_A, Cycles = 3 };
            _instructions[0x04] = new SpcInstruction { Name = "OR A, dp", AddrMode = AddrDirectPage, Operate = OpOR_A, Cycles = 3 };
            _instructions[0x2B] = new SpcInstruction { Name = "ROL dp", AddrMode = AddrDirectPage, Operate = OpROL_dp, Cycles = 4 };

            
           
            // No Operation
            _instructions[0x00] = new SpcInstruction { Name = "NOP", AddrMode = AddrImplied, Operate = OpNOP, Cycles = 2 };

            for (int bit = 0; bit < 8; bit++)
            {
                int b = bit; // capture per-iteration copy for the lambdas
                _instructions[0x02 + b * 0x20] = new SpcInstruction { Name = $"SET1 dp.{b}", AddrMode = AddrDirectPage, Operate = addr => OpSetClrBit(addr, b, true), Cycles = 4 };
                _instructions[0x12 + b * 0x20] = new SpcInstruction { Name = $"CLR1 dp.{b}", AddrMode = AddrDirectPage, Operate = addr => OpSetClrBit(addr, b, false), Cycles = 4 };
                _instructions[0x03 + b * 0x20] = new SpcInstruction { Name = $"BBS dp.{b}, rel", AddrMode = AddrDirectPage, Operate = addr => OpBranchBit(addr, b, true), Cycles = 5 };
                _instructions[0x13 + b * 0x20] = new SpcInstruction { Name = $"BBC dp.{b}, rel", AddrMode = AddrDirectPage, Operate = addr => OpBranchBit(addr, b, false), Cycles = 5 };
            }

            for (int i = 0; i < 16; i++)
            {
                int n = i; // Capture loop variable for the lambda
                _instructions[0x01 + (n * 0x10)] = new SpcInstruction { Name = $"TCALL {n}", AddrMode = AddrImplied, Operate = addr => OpTCALL(addr, n), Cycles = 8 };
            }

            // --- OR A, <mode> - remaining addressing modes ---
            _instructions[0x06] = new SpcInstruction { Name = "OR A, (X)", AddrMode = AddrIndirectX, Operate = OpOR_A, Cycles = 3 };
            _instructions[0x07] = new SpcInstruction { Name = "OR A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpOR_A, Cycles = 6 };
            _instructions[0x14] = new SpcInstruction { Name = "OR A, dp+X", AddrMode = AddrDirectPageX, Operate = OpOR_A, Cycles = 4 };
            _instructions[0x15] = new SpcInstruction { Name = "OR A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpOR_A, Cycles = 5 };
            _instructions[0x16] = new SpcInstruction { Name = "OR A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpOR_A, Cycles = 5 };
            _instructions[0x17] = new SpcInstruction { Name = "OR A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpOR_A, Cycles = 6 };

            // --- EOR A, <mode> - remaining addressing modes ---
            _instructions[0x45] = new SpcInstruction { Name = "EOR A, abs", AddrMode = AddrAbsolute, Operate = OpEOR_A, Cycles = 4 };
            _instructions[0x46] = new SpcInstruction { Name = "EOR A, (X)", AddrMode = AddrIndirectX, Operate = OpEOR_A, Cycles = 3 };
            _instructions[0x47] = new SpcInstruction { Name = "EOR A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpEOR_A, Cycles = 6 };
            _instructions[0x54] = new SpcInstruction { Name = "EOR A, dp+X", AddrMode = AddrDirectPageX, Operate = OpEOR_A, Cycles = 4 };
            _instructions[0x55] = new SpcInstruction { Name = "EOR A, abs+X", AddrMode = AddrAbsoluteIndexedX, Operate = OpEOR_A, Cycles = 5 };
            _instructions[0x56] = new SpcInstruction { Name = "EOR A, abs+Y", AddrMode = AddrAbsoluteIndexedY, Operate = OpEOR_A, Cycles = 5 };
            _instructions[0x57] = new SpcInstruction { Name = "EOR A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpEOR_A, Cycles = 6 };

            // --- CMP A, <mode> - remaining addressing modes ---
            _instructions[0x66] = new SpcInstruction { Name = "CMP A, (X)", AddrMode = AddrIndirectX, Operate = OpCMP_A, Cycles = 3 };
            _instructions[0x67] = new SpcInstruction { Name = "CMP A, [dp+X]", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpCMP_A, Cycles = 6 };
            _instructions[0x77] = new SpcInstruction { Name = "CMP A, [dp]+Y", AddrMode = AddrDirectIndirectIndexedY, Operate = OpCMP_A, Cycles = 6 };

            // --- Shift/rotate memory - remaining addressing modes (all reuse
            // the existing generic Op*_dp handlers, which just Read8/Write8
            // at whatever address the AddrMode resolves) ---
            _instructions[0x1B] = new SpcInstruction { Name = "ASL dp+X", AddrMode = AddrDirectPageX, Operate = OpASL_dp, Cycles = 5 };
            _instructions[0x2C] = new SpcInstruction { Name = "ROL abs", AddrMode = AddrAbsolute, Operate = OpROL_dp, Cycles = 5 };
            _instructions[0x3B] = new SpcInstruction { Name = "ROL dp+X", AddrMode = AddrDirectPageX, Operate = OpROL_dp, Cycles = 5 };
            _instructions[0x3C] = new SpcInstruction { Name = "ROL A", AddrMode = AddrImplied, Operate = OpROL_A, Cycles = 2 };
            _instructions[0x5B] = new SpcInstruction { Name = "LSR dp+X", AddrMode = AddrDirectPageX, Operate = OpLSR_dp, Cycles = 5 };
            _instructions[0x7B] = new SpcInstruction { Name = "ROR dp+X", AddrMode = AddrDirectPageX, Operate = OpROR_dp, Cycles = 5 };

            // --- dd,ds ALU family (source byte encoded first, destination
            // second - see OpAND_dp_dp etc. in Spc700.Opcodes.cs) ---
            _instructions[0x29] = new SpcInstruction { Name = "AND dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpAND_dp_dp, Cycles = 6 };
            _instructions[0x49] = new SpcInstruction { Name = "EOR dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpEOR_dp_dp, Cycles = 6 };
            _instructions[0x89] = new SpcInstruction { Name = "ADC dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpADC_dp_dp, Cycles = 6 };
            _instructions[0xA9] = new SpcInstruction { Name = "SBC dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpSBC_dp_dp, Cycles = 6 };
            _instructions[0xFA] = new SpcInstruction { Name = "MOV dp(d), dp(s)", AddrMode = AddrDirectPage, Operate = OpMOV_dp_dp, Cycles = 5 };

            // --- (X),(Y) family ---
            _instructions[0x19] = new SpcInstruction { Name = "OR (X), (Y)", AddrMode = AddrImplied, Operate = OpOR_IndX_IndY, Cycles = 5 };
            _instructions[0x39] = new SpcInstruction { Name = "AND (X), (Y)", AddrMode = AddrImplied, Operate = OpAND_IndX_IndY, Cycles = 5 };
            _instructions[0x59] = new SpcInstruction { Name = "EOR (X), (Y)", AddrMode = AddrImplied, Operate = OpEOR_IndX_IndY, Cycles = 5 };
            _instructions[0x79] = new SpcInstruction { Name = "CMP (X), (Y)", AddrMode = AddrImplied, Operate = OpCMP_IndX_IndY, Cycles = 5 };
            _instructions[0x99] = new SpcInstruction { Name = "ADC (X), (Y)", AddrMode = AddrImplied, Operate = OpADC_IndX_IndY, Cycles = 5 };
            _instructions[0xB9] = new SpcInstruction { Name = "SBC (X), (Y)", AddrMode = AddrImplied, Operate = OpSBC_IndX_IndY, Cycles = 5 };

            // --- m.b single-bit family ---
            _instructions[0x0A] = new SpcInstruction { Name = "OR1 C, m.b", AddrMode = AddrMemBit, Operate = OpOR1, Cycles = 5 };
            _instructions[0x2A] = new SpcInstruction { Name = "OR1 C, /m.b", AddrMode = AddrMemBit, Operate = OpOR1Not, Cycles = 5 };
            _instructions[0x4A] = new SpcInstruction { Name = "AND1 C, m.b", AddrMode = AddrMemBit, Operate = OpAND1, Cycles = 4 };
            _instructions[0x6A] = new SpcInstruction { Name = "AND1 C, /m.b", AddrMode = AddrMemBit, Operate = OpAND1Not, Cycles = 4 };
            _instructions[0x8A] = new SpcInstruction { Name = "EOR1 C, m.b", AddrMode = AddrMemBit, Operate = OpEOR1, Cycles = 5 };
            _instructions[0xAA] = new SpcInstruction { Name = "MOV1 C, m.b", AddrMode = AddrMemBit, Operate = OpMOV1_C_mb, Cycles = 4 };
            _instructions[0xCA] = new SpcInstruction { Name = "MOV1 m.b, C", AddrMode = AddrMemBit, Operate = OpMOV1_mb_C, Cycles = 6 };
            _instructions[0xEA] = new SpcInstruction { Name = "NOT1 m.b", AddrMode = AddrMemBit, Operate = OpNOT1, Cycles = 5 };

            // --- Branches missing their table entries (handlers already existed) ---
            _instructions[0x50] = new SpcInstruction { Name = "BVC rel", AddrMode = AddrRelative, Operate = OpBVC, Cycles = 2 };
            _instructions[0x70] = new SpcInstruction { Name = "BVS rel", AddrMode = AddrRelative, Operate = OpBVS, Cycles = 2 };

            // --- MOV d+Y family ---
            _instructions[0xD9] = new SpcInstruction { Name = "MOV dp+Y, X", AddrMode = AddrDirectPageY, Operate = OpMOV_mem_X, Cycles = 5 };
            _instructions[0xF9] = new SpcInstruction { Name = "MOV X, dp+Y", AddrMode = AddrDirectPageY, Operate = OpMOV_X_Imm, Cycles = 4 };

            // --- MOV [dp+X], A - reuses the existing generic write-A-to-address handler ---
            _instructions[0xC7] = new SpcInstruction { Name = "MOV [dp+X], A", AddrMode = AddrDirectPageIndexedXIndirect, Operate = OpMOV_IndX_A, Cycles = 7 };

            // --- MOV A, (X)+ - mirrors the existing MOV (X)+, A (0xAF) ---
            _instructions[0xBF] = new SpcInstruction { Name = "MOV A, (X)+", AddrMode = AddrIndirectX, Operate = OpMOV_A_IndXInc, Cycles = 4 };

            // --- Remaining standalone opcodes ---
            _instructions[0x0F] = new SpcInstruction { Name = "BRK", AddrMode = AddrImplied, Operate = OpBRK, Cycles = 8 };
            _instructions[0x9D] = new SpcInstruction { Name = "MOV X, SP", AddrMode = AddrImplied, Operate = OpMOV_X_SP, Cycles = 2 };
            _instructions[0xBE] = new SpcInstruction { Name = "DAS A", AddrMode = AddrImplied, Operate = OpDAS_A, Cycles = 3 };
            _instructions[0xDF] = new SpcInstruction { Name = "DAA A", AddrMode = AddrImplied, Operate = OpDAA_A, Cycles = 3 };
            _instructions[0xEF] = new SpcInstruction { Name = "SLEEP", AddrMode = AddrImplied, Operate = OpHalt, Cycles = 2 };
            _instructions[0xFF] = new SpcInstruction { Name = "STOP", AddrMode = AddrImplied, Operate = OpHalt, Cycles = 2 };
        }

    }
}
