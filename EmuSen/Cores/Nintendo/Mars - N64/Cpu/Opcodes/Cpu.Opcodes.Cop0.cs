using System;
using EmuSen.Cores.Nintendo.Mars.Memory;

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

        // One 64-bit addressing bit per mode, and the mode field that chooses between them - see Mars_Privilege.md §1.
        public const ulong StatusUserExtendedAddressing = 1UL << 5;
        public const ulong StatusSupervisorExtendedAddressing = 1UL << 6;
        public const ulong StatusKernelExtendedAddressing = 1UL << 7;

        public const ulong StatusReverseEndian = 1UL << 25;
        public const ulong StatusModeField = 3UL << 3;
        public const ulong StatusCop0Usable = 1UL << 28;

        private const int StatusModeShift = 3;

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
        public const ulong VectorOffsetExtendedTlbRefill = 0x080;
        public const ulong VectorOffsetGeneral = 0x180;

        public readonly ulong[] Cop0 = new ulong[32];

        // Read many times an instruction and changed only by a Status write, so it is kept, not recomputed - see Mars_Performance.md §18.
        public PrivilegeMode Mode => _mode;

        [EmuSen.Common.SkipInState] private PrivilegeMode _mode;

        // After every Status write; Debug builds check on every step that none was missed - see Mars_Performance.md §18.
        private void RefreshMode() => _mode = ComputeMode();

        [System.Diagnostics.Conditional("DEBUG")]
        private void VerifyMode()
        {
            if (_mode != ComputeMode()) throw new InvalidOperationException("Status was written outside an instruction without Cop0Written(); the privilege mode is stale.");
        }

        // Kernel whenever an exception is being handled, whatever the mode field says - see Mars_Privilege.md §1.
        private PrivilegeMode ComputeMode() =>
            (Cop0[StatusRegister] & (StatusExceptionLevel | StatusErrorLevel)) != 0
                ? PrivilegeMode.Kernel
                : ((Cop0[StatusRegister] & StatusModeField) >> StatusModeShift) switch
                {
                    1 => PrivilegeMode.Supervisor,
                    2 => PrivilegeMode.User,
                    _ => PrivilegeMode.Kernel,
                };

        public bool WideAddressing => (Cop0[StatusRegister] & Mode switch
        {
            PrivilegeMode.Supervisor => StatusSupervisorExtendedAddressing,
            PrivilegeMode.User => StatusUserExtendedAddressing,
            _ => StatusKernelExtendedAddressing,
        }) != 0;

        // Kernel mode reaches coprocessor zero without permission; the other two need it - see §3.
        private void RequireCop0()
        {
            if (Mode == PrivilegeMode.Kernel) return;
            if ((Cop0[StatusRegister] & StatusCop0Usable) != 0) return;

            throw Raise(ExceptionCode.CoprocessorUnusable, CurrentPc);
        }

        private void ExecuteCop0(uint instruction)
        {
            RequireCop0();

            uint rs = (uint)Rs(instruction);

            // Nothing between the sub-opcode and the function field is decoded at all - see Mars_Cop0.md §10.
            if ((rs & 0x10) != 0)
            {
                switch (instruction & 0x3F)
                {
                    case 0x01: ReadTlbEntry(); return;
                    case 0x02: WriteTlbEntry((int)(Cop0[IndexRegister] & 0x1F)); return;
                    case 0x06: WriteTlbEntry(RandomIndex()); return;
                    case 0x08: ProbeTlb(); return;
                    case 0x18: ReturnFromException(); return;

                    // The R3000's return-from-exception slot is the one reserved code that traps - see §10.1.
                    case 0x10: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
                    default: return;
                }
            }

            switch (rs)
            {
                case 0x00: Write32(Rt(instruction), (uint)ReadCop0(Rd(instruction))); return;
                case 0x01: Write(Rt(instruction), ReadCop0(Rd(instruction))); return;
                case 0x04: WriteCop0(Rd(instruction), Read(Rt(instruction))); return;
                case 0x05: WriteCop0(Rd(instruction), Read(Rt(instruction))); return;

                // Decoded and idle: no control registers to move, and a condition that never holds - see §10.2.
                case 0x02 or 0x06 or 0x08: return;
                default: throw Raise(ExceptionCode.ReservedInstruction, CurrentPc);
            }
        }

        // Level-triggered from the aggregator, then edge-latched from the counter; run only when an input changed - see Mars_Performance.md §10.
        private void CheckInterrupts(bool asserted)
        {
            _recheck = false;
            _assertedSeen = asserted;

            if (asserted) Cop0[CauseRegister] |= CauseInterruptRcp;
            else Cop0[CauseRegister] &= ~CauseInterruptRcp;

            if ((Cop0[StatusRegister] & StatusInterruptEnable) == 0) return;
            if ((Cop0[StatusRegister] & (StatusExceptionLevel | StatusErrorLevel)) != 0) return;

            ulong pending = (Cop0[CauseRegister] >> InterruptShift) & (Cop0[StatusRegister] >> InterruptShift) & 0xFF;
            if (pending != 0) throw Raise(ExceptionCode.Interrupt, CurrentPc);
        }

        // Debug builds prove each skipped check would have done nothing, so a write that bypassed Cop0Written fails loudly - see Mars_Performance.md §10.
        [System.Diagnostics.Conditional("DEBUG")]
        private void VerifySkippedCheck(bool asserted)
        {
            ulong status = Cop0[StatusRegister];
            ulong cause = Cop0[CauseRegister];
            bool enabled = (status & StatusInterruptEnable) != 0 && (status & (StatusExceptionLevel | StatusErrorLevel)) == 0;
            bool pending = ((cause >> InterruptShift) & (status >> InterruptShift) & 0xFF) != 0;

            if (((cause & CauseInterruptRcp) != 0) != asserted || (enabled && pending))
            {
                throw new InvalidOperationException("Status or Cause was written outside an instruction without Cop0Written(); the interrupt check would have acted.");
            }

            if ((uint)Cop0[CompareRegister] != _scheduledCompare || unchecked(_bus.Count - (uint)(_bus.Cycles >> 1)) != _scheduledBias)
            {
                throw new InvalidOperationException("Compare or Count was written outside an instruction without Cop0Written(); the timer is due at the wrong cycle.");
            }
        }

        // For a caller that wrote COP0 or loaded the processor outside an instruction - see Mars_Performance.md §10.
        public void Cop0Written()
        {
            _recheck = true;
            RefreshMode();
            ScheduleTimer();
        }

        // The first cycle the counter reaches Compare, counted from the count the timer last settled at - see Mars_Performance.md §10.
        private void ScheduleTimer()
        {
            uint now = _bus.Count;
            uint compare = (uint)Cop0[CompareRegister];

            // A value equal to the settled count is a whole wrap away, as the interval test found it - see Mars_Cpu.md §12.1.
            uint toCompare = unchecked(compare - _lastCount);
            long owed = toCompare == 0 ? 1L << 32 : toCompare;
            long remaining = owed - unchecked(now - _lastCount);

            _timerDue = remaining <= 0 ? _bus.Cycles : 2 * ((_bus.Cycles >> 1) + remaining);
            _scheduledCompare = compare;
            _scheduledBias = unchecked(now - (uint)(_bus.Cycles >> 1));
        }

        // Hardware compares for equality on every count; this is the first successful step to end on or past it - see Mars_Cpu.md §12.1.
        private void TimerReached()
        {
            Cop0[CauseRegister] |= CauseInterruptTimer;
            _recheck = true;
            _lastCount = _bus.Count;
            ScheduleTimer();
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
            if (raised.Code == ExceptionCode.Interrupt) InterruptObserver?.Invoke();

            bool alreadyHandling = (Cop0[StatusRegister] & StatusExceptionLevel) != 0;

            // Read before the fault raises the exception level, which would make every mode kernel - see Mars_Tlb.md §7.5.
            bool extended = WideAddressing;

            // A fault inside a handler keeps the original return address, which is what makes nesting survivable.
            if (!alreadyHandling)
            {
                Cop0[ExceptionPcRegister] = raised.InDelaySlot ? CurrentPc - 4 : CurrentPc;
                SetCauseBranchDelay(raised.InDelaySlot);
            }

            SetCauseCode(raised.Code);
            SetCauseCoprocessor(raised.Coprocessor);

            if (IsAddressRelated(raised.Code)) RecordFaultingAddress(raised.Address);

            Cop0[StatusRegister] |= StatusExceptionLevel;
            RefreshMode();
            LinkedFlag = false;
            _recheck = true;

            Pc = VectorFor(raised.Refill, alreadyHandling, extended);
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

            // Returning breaks the link but leaves the address a handler can still read - see Mars_Cop0.md §6.
            LinkedFlag = false;
            RefreshMode();
            _recheck = true;

            // No delay slot of its own: the next instruction fetched is the one returned to.
            NextPc = Pc + 4;
            _branchPending = false;
        }

        // A refill has its own vector only on the way in from ordinary execution - see Mars_Cpu.md §11.1.
        private ulong VectorFor(bool refill, bool alreadyHandling, bool extended)
        {
            ulong start = (Cop0[StatusRegister] & StatusBootstrapVectors) != 0 ? VectorBaseBootstrap : VectorBase;

            if (!refill || alreadyHandling) return start + VectorOffsetGeneral;

            return start + (extended ? VectorOffsetExtendedTlbRefill : VectorOffsetTlbRefill);
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

        // An entry keeps less than the registers hold, and one global flag for both halves - see Mars_Tlb.md §7.2.
        private void WriteTlbEntry(int index)
        {
            ref TlbEntry entry = ref Tlb.Entries[index & 0x1F];

            ulong pageMask = Tlb.PairedPageMask(Cop0[PageMaskRegister]);
            ulong global = Cop0[EntryLo0Register] & Cop0[EntryLo1Register] & Tlb.EntryLoGlobal;

            entry.PageMask = pageMask;
            entry.EntryHi = Cop0[EntryHiRegister] & EntryHiWritable & ~pageMask;
            entry.EntryLo0 = (Cop0[EntryLo0Register] & Tlb.EntryLoKept) | global;
            entry.EntryLo1 = (Cop0[EntryLo1Register] & Tlb.EntryLoKept) | global;
        }

        // A failed probe sets the top bit rather than an index, which is how a handler tells them apart.
        private void ProbeTlb()
        {
            int found = Tlb.Probe(Cop0[EntryHiRegister]);
            Cop0[IndexRegister] = found < 0 ? 0x8000_0000UL : (ulong)found;
        }

        // The entry a random write lands on is whatever Random reads, which is what makes Wired work - see §7.4.
        private int RandomIndex() => (int)ReadRandom();

        // The four registers a fault fills in for the handler, all from the one address - see Mars_Cop0.md §7.
        private void RecordFaultingAddress(ulong address)
        {
            Cop0[BadVirtualAddressRegister] = address;

            // An address error writes the page number too, not only a miss - see Mars_Tlb.md §7.6.
            Cop0[EntryHiRegister] = (Cop0[EntryHiRegister] & 0xFF) | (address & EntryHiWritable & ~0xFFUL);

            // A fault owns everything below the software field, so the low four bits are cleared too.
            Cop0[ContextRegister] = (Cop0[ContextRegister] & ContextWritable) | ((address >> 9) & ContextBadVpn2);

            Cop0[XContextRegister] = (Cop0[XContextRegister] & XContextWritable)
                | ((address >> 9) & XContextBadVpn2)
                | ((address >> 31) & XContextRegion);
        }

        private static bool IsAddressRelated(ExceptionCode code) =>
            code is ExceptionCode.AddressErrorLoad or ExceptionCode.AddressErrorStore
                or ExceptionCode.TlbLoad or ExceptionCode.TlbStore or ExceptionCode.TlbModification;
    }
}
