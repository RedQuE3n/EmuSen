using System;
using EmuSen.Cores.Nintendo.Mars.Rsp;

namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The signal processor's registers and its DMA, and the processor they drive - see Mars_Rsp.md §5.
    public sealed class SpInterface
    {
        public const uint MemAddress = 0x00;
        public const uint DramAddress = 0x04;
        public const uint ReadLength = 0x08;
        public const uint WriteLength = 0x0C;
        public const uint Status = 0x10;
        public const uint DmaFull = 0x14;
        public const uint DmaBusy = 0x18;
        public const uint Semaphore = 0x1C;

        public const uint StatusHalt = 0x01;
        public const uint StatusBroke = 0x02;
        public const uint StatusSingleStep = 0x20;
        public const uint StatusInterruptOnBreak = 0x40;

        // Eight of them, each with its own clear and set bit in a write - see Mars_Rsp.md §5.1.
        public const int SignalShift = 7;

        public readonly Rsp.Rsp Processor;

        // Bit 12 of the memory address chooses which of the two banks a transfer touches.
        private const uint ImemSelect = 0x1000;

        private readonly MemoryBus _bus;

        private uint _memAddress;
        private uint _dramAddress;
        private bool _semaphore;
        private bool _singleStep;
        private bool _interruptOnBreak;
        private uint _signals;

        public SpInterface(MemoryBus bus)
        {
            _bus = bus;
            Processor = new Rsp.Rsp(bus);
        }

        // Assembled rather than stored: the processor owns the two bits everything else watches - see Mars_Rsp.md §5.
        public uint StatusWord =>
            (Processor.Halted ? StatusHalt : 0)
            | (Processor.Broke ? StatusBroke : 0)
            | (_singleStep ? StatusSingleStep : 0)
            | (_interruptOnBreak ? StatusInterruptOnBreak : 0)
            | (_signals << SignalShift);

        public uint Read32(uint offset)
        {
            switch (offset & 0x1C)
            {
                case MemAddress: return _memAddress;
                case DramAddress: return _dramAddress;

                // Both lengths read back as the last value written, which is how hardware reports them.
                case ReadLength:
                case WriteLength: return 0xFF8;

                case Status: return StatusWord;

                // Instantaneous transfers are never queued and never in progress - see Mars_Memory.md §6.1.
                case DmaFull:
                case DmaBusy: return 0;

                case Semaphore: return TakeSemaphore();
                default: return 0;
            }
        }

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x1C)
            {
                case MemAddress:
                    _memAddress = value & 0x1FF8;
                    break;

                case DramAddress:
                    _dramAddress = value & 0x00FF_FFF8;
                    break;

                case ReadLength:
                    Transfer(value, toSignalProcessor: true);
                    break;

                case WriteLength:
                    Transfer(value, toSignalProcessor: false);
                    break;

                case Status:
                    WriteStatus(value);
                    break;

                case Semaphore:
                    _semaphore = false;
                    break;
            }
        }

        // A read takes the semaphore and reports whether it was already held.
        private uint TakeSemaphore()
        {
            uint held = _semaphore ? 1u : 0u;
            _semaphore = true;
            return held;
        }

        // Every field is a clear bit and a set bit, and a write naming both leaves it alone - see Mars_Rsp.md §5.1.
        private void WriteStatus(uint value)
        {
            if (Asks(value, 0, set: false)) Processor.Start(Pc);
            if (Asks(value, 0, set: true)) Processor.Halted = true;
            if ((value & 0x004) != 0) Processor.Broke = false;

            if (Asks(value, 3, set: false)) _bus.Mi.Clear(MiInterrupt.SignalProcessor);
            if (Asks(value, 3, set: true)) _bus.Mi.Raise(MiInterrupt.SignalProcessor);

            if (Asks(value, 5, set: false)) _singleStep = false;
            if (Asks(value, 5, set: true)) _singleStep = true;
            if (Asks(value, 7, set: false)) _interruptOnBreak = false;
            if (Asks(value, 7, set: true)) _interruptOnBreak = true;

            for (int signal = 0; signal < 8; signal++)
            {
                if (Asks(value, 9 + signal * 2, set: false)) _signals &= ~(1u << signal);
                if (Asks(value, 9 + signal * 2, set: true)) _signals |= 1u << signal;
            }
        }

        // A pair of bits with both asserted is not a set and then a clear; it is no request at all.
        private static bool Asks(uint value, int clearBit, bool set)
        {
            uint pair = (value >> clearBit) & 3;
            return pair == (set ? 2u : 1u);
        }

        // The program counter is a register of its own, a page away from the rest - see Mars_Rsp.md §2.
        public uint Pc
        {
            get => Processor.Pc;
            set
            {
                Processor.Pc = value & Rsp.Rsp.PcMask;
                Processor.NextPc = (Processor.Pc + 4) & Rsp.Rsp.PcMask;
            }
        }

        // One instruction per tick, a placeholder for a clock ratio Phase G owns - see Mars_Rsp.md §7.
        public void Step(long cycles)
        {
            for (long i = 0; i < cycles && !Processor.Halted; i++)
            {
                bool broke = Processor.Broke;

                Processor.Step();

                if (Processor.Broke && !broke && _interruptOnBreak)
                {
                    _bus.Mi.Raise(MiInterrupt.SignalProcessor);
                }

                if (_singleStep) Processor.Halted = true;
            }
        }

        // Length is encoded one short, and the row count and skip make it rectangular - see Mars_Memory.md §6.
        private void Transfer(uint encoded, bool toSignalProcessor)
        {
            uint length = ((encoded & 0xFFF) | 7) + 1;
            uint rows = ((encoded >> 12) & 0xFF) + 1;
            uint skip = (encoded >> 20) & 0xFFF;

            byte[] bank = (_memAddress & ImemSelect) != 0 ? _bus.SpImem : _bus.SpDmem;
            uint bankOffset = _memAddress & 0xFF8;

            for (uint row = 0; row < rows; row++)
            {
                for (uint i = 0; i < length; i++)
                {
                    // Wrapping inside the bank rather than running on into the next one - see Mars_Memory.md §6.2.
                    uint spOffset = (bankOffset + i) % MemoryMap.SpMemSize;
                    uint dramAddress = _dramAddress + i;

                    if (toSignalProcessor) bank[spOffset] = _bus.Read8(dramAddress);
                    else _bus.Write8(dramAddress, bank[spOffset]);
                }

                bankOffset = (bankOffset + length) % MemoryMap.SpMemSize;
                _dramAddress += length + skip;
            }

            _memAddress = (_memAddress & ImemSelect) | bankOffset;
        }
    }
}
