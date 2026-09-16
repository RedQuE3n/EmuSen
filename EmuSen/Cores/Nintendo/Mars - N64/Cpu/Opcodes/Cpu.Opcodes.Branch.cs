using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Branches, jumps, and what a likely branch does with the slot behind it - see Mars_Cpu.md §3.
    public sealed partial class Cpu
    {
        private void BranchIf(bool taken, uint instruction, bool likely, bool link = false)
        {
            // The link happens whether or not the branch is taken, which is the trap in this family.
            if (link) Write(31, NextPc);

            if (taken)
            {
                Branch(unchecked(Pc + (ulong)(SignedImmediate(instruction) << 2)));
                return;
            }

            if (likely) NullifyDelaySlot();
        }

        private void Jump(uint instruction) => Branch(JumpTarget(instruction));

        private void JumpAndLink(uint instruction)
        {
            Write(31, NextPc);
            Branch(JumpTarget(instruction));
        }

        private void JumpRegister(uint instruction, bool link)
        {
            ulong target = Read(Rs(instruction));
            if (link) Write(Rd(instruction), NextPc);

            Branch(target);
        }

        // The target replaces the low 28 bits of the address the delay slot sits at.
        private ulong JumpTarget(uint instruction) => (Pc & 0xFFFF_FFFF_F000_0000) | ((instruction & 0x03FF_FFFF) << 2);
    }
}
