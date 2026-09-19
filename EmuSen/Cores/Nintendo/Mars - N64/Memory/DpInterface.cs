namespace EmuSen.Cores.Nintendo.Mars.Memory
{
    // The display processor's command registers, the stream they hand over, and the processor it feeds - see Mars_Rdp.md §2.
    public sealed class DpInterface
    {
        public const uint Start = 0x00;
        public const uint End = 0x04;
        public const uint Current = 0x08;
        public const uint Status = 0x0C;

        public const uint StatusXbus = 0x001;
        public const uint StatusFreeze = 0x002;
        public const uint StatusStartGclk = 0x008;
        public const uint StatusPipeBusy = 0x020;
        public const uint StatusBufferReady = 0x080;
        public const uint StatusStartValid = 0x400;

        // Twenty-four bits, word-aligned, for all three addresses - see §2.1.
        public const uint AddressMask = 0x00FF_FFF8;

        public readonly Rdp.Rdp Processor;

        [EmuSen.Common.SkipInState] private readonly MemoryBus _bus;

        private uint _start;
        private uint _end;
        private uint _current;
        private bool _startValid;
        private bool _xbus;
        private bool _freeze;
        private bool _running;

        public DpInterface(MemoryBus bus)
        {
            _bus = bus;
            Processor = new Rdp.Rdp(bus);
        }

        // The buffer is never full, because every word handed over has already been taken - see §2.2.
        public uint StatusWord =>
            (_xbus ? StatusXbus : 0)
            | (_freeze ? StatusFreeze : 0)
            | (_running ? StatusStartGclk | StatusPipeBusy : 0)
            | StatusBufferReady
            | (_startValid ? StatusStartValid : 0);

        public uint Read32(uint offset)
        {
            return (offset & 0x1C) switch
            {
                Start => _start,
                End => _end,
                Current => _current,
                Status => StatusWord,
                _ => 0,
            };
        }

        public void Write32(uint offset, uint value)
        {
            switch (offset & 0x1C)
            {
                case Start:
                    WriteStart(value);
                    break;

                case End:
                    WriteEnd(value);
                    break;

                case Status:
                    WriteStatus(value);
                    break;
            }
        }

        // A second start before an end is refused, so a list cannot be moved under the one being handed over - see §2.1.
        private void WriteStart(uint value)
        {
            if (_startValid) return;

            _start = value & AddressMask;
            _startValid = true;
        }

        private void WriteEnd(uint value)
        {
            _end = value & AddressMask;

            if (_startValid)
            {
                _current = _start;
                _startValid = false;
            }

            if (!_freeze) _running = true;

            Take();
        }

        // Only the four bits the corpus measures; a pair with both asserted is no request, by analogy with the RSP - see §2.4.
        private void WriteStatus(uint value)
        {
            if ((value & 3) == 1) _xbus = false;
            if ((value & 3) == 2) _xbus = true;

            if ((value & 0xC) == 4)
            {
                _freeze = false;
                Take();
            }

            if ((value & 0xC) == 8) _freeze = true;
        }

        // Everything up to end is taken at once; an address compare, so a list may run past data memory's end - see §2.3.
        private void Take()
        {
            _bus.Written++;

            while (!_freeze && _current < _end)
            {
                ulong word = _xbus ? ReadDataMemory(_current) : _bus.Read64(_current);
                _current += 8;

                if (Processor.Accept(word)) FullSync();
            }
        }

        private void FullSync()
        {
            _running = false;
            _bus.Mi.Raise(MiInterrupt.DisplayProcessor);
        }

        private ulong ReadDataMemory(uint address)
        {
            ulong word = 0;
            for (uint i = 0; i < 8; i++) word = (word << 8) | _bus.SpDmem[(address + i) & (MemoryMap.SpMemSize - 1)];

            return word;
        }
    }
}
