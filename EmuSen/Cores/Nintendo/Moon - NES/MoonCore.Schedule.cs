namespace EmuSen.Cores.Nintendo.Moon
{
    // Moon's own timeline: a master clock and the scanline the CPU is paced against - see Moon_Core.md §2.
    public partial class MoonCore
    {
        // Where the machine has reached on its master clock.
        private long _masterClock;

        // Where the scanline being paced began; the CPU is budgeted up to the next boundary.
        private long _lineStartClock;

        // Master clocks earned but not yet worth a whole CPU cycle, carried so 1364/12 cannot drift - see Moon_Core.md §2.1.
        private long _cpuRemainder;

        // Cycles the CPU has run beyond its budget, carried so the overshoot never accumulates as drift.
        private long _cpuBudget;

        private bool _frameComplete;

        // Reset each EndFrame, not each scanline - see Moon_Debug.md §3.2.
        private long _cpuApuTicksAccum;
        private long _ppuTicksAccum;

        public long MasterClock => _masterClock;

        // The PPU owns its own position now; this mirrors it for the shell - see Moon_PPU.md §1.
        public int CurrentScanline => Ppu?.Scanline ?? 0;

        // How far the CPU may run before the loop checks back; the PPU decides when a frame ends - see Moon_Core.md §2.2.
        private long NextScanlineBoundary => _lineStartClock + MasterClocksPerScanline;

        private void ResetSchedule()
        {
            _masterClock = 0;
            _lineStartClock = 0;
            _cpuRemainder = 0;
            _cpuBudget = 0;
            _frameComplete = false;
        }

        // Whole CPU cycles earned by masterDelta, remainder carried to the next call - see Moon_Core.md §2.1.
        private long EarnCpuCycles(long masterDelta)
        {
            if (masterDelta < 0) throw new ArgumentOutOfRangeException(nameof(masterDelta), "A clock cannot run backwards.");

            long scaled = masterDelta + _cpuRemainder;
            _cpuRemainder = scaled % MasterClocksPerCpuCycle;
            return scaled / MasterClocksPerCpuCycle;
        }

        // Closes the line the CPU was paced against and opens the next one.
        private void EndScanline(long deadline)
        {
            if (deadline > _masterClock) _masterClock = deadline;
            _lineStartClock += MasterClocksPerScanline;
        }

        private void EndFrame()
        {
            TotalFrames++;

            double ticksToMs = 1000.0 / System.Diagnostics.Stopwatch.Frequency;
            LastFrameCpuApuMs = _cpuApuTicksAccum * ticksToMs;
            LastFramePpuMs = _ppuTicksAccum * ticksToMs;
            _cpuApuTicksAccum = 0;
            _ppuTicksAccum = 0;

            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            Cheats.ApplyAll(ReadForCheat, WriteForCheat);

            if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();

            _frameComplete = true;
        }
    }
}
