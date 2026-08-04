using EmuSen.Crystal;

namespace EmuSen.Cores.Nintendo.Moon
{
    // Moon's half of the Crystal contract - see EmuSen_Crystal_Scheduler.md and Moon_Core.md §2.
    public partial class MoonCore : IScheduleHandler
    {
        internal enum MoonEvent
        {
            ScanlineBoundary = 0,
        }

        private readonly Scheduler _schedule = new(8);

        // The CPU's rate against the master clock; 1364 master clocks a line is not a whole number of them.
        private static readonly ClockRatio CpuRatio = new(1, MasterClocksPerCpuCycle);

        private ClockAccumulator _cpuClock;

        // Cycles the CPU has run beyond its budget, carried so the overshoot never accumulates as drift.
        private long _cpuBudget;

        private long _lineStartClock;
        private bool _frameComplete;

        public Scheduler Schedule => _schedule;

        public long MasterClock => _schedule.Now;

        public int CurrentScanline => _currentScanline;

        private void ResetSchedule()
        {
            _schedule.Reset();
            _schedule.SetHandler(this);
            _schedule.At(MasterClocksPerScanline, (int)MoonEvent.ScanlineBoundary);
            _cpuClock.Reset();
            _cpuBudget = 0;
            _lineStartClock = 0;
            _currentScanline = 0;
            _frameComplete = false;
        }

        public void OnScheduledEvent(int eventId, long now)
        {
            switch ((MoonEvent)eventId)
            {
                case MoonEvent.ScanlineBoundary:
                    EndScanline();
                    break;
            }
        }

        private void EndScanline()
        {
            Ppu!.EndScanline(_currentScanline);

            _currentScanline++;
            _lineStartClock += MasterClocksPerScanline;
            _schedule.At(_lineStartClock + MasterClocksPerScanline, (int)MoonEvent.ScanlineBoundary);

            if (_currentScanline >= Video.Ppu.TotalScanlines)
            {
                _currentScanline = 0;
                EndFrame();
            }
        }

        private void EndFrame()
        {
            TotalFrames++;

            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            Cheats.ApplyAll(ReadForCheat, WriteForCheat);

            if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();

            _frameComplete = true;
        }
    }
}
