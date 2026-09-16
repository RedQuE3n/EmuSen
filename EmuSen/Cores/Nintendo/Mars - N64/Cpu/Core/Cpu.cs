using System;
using EmuSen.Cores.Nintendo.Mars.Memory;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The VR4300's integer core: 64-bit throughout, two program counters - see Mars_Cpu.md §1.
    public sealed partial class Cpu
    {
        public readonly ulong[] Gpr = new ulong[32];

        public ulong Hi;
        public ulong Lo;

        public ulong Pc;
        public ulong NextPc;

        // The address of the instruction being executed; Pc has already moved past it - see Mars_Cpu.md §3.
        public ulong CurrentPc;

        // Set while the instruction being executed follows a taken branch - see Mars_Cpu.md §3.
        public bool InDelaySlot;

        // Where a raised exception is recorded until COP0 exists to vector it - see Mars_Cpu.md §4.1.
        public CpuException? LastException;

        public long Instructions;

        private readonly MarsBus _bus;
        private readonly CpuException _exception = new();

        private bool _branchPending;
        private uint _lastCount;

        public Cpu(MarsBus bus)
        {
            _bus = bus;
            Pc = 0xFFFF_FFFF_A400_0040;
            NextPc = Pc + 4;
        }

        public MarsBus Bus => _bus;

        public void Step()
        {
            try
            {
                CurrentPc = Pc;

                // Checked before the fetch, so the saved address is the instruction not yet run - see §10.
                InDelaySlot = _branchPending;
                CheckInterrupts();

                uint instruction = FetchInstruction();
                _branchPending = false;

                Pc = NextPc;
                NextPc = Pc + 4;

                Execute(instruction);

                // One cycle, plus whatever the vendor manual tabulates for this instruction - see Mars_Cpu.md §5.
                _bus.Tick(1 + _extraCycles);
                _extraCycles = 0;
                Instructions++;

                UpdateTimer();
            }
            catch (CpuException raised)
            {
                LastException = raised;
                EnterException(raised);
                _bus.Tick(1);
            }
        }

        public void Run(int steps)
        {
            for (int i = 0; i < steps; i++) Step();
        }

        private uint FetchInstruction()
        {
            if ((Pc & 3) != 0) throw Raise(ExceptionCode.AddressErrorLoad, Pc);

            return ReadWord(Pc);
        }

        // Written once so the TLB and the caches cannot be bypassed later - see Mars_Cpu.md §6.
        private uint Translate(ulong address)
        {
            if (MemoryMap.TryTranslateDirect(address, out uint physical)) return physical;

            throw Raise(ExceptionCode.TlbLoad, address);
        }

        private uint ReadWord(ulong address) => _bus.Read32(Translate(address));

        private CpuException Raise(ExceptionCode code, ulong address)
        {
            _exception.Code = code;
            _exception.Address = address;
            _exception.InDelaySlot = InDelaySlot;
            return _exception;
        }

        // Register zero is wired to zero, so a write to it is dropped rather than stored.
        private void Write(int register, ulong value)
        {
            if (register != 0) Gpr[register] = value;
        }

        // A 32-bit result occupies a 64-bit register sign-extended, which is the whole of MIPS III - see §2.
        private void Write32(int register, uint value) => Write(register, (ulong)(long)(int)value);

        private ulong Read(int register) => Gpr[register];

        private void Branch(ulong target)
        {
            NextPc = target;
            _branchPending = true;
        }

        // A likely branch that is not taken throws its delay slot away rather than executing it - see §3.1.
        private void NullifyDelaySlot()
        {
            Pc = NextPc;
            NextPc = Pc + 4;
        }
    }
}
