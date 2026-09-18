namespace EmuSen.Cores.Nintendo.Mars.Vi
{
    // The half line the signal is on, and the interrupt it raises when it reaches the one the game asked for - see Mars_VideoTiming.md §1.
    public sealed partial class Vi
    {
        // The processor's pipeline clock, which is what MemoryBus.Cycles counts - see Mars_Memory.md §3.
        private const long ProcessorClock = 93_750_000;

        // What the interface's own clock runs at, a line being VI_H_SYNC of its cycles - see §1.1.
        private const long NtscClock = 48_681_818;
        private const long PalClock = 49_656_530;

        private long _debt;
        private int _halfLine;
        private bool _field;

        // Called from the bus's one counter, so the signal advances whether or not anything is scanning it - see §1.1.
        public void Step(long cycles)
        {
            int sync = (int)Register(VerticalSync) & 0x3FF;
            long line = (long)(Register(HorizontalSync) & 0xFFF) * ProcessorClock;
            if (sync <= 0 || line <= 0) return;

            long clock = IsPal ? PalClock : NtscClock;
            _debt += cycles * clock * 2;

            while (_debt >= line)
            {
                _debt -= line;
                Advance(sync);
            }
        }

        // One half line: the count wraps at the vertical sync, and only an interlaced signal changes field when it does - see §1.2.
        private void Advance(int sync)
        {
            _halfLine++;

            if (_halfLine >= sync)
            {
                _halfLine = 0;
                if (Serrate) _field = !_field;
            }

            _registers[CurrentLine >> 2] = (uint)((_halfLine & ~1) | (_field ? 1 : 0));

            // The interrupt is tested once a line, which is what makes an odd register unreachable - see §2.
            if ((_halfLine & 1) == 0 && _halfLine == (int)(Register(Interrupt) & 0x3FF))
            {
                _bus.Mi.Raise(Memory.MiInterrupt.VideoInterface);
            }
        }
    }
}
