using System;

namespace EmuSen.Cores.Nintendo.Moon.Processor
{
    [Flags]
    public enum CpuFlags : byte
    {
        None = 0,
        C = 1 << 0,
        Z = 1 << 1,
        I = 1 << 2,
        D = 1 << 3,
        B = 1 << 4,
        U = 1 << 5,
        V = 1 << 6,
        N = 1 << 7,
    }

    // The 2A03's 6502 core - binary-only ADC/SBC, one bus access per cycle. See Moon_CPU.md.
    public partial class Cpu
    {
        public const ushort NmiVector = 0xFFFA;
        public const ushort ResetVector = 0xFFFC;
        public const ushort IrqVector = 0xFFFE;

        // Reset leaves the pointer here rather than at 0xFF - see Moon_CPU.md §5.1.
        public const byte ResetStackPointer = 0xFD;

        // ANE/LXA mix in a constant the real chip's analog behavior produces - see Moon_CPU.md §6.3.
        public const byte UnstableMagic = 0xEE;

        private readonly ICpuBus _bus;

        public byte A;
        public byte X;
        public byte Y;
        public byte S;
        public ushort PC;

        // Bits 4 and 5 are not real flip-flops; held U set and B clear - see Moon_CPU.md §4.1.
        public byte P;

        public long Cycles;

        // A KIL/JAM opcode wedges the real chip until reset - see Moon_CPU.md §6.4.
        public bool Jammed;

        // Snapshotted at instruction start, unlike the live PC above - see Moon_CPU.md §2.2.
        public ushort LastInstructionPC;

        private int _instructionCycles;

        private bool _nmiLine;
        private bool _nmiPending;
        private bool _irqLine;

        private bool _serviceNmi;
        private bool _serviceIrq;

        // CLI/SEI/PLP land their I write after the interrupt poll - see Moon_CPU.md §5.3.
        private bool _hasDelayedI;
        private bool _delayedI;

        public Cpu(ICpuBus bus)
        {
            _bus = bus ?? throw new ArgumentNullException(nameof(bus));
        }

        public bool GetFlag(CpuFlags flag) => (P & (byte)flag) != 0;

        public void SetFlag(CpuFlags flag, bool value)
        {
            if (value) P |= (byte)flag;
            else P &= (byte)~(byte)flag;
        }

        // Pulls the reset vector over the bus, so the caller pays the real chip's startup reads.
        public void Reset()
        {
            A = 0;
            X = 0;
            Y = 0;
            S = ResetStackPointer;
            P = (byte)(CpuFlags.I | CpuFlags.U);
            Jammed = false;
            _nmiLine = false;
            _nmiPending = false;
            _irqLine = false;
            _serviceNmi = false;
            _serviceIrq = false;
            _hasDelayedI = false;
            _delayedI = false;
            _instructionCycles = 0;
            Cycles = 0;
            PC = ReadVector(ResetVector);
            LastInstructionPC = PC;
        }

        // Edge-triggered: the latch arms on a low-to-high transition and stays armed until serviced.
        public void SetNmiLine(bool level)
        {
            if (level && !_nmiLine) _nmiPending = true;
            _nmiLine = level;
        }

        // Level-sensitive: whatever holds it asserted must keep holding it - see Moon_CPU.md §5.2.
        public void SetIrqLine(bool level) => _irqLine = level;

        // Runs one instruction, or services one interrupt, and returns the cycles it cost.
        public int Step()
        {
            _instructionCycles = 0;
            LastInstructionPC = PC;

            if (Jammed)
            {
                Read(PC);
                return _instructionCycles;
            }

            if (_serviceNmi)
            {
                _nmiPending = false;
                ServiceInterrupt(NmiVector, false);
            }
            else if (_serviceIrq)
            {
                ServiceInterrupt(IrqVector, false);
            }
            else
            {
                Dispatch(Read(PC++));
            }

            PollInterrupts();
            return _instructionCycles;
        }

        // Sampled at the instruction boundary, before any delayed I write lands - see Moon_CPU.md §5.3.
        private void PollInterrupts()
        {
            _serviceNmi = _nmiPending;
            _serviceIrq = !_serviceNmi && _irqLine && !GetFlag(CpuFlags.I);

            if (_hasDelayedI)
            {
                SetFlag(CpuFlags.I, _delayedI);
                _hasDelayedI = false;
            }
        }

        // <fromBrk> is the only case that pushes the B bit set - see Moon_CPU.md §4.1.
        private void ServiceInterrupt(ushort vector, bool fromBrk)
        {
            Read(PC);
            Push((byte)(PC >> 8));
            Push((byte)PC);
            Push((byte)(fromBrk ? P | 0x30 : (P & ~(byte)CpuFlags.B) | (byte)CpuFlags.U));
            SetFlag(CpuFlags.I, true);
            PC = ReadVector(vector);
        }

        private ushort ReadVector(ushort vector)
        {
            byte lo = Read(vector);
            byte hi = Read((ushort)(vector + 1));
            return (ushort)(lo | (hi << 8));
        }

        private byte Read(ushort address)
        {
            _instructionCycles++;
            Cycles++;
            return _bus.Read(address);
        }

        private void Write(ushort address, byte data)
        {
            _instructionCycles++;
            Cycles++;
            _bus.Write(address, data);
        }

        private void Push(byte value) => Write((ushort)(0x0100 | S--), value);

        private byte Pull() => Read((ushort)(0x0100 | ++S));

        // The chip reads the current top of stack before the pointer moves - see Moon_CPU.md §3.4.
        private byte PullWithDummy()
        {
            Read((ushort)(0x0100 | S));
            return Pull();
        }

        private byte SetZeroNegative(byte value)
        {
            SetFlag(CpuFlags.Z, value == 0);
            SetFlag(CpuFlags.N, (value & 0x80) != 0);
            return value;
        }
    }
}
