using System;

namespace EmuSen.Cores.Nintendo.Mars.Cpu.Core
{
    // The Cause register's exception codes, named before the register exists - see Mars_Cpu.md §4.
    public enum ExceptionCode
    {
        Interrupt = 0,
        TlbModification = 1,
        TlbLoad = 2,
        TlbStore = 3,
        AddressErrorLoad = 4,
        AddressErrorStore = 5,
        BusErrorInstruction = 6,
        BusErrorData = 7,
        Syscall = 8,
        Breakpoint = 9,
        ReservedInstruction = 10,
        CoprocessorUnusable = 11,
        Overflow = 12,
        Trap = 13,
        FloatingPoint = 15,
    }

    // Thrown where the fault is found and caught at the step boundary - see Mars_Cpu.md §4.
    public sealed class CpuException : Exception
    {
        public ExceptionCode Code;
        public ulong Address;
        public bool InDelaySlot;

        // A miss gets its own vector; an entry that exists and is unusable does not - see Mars_Tlb.md §3.
        public bool Refill;

        // Which coprocessor the fault names, written to Cause on every exception - see Mars_Fpu.md §3.1.
        public int Coprocessor;

        // One instance per CPU, rethrown: the unwind is the cost, an allocation per fault need not be.
        public CpuException() : base("VR4300 exception") { }
    }
}
