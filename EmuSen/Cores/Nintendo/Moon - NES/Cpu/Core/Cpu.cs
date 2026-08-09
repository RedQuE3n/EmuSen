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

        // Idle cycles the chip spends before the first instruction of a reset - see Moon_CPU.md §5.1.
        public const int ResetCycles = 8;

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

        // Per-cycle samples; <Earlier> is the one from before the current cycle - see Moon_CPU.md §5.5.
        private bool _nmiSampledLast;
        private bool _nmiSampledEarlier;
        private bool _irqSampledLast;
        private bool _irqSampledEarlier;


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

            // A cold chip is observed at 0; SoftReset's three phantom pushes are what make it 0xFD.
            S = 0;
            P = (byte)(CpuFlags.I | CpuFlags.U);
            Cycles = 0;
            SoftReset();
        }

        // RESET only sets I and spends three phantom pushes; A/X/Y and the other flags survive - see Moon_CPU.md §5.1.
        public void SoftReset()
        {
            S -= 3;
            P |= (byte)(CpuFlags.I | CpuFlags.U);
            Jammed = false;
            _nmiLine = false;
            _nmiPending = false;
            _irqLine = false;
            _serviceNmi = false;
            _serviceIrq = false;
            _nmiSampledLast = false;
            _nmiSampledEarlier = false;
            _irqSampledLast = false;
            _irqSampledEarlier = false;
            _hasDelayedI = false;
            _delayedI = false;
            _instructionCycles = 0;
            ResetSequence();
        }

        // The vector read is not clocked; the eight cycles that follow it are - see Moon_CPU.md §5.1.
        private void ResetSequence()
        {
            PC = (ushort)(_bus.Read(ResetVector) | (_bus.Read((ushort)(ResetVector + 1)) << 8));
            LastInstructionPC = PC;

            for (int i = 0; i < ResetCycles; i++)
            {
                Cycles++;
                _bus.Tick();
                SampleInterrupts();
            }
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
                ServiceInterrupt(NmiVector);
            }
            else if (_serviceIrq)
            {
                ServiceInterrupt(IrqVector);
            }
            else
            {
                Dispatch(Read(PC++));
            }

            PollInterrupts();
            return _instructionCycles;
        }

        // The last cycle of an instruction is too late to be recognised, so the earlier sample wins - see Moon_CPU.md §5.5.
        private void PollInterrupts()
        {
            _serviceNmi = _nmiSampledEarlier;
            _serviceIrq = !_serviceNmi && _irqSampledEarlier;

            if (_hasDelayedI)
            {
                SetFlag(CpuFlags.I, _delayedI);
                _hasDelayedI = false;
            }
        }

        // Seven cycles: the chip fetches an opcode and its operand, discards both, then vectors - see Moon_CPU.md §5.1.
        private void ServiceInterrupt(ushort vector)
        {
            Read(PC);
            Read(PC);
            Push((byte)(PC >> 8));
            Push((byte)PC);
            Push((byte)((P & ~(byte)CpuFlags.B) | (byte)CpuFlags.U));
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
            BeginCycle();
            byte value = _bus.Read(address);
            SampleInterrupts();
            return value;
        }

        private void Write(ushort address, byte data)
        {
            BeginCycle();
            _bus.Write(address, data);
            SampleInterrupts();
        }

        // The cycle's clock edge runs before the access, so a read sees this cycle - see Moon_CPU.md §5.5.
        private void BeginCycle()
        {
            _instructionCycles++;
            Cycles++;
            _bus.Tick();
        }

        // The decision uses the sample from before the final cycle, so this keeps one cycle of history - see Moon_CPU.md §5.5.
        private void SampleInterrupts()
        {
            _nmiSampledEarlier = _nmiSampledLast;
            _irqSampledEarlier = _irqSampledLast;

            _nmiSampledLast = _nmiPending;
            _irqSampledLast = _irqLine && !GetFlag(CpuFlags.I);
        }

        // A taken branch drops an IRQ that arrived only this cycle - see Moon_CPU.md §5.5.
        internal void SuppressJustArrivedIrq()
        {
            if (_irqSampledLast && !_irqSampledEarlier) _irqSampledLast = false;
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
