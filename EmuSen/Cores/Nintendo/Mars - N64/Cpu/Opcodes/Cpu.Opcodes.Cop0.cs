using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Just enough of coprocessor zero to move values and to let the corpus start - see Mars_Cpu.md §8.
    public sealed partial class Cpu
    {
        public const int BadVirtualAddressRegister = 8;
        public const int CountRegister = 9;
        public const int CompareRegister = 11;
        public const int StatusRegister = 12;
        public const int CauseRegister = 13;
        public const int ExceptionPcRegister = 14;
        public const int ErrorExceptionPcRegister = 30;

        public const ulong StatusInterruptEnable = 1UL << 0;
        public const ulong StatusExceptionLevel = 1UL << 1;
        public const ulong StatusErrorLevel = 1UL << 2;
        public const ulong StatusBootstrapVectors = 1UL << 22;

        public const ulong CauseBranchDelay = 1UL << 31;

        // Where a handler lives, and the separate door a TLB refill comes through - see Mars_Cpu.md §9.1.
        public const ulong VectorBase = 0xFFFF_FFFF_8000_0000;
        public const ulong VectorBaseBootstrap = 0xFFFF_FFFF_BFC0_0200;
        public const ulong VectorOffsetTlbRefill = 0x000;
        public const ulong VectorOffsetGeneral = 0x180;

        public readonly ulong[] Cop0 = new ulong[32];

        private void ExecuteCop0(uint instruction)
        {
            uint rs = (uint)Rs(instruction);

            if ((rs & 0x10) != 0)
            {
                uint funct = instruction & 0x3F;

                // The emulator extensions, which real hardware ignores and the corpus calls unconditionally.
                if (funct >= 0x20) return;

                if (funct == 0x18)
                {
                    ReturnFromException();
                    return;
                }

                throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }

            switch (rs)
            {
                case 0x00: Write32(Rt(instruction), (uint)ReadCop0(Rd(instruction))); return;
                case 0x01: Write(Rt(instruction), ReadCop0(Rd(instruction))); return;
                case 0x04: WriteCop0(Rd(instruction), (ulong)(long)(int)(uint)Read(Rt(instruction))); return;
                case 0x05: WriteCop0(Rd(instruction), Read(Rt(instruction))); return;
                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        // Count comes off the machine clock rather than out of storage - see Mars_Memory.md §3.1.
        private ulong ReadCop0(int register) =>
            register == CountRegister ? _bus.Count : Cop0[register];

        private void WriteCop0(int register, ulong value)
        {
            if (register == CountRegister)
            {
                _bus.SetCount((uint)value);
                return;
            }

            Cop0[register] = value;
        }
    }
}

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    public sealed partial class Cpu
    {
        // Everything an exception does to the machine before the handler's first instruction - see Mars_Cpu.md §9.
        private void EnterException(CpuException raised)
        {
            bool alreadyHandling = (Cop0[StatusRegister] & StatusExceptionLevel) != 0;

            // A fault inside a handler keeps the original return address, which is what makes nesting survivable.
            if (!alreadyHandling)
            {
                Cop0[ExceptionPcRegister] = raised.InDelaySlot ? CurrentPc - 4 : CurrentPc;
                SetCauseBranchDelay(raised.InDelaySlot);
            }

            SetCauseCode(raised.Code);

            if (IsAddressRelated(raised.Code)) Cop0[BadVirtualAddressRegister] = raised.Address;

            Cop0[StatusRegister] |= StatusExceptionLevel;

            Pc = VectorFor(raised.Code, alreadyHandling);
            NextPc = Pc + 4;
            _branchPending = false;
            InDelaySlot = false;
        }

        private void ReturnFromException()
        {
            if ((Cop0[StatusRegister] & StatusErrorLevel) != 0)
            {
                Pc = Cop0[ErrorExceptionPcRegister];
                Cop0[StatusRegister] &= ~StatusErrorLevel;
            }
            else
            {
                Pc = Cop0[ExceptionPcRegister];
                Cop0[StatusRegister] &= ~StatusExceptionLevel;
            }

            // No delay slot of its own: the next instruction fetched is the one returned to.
            NextPc = Pc + 4;
            _branchPending = false;
        }

        // A refill has its own vector only on the way in from ordinary execution - see Mars_Cpu.md §9.1.
        private ulong VectorFor(ExceptionCode code, bool alreadyHandling)
        {
            ulong start = (Cop0[StatusRegister] & StatusBootstrapVectors) != 0 ? VectorBaseBootstrap : VectorBase;
            bool refill = !alreadyHandling && code is ExceptionCode.TlbLoad or ExceptionCode.TlbStore;

            return start + (refill ? VectorOffsetTlbRefill : VectorOffsetGeneral);
        }

        private void SetCauseCode(ExceptionCode code) =>
            Cop0[CauseRegister] = (Cop0[CauseRegister] & ~0x7CUL) | ((ulong)code << 2);

        private void SetCauseBranchDelay(bool inDelaySlot) =>
            Cop0[CauseRegister] = inDelaySlot
                ? Cop0[CauseRegister] | CauseBranchDelay
                : Cop0[CauseRegister] & ~CauseBranchDelay;

        private static bool IsAddressRelated(ExceptionCode code) =>
            code is ExceptionCode.AddressErrorLoad or ExceptionCode.AddressErrorStore
                or ExceptionCode.TlbLoad or ExceptionCode.TlbStore or ExceptionCode.TlbModification;
    }
}
