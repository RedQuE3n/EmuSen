using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // Just enough of coprocessor zero to move values and to let the corpus start - see Mars_Cpu.md §8.
    public sealed partial class Cpu
    {
        public const int IndexRegister = 0;
        public const int RandomRegister = 1;
        public const int EntryLo0Register = 2;
        public const int EntryLo1Register = 3;
        public const int ContextRegister = 4;
        public const int PageMaskRegister = 5;
        public const int WiredRegister = 6;
        public const int BadVirtualAddressRegister = 8;
        public const int EntryHiRegister = 10;
        public const int LinkedAddressRegister = 17;
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

        // Which coprocessor a fault names, and zero for every fault that names none - see Mars_Fpu.md §3.1.
        public const ulong CauseCoprocessor = 3UL << 28;

        // The two hardware lines that reach this core: the RCP's aggregate, and the counter's own.
        public const ulong CauseInterruptRcp = 1UL << 10;
        public const ulong CauseInterruptTimer = 1UL << 15;

        private const int InterruptShift = 8;

        // Where a handler lives, and the separate door a TLB refill comes through - see Mars_Cpu.md §11.1.
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

                switch (funct)
                {
                    case 0x01: ReadTlbEntry(); return;
                    case 0x02: WriteTlbEntry((int)(Cop0[IndexRegister] & 0x1F)); return;
                    case 0x06: WriteTlbEntry(RandomIndex()); return;
                    case 0x08: ProbeTlb(); return;
                    case 0x18: ReturnFromException(); return;
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
                _lastCount = _bus.Count;
                return;
            }

            Cop0[register] = value;

            // Writing the comparison value is how a handler acknowledges the timer - see Mars_Cpu.md §12.1.
            if (register == CompareRegister)
            {
                Cop0[CauseRegister] &= ~CauseInterruptTimer;
                _lastCount = _bus.Count;
            }
        }

        // Level-triggered from the aggregator, then edge-latched from the counter - see Mars_Cpu.md §12.
        private void CheckInterrupts()
        {
            if (_bus.Mi.Asserted) Cop0[CauseRegister] |= CauseInterruptRcp;
            else Cop0[CauseRegister] &= ~CauseInterruptRcp;

            if ((Cop0[StatusRegister] & StatusInterruptEnable) == 0) return;
            if ((Cop0[StatusRegister] & (StatusExceptionLevel | StatusErrorLevel)) != 0) return;

            ulong pending = (Cop0[CauseRegister] >> InterruptShift) & (Cop0[StatusRegister] >> InterruptShift) & 0xFF;
            if (pending != 0) throw Raise(ExceptionCode.Interrupt, CurrentPc);
        }

        // Hardware compares for equality; a clock that can step by more than one has to ask about the interval.
        private void UpdateTimer()
        {
            uint now = _bus.Count;
            uint compare = (uint)Cop0[CompareRegister];

            if (Crossed(_lastCount, now, compare)) Cop0[CauseRegister] |= CauseInterruptTimer;

            _lastCount = now;
        }

        private static bool Crossed(uint previous, uint now, uint target)
        {
            if (previous == now) return false;

            // The counter wraps, so "between" is two ranges rather than one - see Mars_Cpu.md §12.1.
            return previous < now
                ? target > previous && target <= now
                : target > previous || target <= now;
        }
    }
}

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    public sealed partial class Cpu
    {
        // Everything an exception does to the machine before the handler's first instruction - see Mars_Cpu.md §11.
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
            SetCauseCoprocessor(raised.Coprocessor);

            if (IsAddressRelated(raised.Code)) Cop0[BadVirtualAddressRegister] = raised.Address;

            Cop0[StatusRegister] |= StatusExceptionLevel;
            LinkedFlag = false;

            Pc = VectorFor(raised.Refill, alreadyHandling);
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

        // A refill has its own vector only on the way in from ordinary execution - see Mars_Cpu.md §11.1.
        private ulong VectorFor(bool refill, bool alreadyHandling)
        {
            ulong start = (Cop0[StatusRegister] & StatusBootstrapVectors) != 0 ? VectorBaseBootstrap : VectorBase;

            return start + (refill && !alreadyHandling ? VectorOffsetTlbRefill : VectorOffsetGeneral);
        }

        private void SetCauseCode(ExceptionCode code) =>
            Cop0[CauseRegister] = (Cop0[CauseRegister] & ~0x7CUL) | ((ulong)code << 2);

        private void SetCauseCoprocessor(int coprocessor) =>
            Cop0[CauseRegister] = (Cop0[CauseRegister] & ~CauseCoprocessor) | ((ulong)coprocessor << 28);

        private void SetCauseBranchDelay(bool inDelaySlot) =>
            Cop0[CauseRegister] = inDelaySlot
                ? Cop0[CauseRegister] | CauseBranchDelay
                : Cop0[CauseRegister] & ~CauseBranchDelay;

        // Copies the indexed entry out into the registers a handler reads.
        private void ReadTlbEntry()
        {
            ref TlbEntry entry = ref Tlb.Entries[Cop0[IndexRegister] & 0x1F];

            Cop0[PageMaskRegister] = entry.PageMask;
            Cop0[EntryHiRegister] = entry.EntryHi;
            Cop0[EntryLo0Register] = entry.EntryLo0;
            Cop0[EntryLo1Register] = entry.EntryLo1;
        }

        private void WriteTlbEntry(int index)
        {
            ref TlbEntry entry = ref Tlb.Entries[index & 0x1F];

            entry.PageMask = Cop0[PageMaskRegister] & 0x01FF_E000;
            entry.EntryHi = Cop0[EntryHiRegister];
            entry.EntryLo0 = Cop0[EntryLo0Register];
            entry.EntryLo1 = Cop0[EntryLo1Register];
        }

        // A failed probe sets the top bit rather than an index, which is how a handler tells them apart.
        private void ProbeTlb()
        {
            int found = Tlb.Probe(Cop0[EntryHiRegister]);
            Cop0[IndexRegister] = found < 0 ? 0x8000_0000UL : (ulong)found;
        }

        // The entry a random write lands on, kept above the wired entries a handler reserves for itself.
        private int RandomIndex()
        {
            int wired = (int)(Cop0[WiredRegister] & 0x1F);
            if (wired >= Tlb.EntryCount) return Tlb.EntryCount - 1;

            int span = Tlb.EntryCount - wired;
            return wired + (int)((ulong)Instructions % (ulong)span);
        }

        // What a refill handler reads instead of recomputing the address itself - see Mars_Tlb.md §4.
        private void OnTlbFailure(ulong address)
        {
            Cop0[ContextRegister] = (Cop0[ContextRegister] & ~0x007F_FFF0UL) | ((address >> 9) & 0x007F_FFF0UL);
            Cop0[EntryHiRegister] = (Cop0[EntryHiRegister] & 0xFF) | (address & 0xFFFF_E000UL);
        }

        private static bool IsAddressRelated(ExceptionCode code) =>
            code is ExceptionCode.AddressErrorLoad or ExceptionCode.AddressErrorStore
                or ExceptionCode.TlbLoad or ExceptionCode.TlbStore or ExceptionCode.TlbModification;
    }
}
