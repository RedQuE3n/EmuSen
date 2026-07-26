using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Shell;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Every opcode whose whole job is pushing or pulling the stack: flags
    // (PHP/PLP), A/X/Y (PHA/PLA/PHX/PLX/PHY/PLY), DB/PB/D (PHB/PLB/PHK/PHD/
    // PLD), and the "push an effective address" family (PEA/PEI/PER).
    public partial class Cpu
    {
        private void OpPHP(uint address)
        {
            Push8(P);
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
    }
}
