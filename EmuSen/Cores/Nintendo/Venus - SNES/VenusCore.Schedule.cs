using System.Diagnostics;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Debug;

namespace EmuSen.Cores.Nintendo.Venus
{
    // Venus's own timeline: the master clock and what closes a scanline on it - see Venus_CPU.md §8.5a.
    public partial class VenusCore
    {
        // Where the machine has reached on its master clock; the CPU overshoots a line and carries it.
        private long _masterClock;

        // Where the current scanline began, so LineCycles is a subtraction rather than a running carry.
        private long _lineStartClock;

        // A frame ends inside EndScanline, but RunFrame has to be the thing that returns.
        private bool _frameComplete;

        public long MasterClock => _masterClock;

        // Where the CPU loop stops and hands the line to EndScanline.
        private long NextScanlineBoundary => _lineStartClock + CyclesPerScanline;

        private void ResetSchedule()
        {
            _masterClock = 0;
            _lineStartClock = 0;
            _frameComplete = false;
        }

        // Everything that used to sit after RunFrame's per-scanline inner loop, in the same order.
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

            // How much a deferred renderer would have to snapshot for this game - see Venus_PPU.md §7.3.
            if (DebugSettings.PpuActiveDisplayWriteLogging)
            {
                Console.WriteLine($"[ACTIVEWRITES] frame {TotalFrames}: {Bus.Ppu.ActiveDisplayWrites} write(s) on {Bus.Ppu.ActiveDisplayLines} of 224 line(s)");
            }
            Bus.Ppu.ResetActiveDisplayCounters();

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
