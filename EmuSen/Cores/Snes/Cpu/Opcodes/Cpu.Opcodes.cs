using System;
using EmuSen.Memory;
using EmuSen.Debug;

namespace EmuSen.Processor
{
    public partial class Cpu
    {
        // --- Operations ---

        private void OpUnknown(uint address) { }

        private void OpNOP(uint address) { }
        
        private void OpPHP(uint address)
        {
            Push8(P);
        }

        private void OpBRK(uint address)
        {
            PC++; // Skip the signature byte

            if (!E) Push8(PB);
            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.I, true);
            SetFlag(CpuFlags.D, false); // BRK automatically clears Decimal mode

            ushort vectorAddr = E ? (ushort)0xFFFE : (ushort)0xFFE6;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);
        }

        private void OpCOP(uint address)
        {
            PC++; // Skip the signature byte

            if (!E) Push8(PB);
            Push16(PC);
            Push8(P);

            SetFlag(CpuFlags.I, true);
            SetFlag(CpuFlags.D, false); 

            ushort vectorAddr = E ? (ushort)0xFFF4 : (ushort)0xFFE4;
            byte low = _bus.Read8(vectorAddr);
            byte high = _bus.Read8((uint)(vectorAddr + 1));

            PB = 0x00;
            PC = (ushort)((high << 8) | low);
        }

        private void OpBVC(uint address)
        {
            if (!GetFlag(CpuFlags.V))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpMVN(uint address)
        {
            byte destBank = (byte)(address >> 8);
            byte srcBank = (byte)(address & 0xFF);
            
            // The Data Bank (DB) is updated to the destination bank during a block move
            DB = destBank;
            
            // Read from Source, Write to Destination
            byte val = _bus.Read8((uint)((srcBank << 16) | X));
            _bus.Write8((uint)((destBank << 16) | Y), val);
            
            // MVN moves forward: Increment X and Y
            if (IsIndex8Bit)
            {
                X = (ushort)((X + 1) & 0xFF);
                Y = (ushort)((Y + 1) & 0xFF);
            }
            else
            {
                X++;
                Y++;
            }
            
            // Decrement the 16-bit Accumulator counter
            A--;
            
            // If the counter hasn't rolled over past 0, loop the instruction
            if (A != 0xFFFF)
            {
                PC -= 3;
            }
        }

        private void OpMVP(uint address)
        {
            byte destBank = (byte)(address >> 8);
            byte srcBank = (byte)(address & 0xFF);
            
            DB = destBank;
            
            byte val = _bus.Read8((uint)((srcBank << 16) | X));
            _bus.Write8((uint)((destBank << 16) | Y), val);
            
            if (IsIndex8Bit)
            {
                X = (ushort)((X - 1) & 0xFF);
                Y = (ushort)((Y - 1) & 0xFF);
            }
            else
            {
                X--;
                Y--;
            }
            
            A--;
            
            if (A != 0xFFFF)
            {
                PC -= 3;
            }
        }

        private void OpTRB(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & val) == 0);
                
                // Reset the bits specified by the Accumulator
                val = (byte)(val & ~(A & 0xFF));
                _bus.Write8(address, val);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & val) == 0);
                
                val = (ushort)(val & ~A);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
            }
        }

        private void OpTSB(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & val) == 0);
                
                val = (byte)(val | (A & 0xFF));
                _bus.Write8(address, val);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & val) == 0);
                
                val = (ushort)(val | A);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
            }
        }

        private void OpASL(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x80) != 0); // Bit 7 goes to Carry
                val = (byte)(val << 1);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x8000) != 0); // Bit 15 goes to Carry
                val = (ushort)(val << 1);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpPLP(uint address)
        {
            P = Pop8();
            
            if (E)
            {
                SetFlag(CpuFlags.M, true);
                SetFlag(CpuFlags.X, true);
            }
            if (GetFlag(CpuFlags.X))
            {
                X &= 0x00FF;
                Y &= 0x00FF;
            }
        }

        private void OpRTI(uint address)
        {
            P = Pop8();
            PC = Pop16();

            if (!E)
            {
                PB = Pop8();
            }

            if (E)
            {
                SetFlag(CpuFlags.M, true);
                SetFlag(CpuFlags.X, true);
            }
            if (GetFlag(CpuFlags.X))
            {
                X &= 0x00FF;
                Y &= 0x00FF;
            }
        }

        private void OpPHA(uint address)
        {
            if (IsMemory8Bit)
            {
                Push8((byte)(A & 0xFF));
            }
            else
            {
                Push16(A);
            }
        }

        private void OpPEA(uint address)
        {
            // PEA (Push Effective Absolute): Fetches a 16-bit immediate value
            // and pushes it directly to the stack.
            ushort data = Fetch16();
            Push16(data);
        }

        private void OpPEI(uint address)
        {
            byte dpOffset = Fetch8();
            ushort dpAddr = (ushort)((D + dpOffset) & 0xFFFF);
            
            ushort data = (ushort)(_bus.Read8(dpAddr) | (_bus.Read8((uint)(dpAddr + 1)) << 8));
            Push16(data);
        }

        private void OpPER(uint address)
        {
            short offset = (short)Fetch16();
            ushort data = (ushort)(PC + offset);
            Push16(data);
        }

        private void OpPLA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = Pop8();
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                A = Pop16();
                UpdateZN(A, false);
            }
        }

        private void OpPHX(uint address)
        {
            if (IsIndex8Bit) Push8((byte)(X & 0xFF));
            else Push16(X);
        }

        private void OpPHY(uint address)
        {
            if (IsIndex8Bit) Push8((byte)(Y & 0xFF));
            else Push16(Y);
        }

        private void OpPLX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = Pop8();
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                X = Pop16();
                UpdateZN(X, false);
            }
        }

        private void OpPLY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = Pop8();
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                Y = Pop16();
                UpdateZN(Y, false);
            }
        }

        private void OpPHB(uint address)
        {
            Push8(DB);
        }

        private void OpPLB(uint address)
        {
            DB = Pop8();
            UpdateZN(DB, true);
        }

        private void OpPHK(uint address)
        {
            Push8(PB);
        }

        private void OpPHD(uint address)
        {
            Push16(D);
        }

        private void OpPLD(uint address)
        {
            D = Pop16();
            UpdateZN(D, false);
        }

        private void OpINCA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((A + 1) & 0xFF);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                A = (ushort)((A + 1) & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpDECA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((A - 1) & 0xFF);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                A = (ushort)((A - 1) & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpINCMem(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((_bus.Read8(address) + 1) & 0xFF);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                val = (ushort)((val + 1) & 0xFFFF);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpDECMem(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = (byte)((_bus.Read8(address) - 1) & 0xFF);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                val = (ushort)((val - 1) & 0xFFFF);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpROLMem(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x80) != 0);
                val = (byte)((val << 1) | carryIn);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x8000) != 0);
                val = (ushort)((val << 1) | carryIn);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpRORA(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x01) != 0);
                a8 = (byte)((a8 >> 1) | (carryIn << 7));
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x0001) != 0);
                A = (ushort)((A >> 1) | (carryIn << 15));
                UpdateZN(A, false);
            }
        }

        private void OpRORMem(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x01) != 0); // Bit 0 falls into Carry
                val = (byte)((val >> 1) | (carryIn << 7)); // Old Carry shifts into Bit 7
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x0001) != 0); // Bit 0 falls into Carry
                val = (ushort)((val >> 1) | (carryIn << 15)); // Old Carry shifts into Bit 15
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }

        private void OpBIT(uint address)
        {
            // BIT (non-immediate): Z from A & mem; N and V copied from the top two
            // bits of the memory operand (bits 7/6 in 8-bit mode, 15/14 in 16-bit).
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & operand) == 0);
                SetFlag(CpuFlags.N, (operand & 0x80) != 0);
                SetFlag(CpuFlags.V, (operand & 0x40) != 0);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & operand) == 0);
                SetFlag(CpuFlags.N, (operand & 0x8000) != 0);
                SetFlag(CpuFlags.V, (operand & 0x4000) != 0);
            }
        }

        private void OpBITImm(uint address)
        {
            // BIT immediate is the special case: it ONLY affects Z, leaving N and V alone.
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                SetFlag(CpuFlags.Z, ((A & 0xFF) & operand) == 0);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.Z, (A & operand) == 0);
            }
        }

        private void OpROLA(uint address)
        {
            int carryIn = GetFlag(CpuFlags.C) ? 1 : 0;
            
            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x80) != 0); 
                a8 = (byte)((a8 << 1) | carryIn);      
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x8000) != 0); 
                A = (ushort)((A << 1) | carryIn);       
                UpdateZN(A, false);
            }
        }

        private void OpASLA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x80) != 0);
                a8 = (byte)(a8 << 1);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x8000) != 0);
                A = (ushort)(A << 1);
                UpdateZN(A, false);
            }
        }

        private void OpLSRA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte a8 = (byte)(A & 0xFF);
                SetFlag(CpuFlags.C, (a8 & 0x01) != 0);
                a8 = (byte)(a8 >> 1);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                SetFlag(CpuFlags.C, (A & 0x0001) != 0);
                A = (ushort)(A >> 1);
                UpdateZN(A, false);
            }
        }

        private void OpLSR(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                SetFlag(CpuFlags.C, (val & 0x01) != 0); // Bit 0 goes to Carry
                val = (byte)(val >> 1);
                _bus.Write8(address, val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                SetFlag(CpuFlags.C, (val & 0x0001) != 0); // Bit 0 goes to Carry
                val = (ushort)(val >> 1);
                _bus.Write8(address, (byte)(val & 0xFF));
                _bus.Write8(address + 1, (byte)(val >> 8));
                UpdateZN(val, false);
            }
        }
        
        private void OpJSR(uint address)
        {
            Push16((ushort)(PC - 1));
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpRTS(uint address)
        {
            PC = (ushort)(Pop16() + 1);
        }

        private void OpJMP(uint address)
        {
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpJML(uint address)
        {
            PB = (byte)(address >> 16);
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpJSL(uint address)
        {
            Push8(PB);
            Push16((ushort)(PC - 1));
            PB = (byte)(address >> 16);
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpRTL(uint address)
        {
            ushort newPC = Pop16();
            byte newPB = Pop8();
            PC = (ushort)(newPC + 1);
            PB = newPB;
        }

        private void OpBRA(uint address)
        {
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpBRL(uint address)
        {
            // Same as BRA - unconditional, always taken - just reached via the wider
            // 16-bit offset computed in AddrRelativeLong.
            PC = (ushort)(address & 0xFFFF);
        }
        
        private void OpBPL(uint address)
        {
            if (!GetFlag(CpuFlags.N))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBNE(uint address)
        {
            if (!GetFlag(CpuFlags.Z))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBEQ(uint address)
        {
            if (GetFlag(CpuFlags.Z))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBVS(uint address)
        {
            if (GetFlag(CpuFlags.V))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBCC(uint address)
        {
            if (!GetFlag(CpuFlags.C))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBCS(uint address)
        {
            if (GetFlag(CpuFlags.C))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpBMI(uint address)
        {
            if (GetFlag(CpuFlags.N))
            {
                PC = (ushort)(address & 0xFFFF);
            }
        }

        private void OpCLI(uint address) { SetFlag(CpuFlags.I, false); }
        private void OpSEI(uint address) { SetFlag(CpuFlags.I, true); }
        private void OpCLC(uint address) { SetFlag(CpuFlags.C, false); }
        private void OpSEC(uint address) { SetFlag(CpuFlags.C, true); }
        private void OpCLV(uint address) { SetFlag(CpuFlags.V, false); }
        private void OpCLD(uint address) { SetFlag(CpuFlags.D, false); }
        private void OpSED(uint address) { SetFlag(CpuFlags.D, true); }

        // WAI: halts instruction fetch/execute until any interrupt condition
        // wakes the CPU. Per documented 65816 behavior (verified via
        // 6502.org/apprize.best's "Programming the 65816" excerpt and the
        // NESDev WAI thread), NMI always wakes AND services it; IRQ wakes it
        // even while masked (I=1), but only actually jumps to the vector if
        // unmasked - if masked, execution just resumes at the instruction
        // after WAI with no interrupt serviced. See Step()/Nmi()/Irq() in
        // Cpu.cs for where _waitingForInterrupt is checked/cleared.
        private void OpWAI(uint address) { _waitingForInterrupt = true; }

        // STP: stops the CPU entirely. Real hardware only wakes on a hardware
        // RESET; here that's Cpu.Reset(), which clears _stopped. No other
        // instruction or interrupt clears this - matches real hardware, where
        // STP is meant for power-down, not a resumable pause.
        private void OpSTP(uint address) { _stopped = true; }

        private void OpTCS(uint address)
        {
            if (E) S = (ushort)(0x0100 | (A & 0xFF));
            else S = A;
        }
        
        private void OpTCD(uint address)
        {
            D = A;
            UpdateZN(D, false); 
        }

        private void OpXCE(uint address)
        {
            bool oldCarry = GetFlag(CpuFlags.C);
            SetFlag(CpuFlags.C, E);
            E = oldCarry;

            if (E) 
            {
                SetFlag(CpuFlags.M, true);
                SetFlag(CpuFlags.X, true);
                X &= 0x00FF; // Clear X high byte
                Y &= 0x00FF; // Clear Y high byte
                S = (ushort)(0x0100 | (S & 0xFF)); // Force stack to Page 1
                Console.WriteLine("[CPU] Switched to Emulation Mode (E=1). M and X flags forced to 1.");
            }
        }

        private void OpSTZ(uint address)
        {
            if (IsMemory8Bit) _bus.Write8(address, 0x00);
            else
            {
                _bus.Write8(address, 0x00);
                _bus.Write8(address + 1, 0x00);
            }
        }

        private void OpLDA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte val = _bus.Read8(address);
                A = (ushort)((A & 0xFF00) | val);
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = val;
                UpdateZN(val, false);
            }
        }

        private void OpLDX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = _bus.Read8(address);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                X = val;
                UpdateZN(val, false);
            }
        }

        private void OpLDY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = _bus.Read8(address);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                ushort val = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                Y = val;
                UpdateZN(val, false);
            }
        }

        private void OpSTA(uint address)
        {
            if (IsMemory8Bit)
            {
                _bus.Write8(address, (byte)(A & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(A & 0xFF));
                _bus.Write8(address + 1, (byte)(A >> 8));
            }
        }

        private void OpSTX(uint address)
        {
            if (IsIndex8Bit)
            {
                _bus.Write8(address, (byte)(X & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(X & 0xFF));
                _bus.Write8(address + 1, (byte)(X >> 8));
            }
        }

        private void OpSTY(uint address)
        {
            if (IsIndex8Bit)
            {
                _bus.Write8(address, (byte)(Y & 0xFF));
            }
            else
            {
                _bus.Write8(address, (byte)(Y & 0xFF));
                _bus.Write8(address + 1, (byte)(Y >> 8));
            }
        }

        private void OpTXA(uint address)
        {
            if (IsMemory8Bit)
            {
                A = (ushort)((A & 0xFF00) | (X & 0xFF));
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                A = X;
                UpdateZN(A, false);
            }
        }

        private void OpTXY(uint address)
        {
            if (IsIndex8Bit)
            {
                Y = (ushort)(X & 0xFF);
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                Y = X;
                UpdateZN(Y, false);
            }
        }

        private void OpTYX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(Y & 0xFF);
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                X = Y;
                UpdateZN(X, false);
            }
        }

        private void OpTDC(uint address)
        {
            // Always full 16-bit regardless of M flag.
            A = D;
            UpdateZN(A, false);
        }

        private void OpTSC(uint address)
        {
            // Always full 16-bit regardless of M flag.
            A = S;
            UpdateZN(A, false);
        }

        private void OpTXS(uint address)
        {
            // No flags affected. In emulation mode the stack stays in page 1.
            if (E) S = (ushort)(0x0100 | (X & 0xFF));
            else S = X;
        }

        private void OpTSX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(S & 0xFF);
                UpdateZN((ushort)(S & 0xFF), true);
            }
            else
            {
                X = S;
                UpdateZN(X, false);
            }
        }

        private void OpTAX(uint address)
        {
            if (IsIndex8Bit)
            {
                X = (ushort)(A & 0xFF);
                UpdateZN((ushort)(X & 0xFF), true);
            }
            else
            {
                X = A;
                UpdateZN(X, false);
            }
        }

        private void OpTYA(uint address)
        {
            if (IsMemory8Bit)
            {
                A = (ushort)((A & 0xFF00) | (Y & 0xFF));
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                A = Y;
                UpdateZN(A, false);
            }
        }

        private void OpTAY(uint address)
        {
            if (IsIndex8Bit)
            {
                Y = (ushort)(A & 0xFF);
                UpdateZN((ushort)(Y & 0xFF), true);
            }
            else
            {
                Y = A;
                UpdateZN(Y, false);
            }
        }

        private void OpINX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((X + 1) & 0xFF);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                X = (ushort)((X + 1) & 0xFFFF);
                UpdateZN(X, false);
            }
        }

        private void OpINY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((Y + 1) & 0xFF);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                Y = (ushort)((Y + 1) & 0xFFFF);
                UpdateZN(Y, false);
            }
        }

        private void OpDEX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((X - 1) & 0xFF);
                X = val;
                UpdateZN(val, true);
            }
            else
            {
                X = (ushort)((X - 1) & 0xFFFF);
                UpdateZN(X, false);
            }
        }

        private void OpDEY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte val = (byte)((Y - 1) & 0xFF);
                Y = val;
                UpdateZN(val, true);
            }
            else
            {
                Y = (ushort)((Y - 1) & 0xFFFF);
                UpdateZN(Y, false);
            }
        }

        // UPDATED: CMP with Targeted APU Debugger
        private void OpCMP(uint address)
        {
            if (DebugSettings.CpuVerboseLogging && address == 0x002140)
            {
                ushort memVal = IsMemory8Bit ? _bus.Read8(address) : (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                Console.WriteLine($"[DEBUG APU] CMP against $2140 | Accumulator (A) = 0x{A:X4} | Memory = 0x{memVal:X4} | 8-Bit M-Flag = {IsMemory8Bit}");
            }

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 - operand;
                
                SetFlag(CpuFlags.C, a8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A - operand;
                
                SetFlag(CpuFlags.C, A >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpCPX(uint address)
        {
            if (IsIndex8Bit)
            {
                byte operand = _bus.Read8(address);
                byte x8 = (byte)(X & 0xFF);
                int result = x8 - operand;
                
                SetFlag(CpuFlags.C, x8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = X - operand;
                
                SetFlag(CpuFlags.C, X >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpCPY(uint address)
        {
            if (IsIndex8Bit)
            {
                byte operand = _bus.Read8(address);
                byte y8 = (byte)(Y & 0xFF);
                int result = y8 - operand;
                
                SetFlag(CpuFlags.C, y8 >= operand);
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = Y - operand;
                
                SetFlag(CpuFlags.C, Y >= operand);
                UpdateZN((ushort)(result & 0xFFFF), false);
            }
        }

        private void OpORA(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) | operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A | operand);
                UpdateZN(A, false);
            }
        }

        private void OpAND(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) & operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A & operand);
                UpdateZN(A, false);
            }
        }

        private void OpEOR(uint address)
        {
            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)((A & 0xFF) ^ operand);
                A = (ushort)((A & 0xFF00) | a8);
                UpdateZN(a8, true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                A = (ushort)(A ^ operand);
                UpdateZN(A, false);
            }
        }

        private void OpADC(uint address)
        {
            int carry = GetFlag(CpuFlags.C) ? 1 : 0;

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 + operand + carry;

                SetFlag(CpuFlags.C, result > 0xFF);
                SetFlag(CpuFlags.V, ((~(a8 ^ operand)) & (a8 ^ result) & 0x80) != 0);
                
                A = (ushort)((A & 0xFF00) | (result & 0xFF));
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A + operand + carry;

                SetFlag(CpuFlags.C, result > 0xFFFF);
                SetFlag(CpuFlags.V, ((~(A ^ operand)) & (A ^ result) & 0x8000) != 0);
                
                A = (ushort)(result & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpSBC(uint address)
        {
            int borrow = GetFlag(CpuFlags.C) ? 0 : 1;

            if (IsMemory8Bit)
            {
                byte operand = _bus.Read8(address);
                byte a8 = (byte)(A & 0xFF);
                int result = a8 - operand - borrow;

                SetFlag(CpuFlags.C, result >= 0);
                SetFlag(CpuFlags.V, ((a8 ^ operand) & (a8 ^ result) & 0x80) != 0);
                
                A = (ushort)((A & 0xFF00) | (result & 0xFF));
                UpdateZN((ushort)(result & 0xFF), true);
            }
            else
            {
                ushort operand = (ushort)(_bus.Read8(address) | (_bus.Read8(address + 1) << 8));
                int result = A - operand - borrow;

                SetFlag(CpuFlags.C, result >= 0);
                SetFlag(CpuFlags.V, ((A ^ operand) & (A ^ result) & 0x8000) != 0);
                
                A = (ushort)(result & 0xFFFF);
                UpdateZN(A, false);
            }
        }

        private void OpXBA(uint address)
        {
            byte low = (byte)(A & 0xFF);
            byte high = (byte)(A >> 8);
            A = (ushort)((low << 8) | high);
            
            UpdateZN((ushort)(A & 0xFF), true);
        }

        private void OpREP(uint address)
        {
            byte val = _bus.Read8(address);
            P &= (byte)~val;
            
            if (DebugSettings.CpuVerboseLogging && (!IsMemory8Bit || !IsIndex8Bit))
            {
                Console.WriteLine($"[DEBUG] REP executed. M={GetFlag(CpuFlags.M)}, X={GetFlag(CpuFlags.X)}");
            }
        }

        private void OpSEP(uint address)
        {
            byte val = _bus.Read8(address);
            P |= val;
            
            if (GetFlag(CpuFlags.X))
            {
                X &= 0x00FF;
                Y &= 0x00FF;
            }
        }

    }
}
