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
                ulong target = unchecked(Pc + (ulong)(SignedImmediate(instruction) << 2));
                if (link) CallObserver?.Invoke(CurrentPc, target);
                Branch(target);
                return;
            }

            if (likely)
            {
                NullifyDelaySlot();
                return;
            }

            // The slot behind an untaken ordinary branch is still a delay slot - see Mars_Cpu.md §3.2.
            _branchPending = true;
        }

        private void Jump(uint instruction) => Branch(JumpTarget(instruction));

        private void JumpAndLink(uint instruction)
        {
            Write(31, NextPc);
            CallObserver?.Invoke(CurrentPc, JumpTarget(instruction));
            Branch(JumpTarget(instruction));
        }

        // A jalr is a call and a jr through ra a return, the conventions `bt` reads - see Mars_Debug.md §2.
        private void JumpRegister(uint instruction, bool link)
        {
            ulong target = Read(Rs(instruction));
            if (link) Write(Rd(instruction), NextPc);

            if (link) CallObserver?.Invoke(CurrentPc, target);
            else if (Rs(instruction) == 31 && ReturnObserver != null) _returnAfterSlot = true;

            Branch(target);
        }

        // The target replaces the low 28 bits of the address the delay slot sits at.
        private ulong JumpTarget(uint instruction) => (Pc & 0xFFFF_FFFF_F000_0000) | ((instruction & 0x03FF_FFFF) << 2);
    }
}
