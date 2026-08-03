using System.Diagnostics;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Crystal;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus
{
    // Venus's half of the Crystal contract: the master timeline and its events - see EmuSen_Crystal_Scheduler.md §7.
    public partial class VenusCore : IScheduleHandler
    {
        // Every event still fires at a scanline boundary, which is why there is only one - see §7's note.
        internal enum VenusEvent
        {
            ScanlineBoundary = 0,
        }

        private readonly Scheduler _schedule = new(8);

        // Where the current scanline began, so LineCycles is a subtraction rather than a running carry.
        private long _lineStartClock;

        // A frame ends inside an event, but RunFrame has to be the thing that returns.
        private bool _frameComplete;

        public Scheduler Schedule => _schedule;

        public long MasterClock => _schedule.Now;

        private void ResetSchedule()
        {
            _schedule.Reset();
            _schedule.SetHandler(this);
            _schedule.At(CyclesPerScanline, (int)VenusEvent.ScanlineBoundary);
            _lineStartClock = 0;
            _frameComplete = false;
        }

        // Everything that used to sit after RunFrame's per-scanline inner loop, in the same order.
        public void OnScheduledEvent(int eventId, long now)
        {
            switch ((VenusEvent)eventId)
            {
                case VenusEvent.ScanlineBoundary:
                    EndScanline();
                    break;
            }
        }

        private void EndScanline()
        {
            Bus!.CurrentScanline = _currentScanline;

            if (_currentScanline < 225)
            {
                // HDMA runs in H-blank, before the scanline it feeds is drawn - see Venus_PPU.md §4.1.
                Bus.Dma.ExecuteHdma();
                long afterHdma = Stopwatch.GetTimestamp();
                _hdmaTicksAccum += afterHdma - _phaseEndCpu;

                // HDMA above still runs; only the pixel pass drops - see EmuSen_Rewind_And_FastForward.md §2.2.
                if (_currentScanline < 224 && !SkipRendering)
                {
                    Renderer!.RenderScanline(Bus, _currentScanline);
                }
                _ppuTicksAccum += Stopwatch.GetTimestamp() - afterHdma;
            }

            if (_currentScanline == InterruptController.AutoJoypadScanline)
            {
                Bus.Interrupts.InVBlank = true;
                Bus.Interrupts.RaiseVBlank();
                Bus.Ppu.ReloadOamAddressForVBlank();
                if (Bus.Interrupts.NmiEnabled) Cpu!.Nmi();
            }

            // The auto-joypad read completes partway into vblank, not at its start - see Venus_Memory.md §4.4.
            if (_currentScanline == InterruptController.AutoJoypadLatchScanline && Bus.Interrupts.AutoJoypadEnabled)
            {
                Bus.Input.LatchAutoJoypad();
            }

            _scanlineStarted = false;
            _currentScanline++;
            _lineStartClock += CyclesPerScanline;
            _schedule.At(_lineStartClock + CyclesPerScanline, (int)VenusEvent.ScanlineBoundary);

            if (_currentScanline >= _totalScanlines)
            {
                _currentScanline = 0;
                EndFrame();
            }
        }

        private void EndFrame()
        {
            TotalFrames++;
            Bus!.FrameCount = TotalFrames;
            Bus.FrameObserver?.OnFrame(TotalFrames);

            // A shared reference point between the CPU and SPC700 trace streams, which have none otherwise.
            if (DebugSettings.MasterLoggingEnabled &&
                (DebugSettings.CpuVerboseLogging || DebugSettings.Spc700VerboseLogging))
            {
                Console.WriteLine($"[FRAME] {TotalFrames}");
            }

            // Periodic autosave - see Cartridge.SaveSram's own comment for why this is safe this often.
            if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();

            double ticksToMs = 1000.0 / Stopwatch.Frequency;
            LastFrameCpuSpc700Ms = _cpuSpc700TicksAccum * ticksToMs;
            LastFramePpuMs = _ppuTicksAccum * ticksToMs;
            LastFrameHdmaMs = _hdmaTicksAccum * ticksToMs;
            _cpuSpc700TicksAccum = 0;
            _ppuTicksAccum = 0;
            _hdmaTicksAccum = 0;

            // The clock runs on across frames; a long holds ~13 million years of it at 21MHz.
            _frameComplete = true;
        }
    }
}
