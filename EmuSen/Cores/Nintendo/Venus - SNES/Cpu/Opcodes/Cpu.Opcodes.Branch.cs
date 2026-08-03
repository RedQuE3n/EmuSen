using System;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus.Processor
{
    // Every opcode that changes PC (and sometimes PB): unconditional jumps/
    // calls (JMP/JML/JSR/JSL/RTS/RTL/BRA/BRL) and every conditional branch
    // (BPL/BNE/BEQ/BVS/BVC/BCC/BCS/BMI).
    public partial class Cpu
    {
        private void OpJSR(uint address)
        {
            Push16((ushort)(PC - 1));
            CallStack?.NotePush((LastInstructionPB << 16) | LastInstructionPC, (PB << 16) | (int)(address & 0xFFFF), CallFrameKind.Call);
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpRTS(uint address)
        {
            CallStack?.NotePop();
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
            CallStack?.NotePush((LastInstructionPB << 16) | LastInstructionPC, (int)(address & 0xFFFFFF), CallFrameKind.Call);
            PB = (byte)(address >> 16);
            PC = (ushort)(address & 0xFFFF);
        }

        private void OpRTL(uint address)
        {
            CallStack?.NotePop();
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

        private void OpBVC(uint address)
        {
            if (!GetFlag(CpuFlags.V))
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
    }
}
