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

        // The console's video clock, which the audio interface's DAC divides as well - see Mars_Audio.md §2.
        public long VideoClock => IsPal ? PalClock : NtscClock;

        private long _debt;
        private int _halfLine;
        private bool _field;

        // The cycle _debt was last brought up to; derived, so a loaded state rebases it - see Mars_Performance.md §9.
        [EmuSen.Common.SkipInState] private long _debtAt;

        // The first cycle at which the next half line is owed, or never while the signal is unprogrammed - see Mars_Performance.md §9.
        [EmuSen.Common.SkipInState] public long Due;

        // Fields completed since power-on, counted where the half line wraps, so a frame can end on one - see Mars_Core.md §3.
        public long Fields { get; private set; }

        // A line is VI_H_SYNC of the interface's clocks and the field VI_V_SYNC half lines; zero in either stops the signal - see §1.1.
        private bool Programmed(out int sync, out long line)
        {
            sync = (int)Register(VerticalSync) & 0x3FF;
            line = (long)(Register(HorizontalSync) & 0xFFF) * ProcessorClock;
            return sync > 0 && line > 0;
        }

        // What every tick since _debtAt added, at the rate the registers set before a write changes them - see Mars_Performance.md §9.
        public void Settle()
        {
            long now = _bus.Cycles;
            if (Programmed(out _, out _)) _debt += (now - _debtAt) * VideoClock * 2;
            _debtAt = now;
        }

        // Every half line owed by now, in one go, as a tick that crossed several always advanced them - see §1.1.
        public void Catch()
        {
            if (_bus.Cycles < Due) return;

            Settle();
            if (Programmed(out int sync, out long line))
            {
                while (_debt >= line)
                {
                    _debt -= line;
                    Advance(sync);
                }
            }

            Schedule();
        }

        // Only after Settle or Catch, since it counts from _debtAt - see Mars_Performance.md §9.
        public void Schedule()
        {
            if (!Programmed(out _, out long line))
            {
                Due = long.MaxValue;
                return;
            }

            long step = VideoClock * 2;
            long remaining = line - _debt;
            Due = remaining <= 0 ? _debtAt : _debtAt + (remaining + step - 1) / step;
        }

        // A loaded state was settled when it was saved, so its debt is owed from the cycle it was saved at - see Mars_SaveStates.md §2.
        public void Rebase() => _debtAt = _bus.Cycles;

        // One half line: the count wraps at the vertical sync, and only an interlaced signal changes field when it does - see §1.2.
        private void Advance(int sync)
        {
            _halfLine++;

            if (_halfLine >= sync)
            {
                _halfLine = 0;
                if (Serrate) _field = !_field;
                Fields++;
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
