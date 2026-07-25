using System;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus.Apu
{
    public partial class Spc700
    {
        // --- Operations ---

        // Not sure what the hell these two do
        private void OpUnknown(ushort address) { }

        private void OpNOP(ushort address) { }
        private void OpBEQ(ushort address) { TakeBranch(address, GetFlag(SpcFlags.Z)); }
        private void OpBNE(ushort address) { TakeBranch(address, !GetFlag(SpcFlags.Z)); }
        private void OpBMI(ushort address) { TakeBranch(address, GetFlag(SpcFlags.N)); }
        private void OpBPL(ushort address) { TakeBranch(address, !GetFlag(SpcFlags.N)); }
        private void OpBVS(ushort address) { TakeBranch(address, GetFlag(SpcFlags.V)); }
        private void OpBVC(ushort address) { TakeBranch(address, !GetFlag(SpcFlags.V)); }
        private void OpBRA(ushort address) { TakeBranch(address, true); } // Branch Always

        // Beginning of the methods again

        // --- Operations ---

        private void OpAND_dp_imm(ushort address)
        {
            byte immediate = Read8(address);
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddr = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte result = (byte)(Read8(dpAddr) & immediate);
            Write8(dpAddr, result);
            UpdateZN(result);
        }

        private void OpEOR_dp_imm(ushort address)
        {
            byte immediate = Read8(address);
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddr = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte result = (byte)(Read8(dpAddr) ^ immediate);
            Write8(dpAddr, result);
            UpdateZN(result);
        }

        private void OpSBC_dp_imm(ushort address)
        {
            byte immediate = Read8(address);
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddr = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte directPageVal = Read8(dpAddr);
            int borrow = GetFlag(SpcFlags.C) ? 0 : 1;
            int result = directPageVal - immediate - borrow;
            
            SetFlag(SpcFlags.H, ((directPageVal & 0x0F) - (immediate & 0x0F) - borrow) >= 0);
            SetFlag(SpcFlags.V, ((directPageVal ^ immediate) & (directPageVal ^ result) & 0x80) != 0);
            SetFlag(SpcFlags.C, result >= 0);
            
            byte resultByte = (byte)(result & 0xFF);
            Write8(dpAddr, resultByte);
            UpdateZN(resultByte);
        }

        private void OpROL_dp(ushort address)
        {
            byte val = Read8(address);
            int carryIn = GetFlag(SpcFlags.C) ? 1 : 0;
            int carryOut = (val & 0x80) != 0 ? 1 : 0;
            byte result = (byte)((val << 1) | carryIn);
            
            Write8(address, result);
            SetFlag(SpcFlags.C, carryOut != 0);
            UpdateZN(result);
        }

        private void OpTSET1_abs(ushort address)
        {
            byte memVal = Read8(address);
            UpdateZN((byte)(A - memVal));
            Write8(address, (byte)(memVal | A));
        }

        private void OpTCLR1_abs(ushort address)
        {
            byte memVal = Read8(address);
            UpdateZN((byte)(A - memVal));
            Write8(address, (byte)(memVal & ~A));
        }
        private void OpOR_A(ushort address)
        {
            byte val = Read8(address);
            A |= val;
            UpdateZN(A);
        }

        private void OpNOTC(ushort address)
        {
            SetFlag(SpcFlags.C, !GetFlag(SpcFlags.C));
        }
        private void OpCMP_dp_dp(ushort _)
        {
            // 1. Fetch the source and destination direct page offsets directly from the PC stream
            // (Ensure this matches your PC incrementing logic. If your fetch loop already 
            // increments PC past the opcode, PC now points to the srcOffset).
            byte srcOffset = Read8(PC++);
            byte dstOffset = Read8(PC++);
        
            // 2. Resolve the Direct Page base address (P flag)
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
        
            // 3. Read the actual values from the resolved direct page addresses
            byte srcVal = Read8((ushort)(dpBase + srcOffset));
            byte dstVal = Read8((ushort)(dpBase + dstOffset));
        
            // 4. Perform the comparison (Destination - Source)
            int result = dstVal - srcVal;
        
            // 5. Update Flags: Carry, Zero, Negative 
            SetFlag(SpcFlags.C, dstVal >= srcVal);
            SetFlag(SpcFlags.Z, (byte)result == 0);
            SetFlag(SpcFlags.N, (result & 0x80) != 0);
        }
        private void OpMOV_A_indX(ushort address)
        {
            // Read the value from the resolved effective address
            A = Read8(address);
            
            // Update Zero and Negative flags based on the loaded value
            SetFlag(SpcFlags.Z, A == 0);
            SetFlag(SpcFlags.N, (A & 0x80) != 0);
        }

        private void OpSUBW_YA_dp(ushort address)
        {
            // Read 16-bit word (little-endian) from the direct page
            ushort operand = (ushort)(Read8(address) | (Read8((ushort)(address + 1)) << 8));
            
            // Reconstruct the 16-bit YA register
            ushort ya = (ushort)((Y << 8) | A);
            
            int result = ya - operand; // Note: SUBW does not use Carry as an input
            
            SetFlag(SpcFlags.H, (ya & 0x0FFF) >= (operand & 0x0FFF));
            SetFlag(SpcFlags.V, ((ya ^ operand) & (ya ^ result) & 0x8000) != 0);
            SetFlag(SpcFlags.C, result >= 0);
            
            ya = (ushort)(result & 0xFFFF);
            
            A = (byte)(ya & 0xFF);
            Y = (byte)(ya >> 8);
            
            SetFlag(SpcFlags.Z, ya == 0);
            SetFlag(SpcFlags.N, (ya & 0x8000) != 0);
        }

        private void OpCMPW_YA_dp(ushort address)
        {
            // Read 16-bit word (little-endian) from the direct page
            ushort memVal = (ushort)(Read8(address) | (Read8((ushort)(address + 1)) << 8));
            
            // Reconstruct the 16-bit YA register
            ushort ya = (ushort)((Y << 8) | A);
            
            // Perform the comparison (YA - memVal)
            int result = ya - memVal;
            
            // Update flags
            SetFlag(SpcFlags.C, ya >= memVal);
            SetFlag(SpcFlags.Z, (result & 0xFFFF) == 0);
            SetFlag(SpcFlags.N, (result & 0x8000) != 0);
        }

        private void OpPCALL(ushort address)
        {
            byte upageOffset = Read8(address);
            
            // Push current PC to stack
            Push8((byte)(PC >> 8));
            Push8((byte)(PC & 0xFF));
            
            // Jump to $FFxx
            PC = (ushort)(0xFF00 | upageOffset);
        }

        private void OpTCALL(ushort address, int n)
        {
            Push8((byte)(PC >> 8));
            Push8((byte)(PC & 0xFF));
            
            ushort vectorAddr = (ushort)(0xFFDE - (n * 2));
            
            byte low = Read8(vectorAddr);
            byte high = Read8((ushort)(vectorAddr + 1));
            
            PC = (ushort)((high << 8) | low);
        }

        private void OpRETI(ushort address)
        {
            PSW = Pop8();
            byte low = Pop8();
            byte high = Pop8();
            
            PC = (ushort)((high << 8) | low);
        }

        private void OpPUSH_PSW(ushort address)
        {
            Push8(PSW);
        }

        private void OpPOP_PSW(ushort address)
        {
            PSW = Pop8();
        }

        private void OpLSR_dp(ushort address)
        {
            byte val = Read8(address);
            
            SetFlag(SpcFlags.C, (val & 0x01) != 0);
            
            val = (byte)(val >> 1);
            
            Write8(address, val);
            UpdateZN(val);
        }

        private void OpDIV_YA_X(ushort address)
        {
            // Real hardware does NOT compute a plain ya/X - the SPC700's
            // DIV instruction emulates a specific bit-by-bit restoring
            // division circuit, which only produces a plain quotient/
            // remainder when Y < (X<<1); otherwise it overflows partway
            // through and produces a different (still fully documented,
            // deterministic) result. Ported from the widely-referenced
            // algorithm (matches bsnes/higan's implementation) and
            // verified against the TomHarte/ProcessorTests spc700 ground-
            // truth vectors via the SpcValidation harness - a previous
            // "just do ya/X always" version failed ~23% of DIV's own test
            // vectors, exactly the fraction where Y >= (X<<1) doesn't hold.
            ushort ya = (ushort)((Y << 8) | A);

            SetFlag(SpcFlags.H, (Y & 0x0F) >= (X & 0x0F));
            SetFlag(SpcFlags.V, Y >= X);

            if (Y < (X << 1))
            {
                A = (byte)(ya / X);
                Y = (byte)(ya % X);
            }
            else
            {
                A = (byte)(255 - (ya - (X << 9)) / (256 - X));
                Y = (byte)(X + (ya - (X << 9)) % (256 - X));
            }

            UpdateZN(A);
        }
        private void OpADDW_YA_dp(ushort address)
        {
            ushort operand = (ushort)(Read8(address) | (Read8((ushort)(address + 1)) << 8));
            
            ushort ya = (ushort)((Y << 8) | A);
            
            int result = ya + operand;
            
            SetFlag(SpcFlags.H, ((ya & 0x0FFF) + (operand & 0x0FFF)) > 0x0FFF);
            SetFlag(SpcFlags.V, ((~(ya ^ operand)) & (ya ^ result) & 0x8000) != 0);
            SetFlag(SpcFlags.C, result > 0xFFFF);
            
            ya = (ushort)(result & 0xFFFF);

            A = (byte)(ya & 0xFF);
            Y = (byte)(ya >> 8);
            
            SetFlag(SpcFlags.Z, ya == 0);
            SetFlag(SpcFlags.N, (ya & 0x8000) != 0);
        }
        private void OpOR_A_abs(ushort address)
        {
            byte memVal = Read8(address);
            A |= memVal;
            UpdateZN(A);
        }

        private void OpSBC_A(ushort address)
        {
            byte operand = Read8(address);
            int borrow = GetFlag(SpcFlags.C) ? 0 : 1;
            int result = A - operand - borrow;
            
            SetFlag(SpcFlags.H, ((A & 0x0F) - (operand & 0x0F) - borrow) >= 0);
            SetFlag(SpcFlags.V, ((A ^ operand) & (A ^ result) & 0x80) != 0);
            SetFlag(SpcFlags.C, result >= 0);
            
            A = (byte)(result & 0xFF);
            UpdateZN(A);
        }

        private void OpADC_A(ushort address)
        {
            byte operand = Read8(address);
            int carry = GetFlag(SpcFlags.C) ? 1 : 0;
            
            int result = A + operand + carry;
            
            // Half-carry flag (H): Set if carry out from bit 3
            SetFlag(SpcFlags.H, ((A & 0x0F) + (operand & 0x0F) + carry) > 0x0F);
            
            // Overflow flag (V): Set if signs of A and operand are same, but result sign differs
            SetFlag(SpcFlags.V, (~(A ^ operand) & (A ^ result) & 0x80) != 0);
            
            // Carry flag (C): Set if result exceeds 8 bits
            SetFlag(SpcFlags.C, result > 0xFF);
            
            A = (byte)(result & 0xFF);
            UpdateZN(A);
        }

        private void OpOR_dp_dp(ushort srcAddress)
        {
            byte srcVal = Read8(srcAddress);
            
            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
            
            byte destVal = Read8(destAddress);
            byte result = (byte)(destVal | srcVal);
            
            Write8(destAddress, result);
            UpdateZN(result);
        }

        private void OpAND_A(ushort address)
        {
            byte operand = Read8(address);
            A &= operand;
            UpdateZN(A);
        }

        private void OpEOR_A(ushort address)
        {
            byte operand = Read8(address);
            A ^= operand;
            UpdateZN(A);
        }

        private void OpXCN_A(ushort address)
        {
            A = (byte)((A << 4) | (A >> 4));
            
            UpdateZN(A);
        }

        private void OpROR_dp(ushort address)
        {
            byte val = Read8(address);
            bool oldCarry = GetFlag(SpcFlags.C);
            bool newCarry = (val & 0x01) != 0;

            val = (byte)((val >> 1) | (oldCarry ? 0x80 : 0x00));
            
            Write8(address, val);
            SetFlag(SpcFlags.C, newCarry);
            UpdateZN(val);
        }

        private void OpADC_dp_imm(ushort address)
        {
            byte immediate = Read8(address);
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddr = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte directPageVal = Read8(dpAddr);
            int carry = GetFlag(SpcFlags.C) ? 1 : 0;
            int result = directPageVal + immediate + carry;
            byte resultByte = (byte)(result & 0xFF);
            Write8(dpAddr, resultByte);
            SetFlag(SpcFlags.C, result > 0xFF);
            SetFlag(SpcFlags.Z, resultByte == 0);
            SetFlag(SpcFlags.N, (resultByte & 0x80) != 0);
            SetFlag(SpcFlags.H, ((directPageVal & 0x0F) + (immediate & 0x0F) + carry) > 0x0F);
            SetFlag(SpcFlags.V, (~(directPageVal ^ immediate) & (directPageVal ^ resultByte) & 0x80) != 0);
        }

        private void OpOR_dp_imm(ushort address)
        {
            byte immediate = Read8(address);
            byte dpOffset = Read8(PC);
            PC++;
            ushort dpAddr = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte result = (byte)(Read8(dpAddr) | immediate);
            Write8(dpAddr, result);
            UpdateZN(result);
        }

        private void OpJMP_abs(ushort address)
        {
            PC = address;
        }

        private void OpSetClrBit(ushort address, int bit, bool set)
        {
            byte val = Read8(address);
            if (set) { val = (byte)(val | (1 << bit)); }
            else { val = (byte)(val & ~(1 << bit)); }
            Write8(address, val);
        }

        private void OpBranchBit(ushort address, int bit, bool branchIfSet)
        {
            byte val = Read8(address);
            sbyte rel = (sbyte)Read8(PC);
            PC++;

            bool isSet = (val & (1 << bit)) != 0;
            if (isSet == branchIfSet)
            {
                PC = (ushort)(PC + rel);
                CycleBudget -= 2;
            }
        }

        private void TakeBranch(ushort address, bool condition)
        {
            sbyte offset = (sbyte)Read8(address);
            if (condition)
            {
                PC = (ushort)(PC + offset);
                CycleBudget -= 2; // Branches take 2 extra cycles if taken
            }
        }

        private void OpASL_dp(ushort address)
        {
            byte val = Read8(address);
            
            SetFlag(SpcFlags.C, (val & 0x80) != 0);
            
            val = (byte)((val << 1) & 0xFF);
            
            Write8(address, val);
            UpdateZN(val);
        }

        private void OpROR_A(ushort address)
        {
            bool oldCarry = GetFlag(SpcFlags.C);
            
            SetFlag(SpcFlags.C, (A & 0x01) != 0);
            
            A = (byte)((A >> 1) | (oldCarry ? 0x80 : 0x00));
            
            UpdateZN(A);
        }

        private void OpLSR_A(ushort address)
        {
            SetFlag(SpcFlags.C, (A & 0x01) != 0);
            
            A = (byte)(A >> 1);
            
            UpdateZN(A);
        }

        private void OpJMP_AbsIndexedXIndirect(ushort address)
        {
            byte low = Read8(address);
            byte high = Read8((ushort)(address + 1));
    
            PC = (ushort)((high << 8) | low);
        }

        private void OpINCW_dp(ushort address)
        {
            ushort val = (ushort)(Read8(address) | (Read8((ushort)(address + 1)) << 8));
            
            val++;
            
            Write8(address, (byte)(val & 0xFF));
            Write8((ushort)(address + 1), (byte)(val >> 8));
            SetFlag(SpcFlags.Z, val == 0);
            SetFlag(SpcFlags.N, (val & 0x8000) != 0);
        }

        private void OpDECW_dp(ushort address)
        {
            // Read the 16-bit word (little-endian) from the direct page
            ushort val = (ushort)(Read8(address) | (Read8((ushort)(address + 1)) << 8));
            
            // Decrement the word
            val--;
            
            // Write it back
            Write8(address, (byte)(val & 0xFF));
            Write8((ushort)(address + 1), (byte)(val >> 8));
            
            // Update flags based on the 16-bit result
            SetFlag(SpcFlags.Z, val == 0);
            SetFlag(SpcFlags.N, (val & 0x8000) != 0);
        }

        private void OpASL_A(ushort address)
        {
            SetFlag(SpcFlags.C, (A & 0x80) != 0);
            A = (byte)((A << 1) & 0xFF);
    
            UpdateZN(A);
        }

        private void OpMOV_Y_Imm(ushort address)
        {
            Y = Read8(address);
            UpdateZN(Y);
        }

        private void OpMOV_mem_X(ushort address)
        {
            Write8(address, X);
        }

        private void OpOR_A_dp(ushort address)
        {
            byte memVal = Read8(address);
            A |= memVal;
            UpdateZN(A);
        }

        private void OpCBNE_dp_rel(ushort address)
        {
            byte dpVal = Read8(address);
            byte offset = Read8(PC); 
            PC++;

            if (A != dpVal)
            {
                sbyte rel = (sbyte)offset;
                PC = (ushort)(PC + rel);
                CycleBudget -= 2; // Branch taken penalty
            }
        }

        private void OpDBNZ_Y(ushort address)
        {
            sbyte offset = (sbyte)Read8(address);
            Y--;

            if (Y != 0)
            {
                PC = (ushort)(PC + offset);
                CycleBudget -= 2;
            }
        }

        private void OpDBNZ_dp(ushort address)
        {
            byte val = (byte)(Read8(address) - 1);
            Write8(address, val);

            byte offset = Read8(PC);
            PC++;

            if (val != 0)
            {
                PC = (ushort)(PC + (sbyte)offset);
                CycleBudget -= 2;
            }
        }

        private void OpCMP_A(ushort address)
        {
            byte memVal = Read8(address);
            int result = A - memVal;
            
            SetFlag(SpcFlags.C, A >= memVal);
            
            UpdateZN((byte)(result & 0xFF));
        }

        private void OpMOV_A_X(ushort address)
        {
            A = X;
            UpdateZN(A);
        }

        private void OpMOV_A_abs(ushort address)
        {
            A = Read8(address);
            UpdateZN(A);
        }

        private void OpPOP_Y(ushort address)
        {
            Y = Pop8();
        }

        private void OpPOP_X(ushort address)
        {
            X = Pop8();
        }

        private void OpBCC(ushort address)
        {
            sbyte offset = (sbyte)Read8(address);
        
            if (!GetFlag(SpcFlags.C))
            {
                PC = (ushort)(PC + offset);
                CycleBudget -= 2; 
            }
        }

        private void OpBCS(ushort address)
        {
            sbyte offset = (sbyte)Read8(address);
            if (GetFlag(SpcFlags.C))
            {
                PC = (ushort)(PC + offset);
                CycleBudget -= 2; 
            }
        }

        private void OpCLRC(ushort address)
        {
            SetFlag(SpcFlags.C, false);
        }

        private void OpSETC(ushort address)
        {
            SetFlag(SpcFlags.C, true);
        }

        private void OpMUL_YA(ushort address)
        {
            int product = Y * A;
            
            A = (byte)(product & 0xFF);
            Y = (byte)(product >> 8);
            
            UpdateZN(Y);
        }

        private void OpPUSH_Y(ushort address)
        {
            Push8(Y);
        }

        private void OpPUSH_X(ushort address)
        {
            Push8(X);
        }

        private void OpMOV_Y_abs(ushort address)
        {
            Y = Read8(address);
            UpdateZN(Y);
        }

        private void OpRET(ushort address)
        {
            byte low = Pop8();
            byte high = Pop8();
            
            PC = (ushort)((high << 8) | low);
        }

        private void OpMOV_abs_Y(ushort address)
        {
            Write8(address, Y);
        }

        private void OpCALL(ushort address)
        {
            Push8((byte)(PC >> 8));
            Push8((byte)(PC & 0xFF));
            
            PC = address;
        }

        private void OpMOV_Y_A(ushort address)
        {
            Y = A;
            UpdateZN(Y);
        }

        private void OpMOV_A_absX(ushort address)
        {
            A = Read8(address);
            UpdateZN(A);
        }

        private void OpMOV_absX_A(ushort address)
        {
            Write8(address, A);
        }

        private void OpCMP_X_Imm(ushort address)
        {
            byte imm = Read8(address);
            int result = X - imm;
            SetFlag(SpcFlags.C, X >= imm);
            UpdateZN((byte)(result & 0xFF));
        }

        private void OpMOV_IndXInc_A(ushort address)
        {
            Write8(address, A);
            X++;
        }

        private void OpMOV_abs_A(ushort address)
        {
            Write8(address, A);
        }

        private void OpCLRP(ushort address)
        {
            SetFlag(SpcFlags.P, false);
        }

        private void OpSETP(ushort address)
        {
            SetFlag(SpcFlags.P, true);
        }

        private void OpEI(ushort address)
        {
            SetFlag(SpcFlags.I, true);
        }

        private void OpDI(ushort address)
        {
            SetFlag(SpcFlags.I, false);
        }

        private void OpCLRV(ushort address)
        {
            SetFlag(SpcFlags.V, false);
            SetFlag(SpcFlags.H, false); // CLRV clears both Overflow (V) and Half-Carry (H) in the SPC700!
        }

        private void OpMOV_X_Imm(ushort address)
        {
            X = Read8(address);
            UpdateZN(X);
        }

        private void OpMOV_SP_X(ushort address)
        {
            SP = X;
        }

        private void OpMOV_A_Imm(ushort address)
        {
            A = Read8(address);
            UpdateZN(A);
        }

        private void OpMOV_IndX_A(ushort address)
        {
            Write8(address, A);
        }

        private void OpDEC_X(ushort address)
        {
            X--;
            UpdateZN(X);
        }

        private void OpDEC_mem(ushort address)
        {
            byte val = Read8(address);
            val--;
            Write8(address, val);
            UpdateZN(val);
        }

        private void OpDEC_A(ushort address)
        {
            A--;
            UpdateZN(A);
        }

        private void OpDEC_Y(ushort address)
        {
            Y--;
            UpdateZN(Y);
        }

        private void OpMOV_dp_imm(ushort address)
        {
            byte imm = Read8(address);
            byte dpOffset = Read8((ushort)(address + 1));
            
            ushort dpAddress = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
            Write8(dpAddress, imm);
        }

        private void OpCMP_dp_imm(ushort address)
        {
            byte imm = Read8(address);
            byte dpOffset = Read8((ushort)(address + 1));
            
            ushort dpAddress = (ushort)(dpOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));
            byte memVal = Read8(dpAddress);
            
            int result = memVal - imm;
            
            SetFlag(SpcFlags.C, memVal >= imm);
            UpdateZN((byte)(result & 0xFF));
        }

        private void OpMOVW_YA_dp(ushort address)
        {
            A = Read8(address);
            Y = Read8((ushort)(address + 1));
            
            ushort ya = (ushort)((Y << 8) | A);
            
            SetFlag(SpcFlags.Z, ya == 0);
            SetFlag(SpcFlags.N, (ya & 0x8000) != 0);
        }

        private void OpMOVW_dp_YA(ushort address)
        {
            Write8(address, A);
            Write8((ushort)(address + 1), Y);
        }

        private void OpMOV_dp_A(ushort address)
        {
            Write8(address, A);
        }

        private void OpMOV_A_Y(ushort address)
        {
            A = Y;
            UpdateZN(A);
        }

        private void OpMOV_X_A(ushort address)
        {
            X = A;
            UpdateZN(X);
        }

        private void OpMOV_Y_mem(ushort address)
        {
            Y = Read8(address);
            UpdateZN(Y);
        }

        private void OpCMP_Y_dp(ushort address)
        {
            byte memVal = Read8(address);
            int result = Y - memVal;
            
            SetFlag(SpcFlags.C, Y >= memVal);
            UpdateZN((byte)(result & 0xFF));
        }

        private void OpMOV_A_dp(ushort address)
        {
            A = Read8(address);
            UpdateZN(A);
        }

        private void OpMOV_dp_Y(ushort address)
        {
            Write8(address, Y);
        }

        private void OpMOV_IndY_A(ushort address)
        {
            Write8(address, A);
        }

        private void OpINC_dp(ushort address)
        {
            byte val = Read8(address);
            val++;
            Write8(address, val);
            UpdateZN(val);
        }

        private void OpINC_Y(ushort address)
        {
            Y++;
            UpdateZN(Y);
        }

        private void OpINC_A(ushort address)
        {
            A++;
            UpdateZN(A);
        }

        private void OpINC_X(ushort address)
        {
            X++;
            UpdateZN(X);
        }

        private void OpPUSH_A(ushort address)
        {
            Push8(A);
        }

        private void OpPOP_A(ushort address)
        {
            A = Pop8();
        }

        private void OpJMP(ushort address)
        {
            PC = address;
        }

        // --- dd,ds family: (dd) = (dd) OP (ds) - see Venus_APU.md §2.1 ---

        private void OpAND_dp_dp(ushort srcAddress)
        {
            byte srcVal = Read8(srcAddress);

            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte destVal = Read8(destAddress);
            byte result = (byte)(destVal & srcVal);

            Write8(destAddress, result);
            UpdateZN(result);
        }

        private void OpEOR_dp_dp(ushort srcAddress)
        {
            byte srcVal = Read8(srcAddress);

            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte destVal = Read8(destAddress);
            byte result = (byte)(destVal ^ srcVal);

            Write8(destAddress, result);
            UpdateZN(result);
        }

        private void OpADC_dp_dp(ushort srcAddress)
        {
            byte srcVal = Read8(srcAddress);

            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte destVal = Read8(destAddress);
            int carry = GetFlag(SpcFlags.C) ? 1 : 0;
            int result = destVal + srcVal + carry;
            byte resultByte = (byte)(result & 0xFF);

            Write8(destAddress, resultByte);
            SetFlag(SpcFlags.H, ((destVal & 0x0F) + (srcVal & 0x0F) + carry) > 0x0F);
            SetFlag(SpcFlags.V, (~(destVal ^ srcVal) & (destVal ^ resultByte) & 0x80) != 0);
            SetFlag(SpcFlags.C, result > 0xFF);
            UpdateZN(resultByte);
        }

        private void OpSBC_dp_dp(ushort srcAddress)
        {
            byte srcVal = Read8(srcAddress);

            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            byte destVal = Read8(destAddress);
            int borrow = GetFlag(SpcFlags.C) ? 0 : 1;
            int result = destVal - srcVal - borrow;
            byte resultByte = (byte)(result & 0xFF);

            Write8(destAddress, resultByte);
            SetFlag(SpcFlags.H, ((destVal & 0x0F) - (srcVal & 0x0F) - borrow) >= 0);
            SetFlag(SpcFlags.V, ((destVal ^ srcVal) & (destVal ^ resultByte) & 0x80) != 0);
            SetFlag(SpcFlags.C, result >= 0);
            UpdateZN(resultByte);
        }

        private void OpMOV_dp_dp(ushort srcAddress)
        {
            // "(no read)" of destination per hardware docs - see Venus_APU.md §2.1.
            byte srcVal = Read8(srcAddress);

            byte destOffset = Read8(PC);
            PC++;
            ushort destAddress = (ushort)(destOffset + (GetFlag(SpcFlags.P) ? 0x0100 : 0x0000));

            Write8(destAddress, srcVal);
        }

        // --- (X),(Y) family: (X) = (X) OP (Y) - see Venus_APU.md §2.2 ---

        private void OpOR_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            ushort addrX = (ushort)(dpBase + X);
            ushort addrY = (ushort)(dpBase + Y);
            byte result = (byte)(Read8(addrX) | Read8(addrY));
            Write8(addrX, result);
            UpdateZN(result);
        }

        private void OpAND_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            ushort addrX = (ushort)(dpBase + X);
            ushort addrY = (ushort)(dpBase + Y);
            byte result = (byte)(Read8(addrX) & Read8(addrY));
            Write8(addrX, result);
            UpdateZN(result);
        }

        private void OpEOR_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            ushort addrX = (ushort)(dpBase + X);
            ushort addrY = (ushort)(dpBase + Y);
            byte result = (byte)(Read8(addrX) ^ Read8(addrY));
            Write8(addrX, result);
            UpdateZN(result);
        }

        private void OpCMP_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            byte valX = Read8((ushort)(dpBase + X));
            byte valY = Read8((ushort)(dpBase + Y));
            int result = valX - valY;

            SetFlag(SpcFlags.C, valX >= valY);
            SetFlag(SpcFlags.Z, (byte)result == 0);
            SetFlag(SpcFlags.N, (result & 0x80) != 0);
        }

        private void OpADC_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            ushort addrX = (ushort)(dpBase + X);
            ushort addrY = (ushort)(dpBase + Y);
            byte valX = Read8(addrX);
            byte valY = Read8(addrY);
            int carry = GetFlag(SpcFlags.C) ? 1 : 0;
            int result = valX + valY + carry;
            byte resultByte = (byte)(result & 0xFF);

            Write8(addrX, resultByte);
            SetFlag(SpcFlags.H, ((valX & 0x0F) + (valY & 0x0F) + carry) > 0x0F);
            SetFlag(SpcFlags.V, (~(valX ^ valY) & (valX ^ resultByte) & 0x80) != 0);
            SetFlag(SpcFlags.C, result > 0xFF);
            UpdateZN(resultByte);
        }

        private void OpSBC_IndX_IndY(ushort address)
        {
            ushort dpBase = GetFlag(SpcFlags.P) ? (ushort)0x0100 : (ushort)0x0000;
            ushort addrX = (ushort)(dpBase + X);
            ushort addrY = (ushort)(dpBase + Y);
            byte valX = Read8(addrX);
            byte valY = Read8(addrY);
            int borrow = GetFlag(SpcFlags.C) ? 0 : 1;
            int result = valX - valY - borrow;
            byte resultByte = (byte)(result & 0xFF);

            Write8(addrX, resultByte);
            SetFlag(SpcFlags.H, ((valX & 0x0F) - (valY & 0x0F) - borrow) >= 0);
            SetFlag(SpcFlags.V, ((valX ^ valY) & (valX ^ resultByte) & 0x80) != 0);
            SetFlag(SpcFlags.C, result >= 0);
            UpdateZN(resultByte);
        }

        // --- m.b family - see Venus_APU.md §2.3 ---

        private void OpOR1(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, GetFlag(SpcFlags.C) | memBit);
        }

        private void OpOR1Not(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, GetFlag(SpcFlags.C) | !memBit);
        }

        private void OpAND1(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, GetFlag(SpcFlags.C) & memBit);
        }

        private void OpAND1Not(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, GetFlag(SpcFlags.C) & !memBit);
        }

        private void OpEOR1(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, GetFlag(SpcFlags.C) ^ memBit);
        }

        private void OpMOV1_C_mb(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            bool memBit = (Read8(memAddr) & (1 << bit)) != 0;
            SetFlag(SpcFlags.C, memBit);
        }

        private void OpMOV1_mb_C(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            byte val = Read8(memAddr);
            val = GetFlag(SpcFlags.C) ? (byte)(val | (1 << bit)) : (byte)(val & ~(1 << bit));
            Write8(memAddr, val);
        }

        private void OpNOT1(ushort raw)
        {
            ushort memAddr = (ushort)(raw & 0x1FFF);
            int bit = (raw >> 13) & 0x07;
            byte val = Read8(memAddr);
            val = (byte)(val ^ (1 << bit));
            Write8(memAddr, val);
        }

        // --- Remaining standalone opcodes ---

        private void OpROL_A(ushort address)
        {
            int carryIn = GetFlag(SpcFlags.C) ? 1 : 0;
            SetFlag(SpcFlags.C, (A & 0x80) != 0);
            A = (byte)((A << 1) | carryIn);
            UpdateZN(A);
        }

        private void OpMOV_X_SP(ushort address)
        {
            X = SP;
            UpdateZN(X);
        }

        private void OpMOV_A_IndXInc(ushort address)
        {
            // Mirrors OpMOV_IndXInc_A (0xAF, "MOV (X)+, A") - same AddrIndirectX
            // addressing (current dp+X address, not yet incremented), just
            // read-then-increment instead of write-then-increment.
            A = Read8(address);
            X++;
            UpdateZN(A);
        }

        // DAA/DAS - see Venus_APU.md §2.4.
        private void OpDAA_A(ushort address)
        {
            if ((A & 0x0F) > 9 || GetFlag(SpcFlags.H))
            {
                A = (byte)(A + 0x06);
            }
            if (A > 0x99 || GetFlag(SpcFlags.C))
            {
                A = (byte)(A + 0x60);
                SetFlag(SpcFlags.C, true);
            }
            UpdateZN(A);
        }

        private void OpDAS_A(ushort address)
        {
            if ((A & 0x0F) > 9 || !GetFlag(SpcFlags.H))
            {
                A = (byte)(A - 0x06);
            }
            if (A > 0x99 || !GetFlag(SpcFlags.C))
            {
                A = (byte)(A - 0x60);
                SetFlag(SpcFlags.C, false);
            }
            UpdateZN(A);
        }

        private void OpBRK(ushort address)
        {
            Push8((byte)(PC >> 8));
            Push8((byte)(PC & 0xFF));
            Push8(PSW);
            SetFlag(SpcFlags.B, true);
            SetFlag(SpcFlags.I, false);

            byte low = Read8(0xFFDE);
            byte high = Read8(0xFFDF);
            PC = (ushort)((high << 8) | low);
        }

        // Shared by SLEEP (0xEF) and STOP (0xFF) - both just halt the CPU
        // until Reset(); see _halted in Spc700.cs.
        private void OpHalt(ushort address)
        {
            _halted = true;
        }
    }
}