using System;
using System.Diagnostics;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus.Memory;
using EmuSen.Cores.Nintendo.Venus.Apu;
using EmuSen.Cores.Nintendo.Venus.Processor;
using EmuSen.Cores.Nintendo.Venus.Video;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.Cores.Nintendo.Venus
{
    // The SNES's ICore implementation - owns and drives Cpu/MemoryBus/
    // Spc700/Renderer frame-by-frame. This is the ONE place the per-
    // scanline timing loop lives now; it used to exist as two separately-
    // maintained copies (the console frontend's Main loop, now
    // EmuSen.Hotaru/Program.cs, and Common/EmulatorSession.cs's
    // RunFrame()) that had to be kept in sync
    // by hand - EmulatorSession's own header comment even said so
    // explicitly. Both now construct a VenusCore and call RunFrame() on
    // it instead.
    //
    // Exposes Cart/Spc700/Bus/Cpu/Renderer as public properties beyond
    // what ICore requires. This is deliberate, not a leaky abstraction:
    // Program.cs's debug toolchain (SnesDebugTarget, DianaOSInterpreter,
    // the F1-F9 hotkeys, the F4 prompt) all need real SNES-specific access
    // a core-agnostic interface has no business providing - see
    // ICore.cs's own comment for why input and debug-toolchain wiring
    // deliberately stay on the concrete type instead of the interface.
    public class VenusCore : ICore
    {
        // Real master clocks per scanline (341 dots x 4 master-clocks/dot),
        // not an abstract "CPU cycle" count - Cpu.Step() now returns actual
        // elapsed master clocks directly (see its own comment), computed
        // from real per-access region speeds instead of a flat assumed
        // rate. This used to be 227, a CPU-cycle-unit figure back-derived
        // assuming every access ran at FastROM's 6-master-clock rate
        // uniformly (227 = 1364/6) - wrong for the common SlowROM case
        // (real hardware only fits ~1364/8 ≈ 170 CPU cycles per scanline
        // there), and the source of this project's ~8% SPC700 audio-pacing
        // undershoot (see Venus_APU.md).
        public const int CyclesPerScanline = 1364;

        // The once-per-scanline DRAM refresh stall - see Venus_CPU.md §8.7.
        public const int DramRefreshPosition = 538;
        public const int DramRefreshClocks = 40;

        // Both differ per region; the dot clock and 224-line active display do not - see Venus_CPU.md §8.5c.
        private const int NtscScanlines = 262;
        private const int PalScanlines = 312;
        private const int NtscMasterClockHz = 21477272;
        private const int PalMasterClockHz = 21281370;

        // The SNES's two independent crystals - see Venus_CPU.md §8.5b.
        private const int ApuClockHz = 1024000;
        private const int SaveEveryNFrames = 300; // ~5 seconds at 60fps

        // "SNES" little-endian, then the format version - see EmuSen_Save_States.md §3.
        private const uint StateMagic = 0x53454E53;
        // v2 appends the SA-1's state after the four original blobs, v3 the
        // NEC DSP's, each only for a cartridge carrying that chip - see
        // EmuSen_Save_States.md §3.
        private const int StateVersion = 3;

        private readonly bool _headless;
        private int _currentScanline;

        // Set from the cartridge header at LoadRom; NTSC until one is loaded.
        private int _totalScanlines = NtscScanlines;
        private int _masterClockHz = NtscMasterClockHz;

        public ConsoleRegion Region { get; private set; } = ConsoleRegion.Ntsc;

        public Cartridge? Cart { get; private set; }
        public Spc700? Spc700 { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Cpu? Cpu { get; private set; }
        public Renderer? Renderer { get; private set; }

        public string CoreName => "SNES";

        // Per-phase breakdown of the last completed RunFrame() call, timed
        // at per-scanline granularity - see Venus_PPU.md §13.
        public double LastFrameCpuSpc700Ms { get; private set; }
        public double LastFramePpuMs { get; private set; }
        public double LastFrameHdmaMs { get; private set; }

        // Sub-breakdown of LastFramePpuMs, straight from Renderer - see Venus_PPU.md §13.
        public double LastFrameObjEvalMs => Renderer?.LastFrameObjEvalMs ?? 0;
        public double LastFrameBlendMs => Renderer?.LastFrameBlendMs ?? 0;

        // Splits the per-scanline BG/OBJ composite into main vs sub screen - see Venus_PPU.md §13.
        public double LastFrameMainCompositeMs => Renderer?.LastFrameMainCompositeMs ?? 0;
        public double LastFrameSubCompositeMs => Renderer?.LastFrameSubCompositeMs ?? 0;

        // Not a fixed constant - pseudo-hi-res (SETINI bit 3) can make a
        // frame 512 pixels wide instead of 256. See Renderer.cs's
        // FrameWidth field for the full citation. Defaults to 256 before
        // a ROM is loaded (no renderer exists yet).
        public int ScreenWidth => Renderer?.FrameWidth ?? 256;
        public int ScreenHeight => 224;

        // NTSC 21477272/(262*1364) ~= 60.098, PAL 21281370/(312*1364) ~= 50.007 - see Venus_CPU.md §8.5b.
        public double FrameRateHz => _masterClockHz / (_totalScanlines * (double)CyclesPerScanline);

        // The master clocks one frame is worth - what a coprocessor's own
        // per-frame clock total has to match - see Venus_SA1.md §2.3.
        public long MasterClocksPerFrame => (long)_totalScanlines * CyclesPerScanline;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        // Drops the per-scanline pixel pass only - see EmuSen_Rewind_And_FastForward.md §2.2.
        public bool SkipRendering { get; set; }

        // True from the instant RunFrame() halts on a breakpoint (or an
        // armed single-step - see BreakpointRegistry) until the next
        // RunFrame() call resumes past it. Kept on the concrete type
        // rather than ICore, same call EmulatorSession's own comment
        // already makes for the LastFrame*Ms profiling properties: nothing
        // consumes this except the console frontend's own debug prompt
        // (EmuSen.Hotaru/Program.cs), which already holds a
        // concrete VenusCore, not just an ICore.
        public bool IsHaltedAtBreakpoint { get; private set; }

        // The 24-bit CPU address RunFrame() halted in front of - only
        // meaningful while IsHaltedAtBreakpoint is true.
        public int HaltedAddress { get; private set; }

        // Mid-scanline resume state - see RunFrame()'s own comment on why
        // these are fields instead of locals. Reset once a scanline's
        // instruction loop actually finishes (not on every RunFrame() call,
        // since a breakpoint can return out of this method partway through
        // a scanline and a later call needs to pick up exactly where it
        // left off rather than redoing that scanline's start-of-line work).
        private bool _scanlineStarted;
        private int _lineCycles;
        private bool _refreshedThisLine;
        private long _phaseStart;

        // Skips exactly one breakpoint check right after resuming from a
        // halt, so `continue`ing past a breakpoint executes the instruction
        // it's sitting on instead of instantly re-halting on the same PC
        // forever. Set for exactly one instruction each time RunFrame() is
        // (re)entered while IsHaltedAtBreakpoint is true.
        private bool _justResumedFromBreakpoint;

        // Which chip the pending halt came from, "cpu" when it was the S-CPU - see `man cpus`.
        public string HaltedCpu { get; private set; } = "cpu";

        // True while RunFrame() is halted on the coprocessor - lets a frontend
        // label the halt and read the right PC - see Venus_SA1.md §11.5.
        public bool IsHaltedOnCoprocessor => IsHaltedAtBreakpoint && HaltedCpu is "sa1" or "gsu" or "dsp";

        // Per-frame phase-timing accumulators - promoted from RunFrame()
        // locals to fields for the same mid-frame-resume reason as
        // _lineCycles above: a breakpoint can pause a frame partway through,
        // so these need to keep accumulating across however many RunFrame()
        // calls it actually takes to finish one frame, only resetting once
        // the frame genuinely completes. NOTE: if a breakpoint halt sits
        // open for a while (the user inspecting state at the F4 prompt),
        // the wall-clock gap while halted gets counted into whichever
        // phase's Stopwatch span was in progress at the time - an accepted
        // distortion of that one frame's LastFrame*Ms numbers, since this
        // is a debug/profiling readout, not anything gameplay-affecting.
        private long _cpuSpc700TicksAccum;
        private long _ppuTicksAccum;
        private long _hdmaTicksAccum;

        // Fractional carry for the CPU-cycle -> SPC700-cycle conversion
        // just below - see that call site's own comment for why this is
        // needed at all. Kept as a field (not a RunFrame local) so the
        // remainder isn't silently discarded/reset every RunFrame() call,
        // which would reintroduce slow, cumulative drift over a long play
        // session even after the per-call scaling itself was fixed.
        private int _spc700CycleRemainder;

        // headless: true skips window/texture creation entirely (see
        // Renderer.cs) - the console/Raylib build (Program.cs) wants a
        // real on-screen window, the Avalonia frontend (via
        // EmulatorSession) doesn't; it blits GetFrameBufferRgba() into
        // its own WriteableBitmap instead.
        public VenusCore(bool headless = false)
        {
            _headless = headless;
        }

        // Delegated to Cartridge, which can answer from the header alone - see
        // EmuSen_Firmware.md §1.
        public System.Collections.Generic.IReadOnlyList<Common.Firmware.FirmwareRequest> GetFirmwareRequirements(string romPath) =>
            Cartridge.FirmwareRequirements(romPath);

        public void LoadRom(string path)
        {
            Cart = new Cartridge(path);
            Spc700 = new Spc700();
            Bus = new MemoryBus(Cart, Spc700);
            Cpu = new Cpu(Bus);
            Renderer = new Renderer(headless: _headless);

            Region = Cart.Region;
            bool pal = Region == ConsoleRegion.Pal;
            _totalScanlines = pal ? PalScanlines : NtscScanlines;
            _masterClockHz = pal ? PalMasterClockHz : NtscMasterClockHz;
            Bus.Ppu.IsPal = pal;
            if (Cart.Sa1 != null) Cart.Sa1.TotalScanlines = _totalScanlines;

            // The NEC DSPs have their own crystal, so they need the master
            // rate to convert against - see Venus_NecDSP.md §4.1.
            if (Cart.NecDsp != null) Cart.NecDsp.MasterClockHz = _masterClockHz;

            _currentScanline = 0;
            TotalFrames = 0;
        }

        // Runs up to one frame's worth of scanlines: CPU/APU stepping,
        // HDMA, NMI, and PPU scanline compositing into the renderer's
        // pixel buffer. Doesn't touch any window/presentation surface -
        // every caller (Program.cs's FramePresenter, Avalonia's
        // EmulatorSession) gets pixels via GetFrameBufferRgba() afterward
        // and presents them itself.
        //
        // Can now return EARLY, mid-frame, if a breakpoint (or an armed
        // single-step) halts execution - check IsHaltedAtBreakpoint after
        // every call. When that happens, _currentScanline/_lineCycles/the
        // phase-timing accumulators are all left exactly as they were so
        // the NEXT call to RunFrame() resumes this same in-progress frame
        // instead of restarting it - see _scanlineStarted's own comment for
        // why the top-of-scanline block below only runs once per scanline
        // even across a halt/resume.
        public void RunFrame()
        {
            if (Bus is null || Cpu is null || Spc700 is null || Renderer is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            // Resuming from a prior halt: let exactly the instruction we
            // stopped in front of execute before re-arming breakpoint
            // checks, or `continue` would just instantly re-halt on the
            // same PC every time.
            if (IsHaltedAtBreakpoint)
            {
                IsHaltedAtBreakpoint = false;

                // Whichever CPU the halt was on is the one that has to skip a
                // check - arming the S-CPU's flag for an SA-1 halt would let
                // the SA-1 re-break instantly - see Venus_SA1.md §11.5.
                switch (HaltedCpu)
                {
                    case "sa1": Cart!.Sa1!.ResumeFromBreakpoint(); break;
                    case "gsu": Cart!.SuperFx!.ResumeFromBreakpoint(); break;
                    case "dsp": Cart!.NecDsp!.ResumeFromBreakpoint(); break;
                    case "spc": Bus.Spc700.ResumeFromBreakpoint(); break;
                    default: _justResumedFromBreakpoint = true; break;
                }
                HaltedCpu = "cpu";
            }

            while (true)
            {
                if (!_scanlineStarted)
                {
                    if (_currentScanline == 0)
                    {
                        Bus.Interrupts.EndVBlank();
                        Bus.Dma.InitHdma();
                    }

                    // Documented NMI-enable-during-vblank quirk - see
                    // PendingImmediateNmi's comment in InterruptController.cs.
                    // Checked here at the same once-per-scanline granularity
                    // as the H/V-IRQ check just below, for the same reason.
                    if (Bus.Interrupts.PendingImmediateNmi)
                    {
                        Bus.Interrupts.PendingImmediateNmi = false;
                        Cpu.Nmi();
                    }

                    // H/V-IRQ trigger check, done here (before this scanline's
                    // CPU code runs) so a handler's register changes - e.g.
                    // SMW's classic status-bar screen split - take effect in
                    // time for THIS scanline's render, not the next one.
                    // H-only fires every line; V-only and HV fire once per
                    // frame at the target scanline. This approximates real
                    // hardware's dot-precise H-position check as "the whole
                    // target scanline", since our timing runs at
                    // per-scanline granularity rather than per-dot.
                    if (DebugSettings.HvIrqEnabled)
                    {
                        if (Bus.Interrupts.HIrqEnabled && !Bus.Interrupts.VIrqEnabled)
                        {
                            Bus.Interrupts.RaiseTimerIrq();
                            Cpu.Irq();
                        }
                        else if (Bus.Interrupts.VIrqEnabled && _currentScanline == Bus.Interrupts.VTime)
                        {
                            Bus.Interrupts.RaiseTimerIrq();
                            Cpu.Irq();
                        }
                    }

                    _phaseStart = Stopwatch.GetTimestamp();
                    // Carry the boundary-crossing instruction's overshoot - see Venus_CPU.md §8.5a.
                    _lineCycles = _lineCycles > CyclesPerScanline ? _lineCycles - CyclesPerScanline : 0;
                    _refreshedThisLine = false;
                    Bus.LineCycles = _lineCycles;
                    Bus.ScanlineObserver?.Invoke(_currentScanline); // see `man runto`
                    _scanlineStarted = true;
                }

                while (_lineCycles < CyclesPerScanline)
                {
                    int pc24 = (Cpu.PB << 16) | Cpu.PC;
                    if (!_justResumedFromBreakpoint && Bus.BreakpointChecker != null && Bus.BreakpointChecker(pc24))
                    {
                        IsHaltedAtBreakpoint = true;
                        HaltedCpu = "cpu";
                        HaltedAddress = pc24;
                        return; // halted mid-scanline - _scanlineStarted/_lineCycles carry over to the next call
                    }
                    _justResumedFromBreakpoint = false;

                    int cpuCycles = Cpu.Step();

                    // Hardware stops the CPU once per scanline to refresh DRAM; the
                    // instruction spanning that point is charged for it - see Venus_CPU.md §8.7.
                    if (!_refreshedThisLine && _lineCycles + cpuCycles >= DramRefreshPosition)
                    {
                        _refreshedThisLine = true;
                        cpuCycles += DramRefreshClocks;
                    }

                    _lineCycles += cpuCycles;
                    Bus.LineCycles = _lineCycles;

                    // Cpu.Step()'s returned cycle count is real elapsed
                    // master clocks now (see its own comment) - each
                    // instruction's byte/cycle accounting is converted
                    // through the actual region speed of whatever address
                    // it touched (MemoryBus.GetAccessSpeedCycles), instead
                    // of this loop assuming a single flat "1 CPU cycle-unit
                    // = 6 master clocks" rate for the whole machine. SPC700
                    // cycle (its own independent 1.024MHz crystal,
                    // universally approximated as master/21 since it isn't
                    // derived from the main clock at all) = ~21 master
                    // clocks, so master clocks convert straight to SPC700
                    // cycles by dividing by 21 - scaled here with an
                    // explicit remainder carry (not float math) so the
                    // fractional part isn't silently dropped every single
                    // call.
                    //
                    // Before this dynamic per-access accounting existed,
                    // the whole machine was scaled at a flat FastROM-style
                    // 6-master-clocks-per-cycle rate regardless of the
                    // ROM's actual (usually SlowROM, 8mc) speed, which fed
                    // the SPC700 a fixed ~59474*6/21 SPC cycles/frame no
                    // matter what the CPU actually executed - a systematic
                    // ~7.5-8% audio-pacing undershoot relative to the real
                    // NTSC frame rate, confirmed via EmuSen.Pharaoh's
                    // `audiodump` verb (see Venus_APU.md). Master clocks are
                    // now real per-instruction quantities, so SPC700 pacing
                    // tracks actual elapsed hardware time directly, the
                    // same approach MesenCE's Spc.cpp uses.
                    // Exact 1.024MHz/21.477272MHz ratio, not /21 - see Venus_CPU.md §8.5b.
                    long scaledSpc700Cycles = (long)cpuCycles * ApuClockHz + _spc700CycleRemainder;
                    _spc700CycleRemainder = (int)(scaledSpc700Cycles % _masterClockHz);
                    Spc700.CycleBudget += (int)(scaledSpc700Cycles / _masterClockHz);
                    // Only run an instruction the budget actually covers - running
                    // past it made SPC700 port writes visible to the CPU up to a
                    // whole instruction early, corrupting audio uploads whose
                    // handshake acks before reading - see Venus_APU.md §1.6.
                    while (Spc700.CycleBudget >= Spc700.PeekStepCycles())
                    {
                        Spc700.Step();
                        if (Spc700.HaltedAtBreakpoint)
                        {
                            IsHaltedAtBreakpoint = true;
                            HaltedCpu = "spc";
                            HaltedAddress = Spc700.HaltedAddress;
                            return; // Spc700.CycleBudget carries the unspent cycles - see `man cpus`
                        }
                    }

                    // The SA-1 shares the master clock rather than having its
                    // own crystal, so it takes the same figure directly - see
                    // Venus_SA1.md §2.2. Its IRQ line into the S-CPU is polled
                    // here, at the same per-instruction granularity the two
                    // chips actually interleave at on hardware.
                    if (Cart!.Sa1 is { } sa1)
                    {
                        sa1.Run(cpuCycles);
                        if (sa1.HaltedAtBreakpoint)
                        {
                            IsHaltedAtBreakpoint = true;
                            HaltedCpu = "sa1";
                            HaltedAddress = sa1.HaltedAddress;
                            return; // sa1._clockBudget carries the unspent clocks - see Venus_SA1.md §11.5
                        }
                        if (sa1.ScpuIrqPending) Cpu.Irq();
                    }
                    else if (Cart.SuperFx is { } gsu)
                    {
                        gsu.Run(cpuCycles);
                        if (gsu.HaltedAtBreakpoint)
                        {
                            IsHaltedAtBreakpoint = true;
                            HaltedCpu = "gsu";
                            HaltedAddress = gsu.HaltedAddress;
                            return; // gsu._clockBudget carries the unspent clocks - see `man cpus`
                        }
                        if (gsu.ScpuIrqPending) Cpu.Irq();
                    }
                    else if (Cart.NecDsp is { } dsp)
                    {
                        // No IRQ line: the S-CPU polls SR instead - see Venus_NecDSP.md §3.2.
                        dsp.Run(cpuCycles);
                        if (dsp.HaltedAtBreakpoint)
                        {
                            IsHaltedAtBreakpoint = true;
                            HaltedCpu = "dsp";
                            HaltedAddress = dsp.HaltedAddress;
                            return; // dsp._clockBudget carries the unspent clocks - see Venus_NecDSP.md §9
                        }
                    }
                }

                long afterCpuSpc700 = Stopwatch.GetTimestamp();
                _cpuSpc700TicksAccum += afterCpuSpc700 - _phaseStart;

                Bus.CurrentScanline = _currentScanline;

                if (_currentScanline < 225)
                {
                    // Real hardware runs each scanline's HDMA transfer
                    // during H-blank, BEFORE that same scanline's active
                    // display period - so whatever a scanline's own HDMA
                    // table entry changes (a window edge, a VRAM byte, a
                    // CGRAM color) is already in effect by the time that
                    // scanline's pixels are actually drawn. This used to
                    // render first and run HDMA after, which fed every
                    // scanline the PREVIOUS scanline's HDMA update instead
                    // of its own - invisible for anything that changes
                    // slowly or not at all frame-to-frame, but a visible,
                    // wrong band of color for any effect that changes
                    // rapidly per scanline (found via a Zelda: A Link to
                    // the Past bridge/rain scene using indirect HDMA to
                    // drive per-scanline window/VRAM updates).
                    Bus.Dma.ExecuteHdma();
                    long afterPpu = Stopwatch.GetTimestamp();
                    _hdmaTicksAccum += afterPpu - afterCpuSpc700;

                    // HDMA above still runs; only the pixel pass drops - see
                    // EmuSen_Rewind_And_FastForward.md §2.2.
                    if (_currentScanline < 224 && !SkipRendering)
                    {
                        Renderer.RenderScanline(Bus, _currentScanline);
                    }
                    _ppuTicksAccum += Stopwatch.GetTimestamp() - afterPpu;
                }

                if (_currentScanline == InterruptController.AutoJoypadScanline)
                {
                    Bus.Interrupts.InVBlank = true;
                    Bus.Interrupts.RaiseVBlank();
                    Bus.Ppu.ReloadOamAddressForVBlank();
                    if (Bus.Interrupts.NmiEnabled) Cpu.Nmi();
                }

                // The auto-joypad read completes partway into vblank, not at its start - see Venus_Memory.md §4.4.
                if (_currentScanline == InterruptController.AutoJoypadLatchScanline && Bus.Interrupts.AutoJoypadEnabled)
                {
                    Bus.Input.LatchAutoJoypad();
                }

                _scanlineStarted = false;
                _currentScanline++;
                if (_currentScanline >= _totalScanlines)
                {
                    _currentScanline = 0;
                    TotalFrames++;
                    Bus.FrameCount = TotalFrames;
                    Bus.FrameObserver?.OnFrame(TotalFrames);

                    // A frame-boundary marker in the raw trace stream -
                    // added for a Super Metroid boot-hang investigation
                    // where the question was "what was the CPU doing
                    // relative to what the SPC700 was doing, in real
                    // time" and CpuVerboseLogging/Spc700VerboseLogging
                    // write to the same Console stream but with no shared
                    // reference point, making that correlation only
                    // possible by separately re-running with each flag on
                    // and cross-referencing by eye. Gated on the same
                    // flags that would otherwise be producing trace
                    // output at all - this is a no-op (and prints
                    // nothing) unless at least one of them is on, so it
                    // can't add noise to a run that isn't already tracing
                    // CPU/SPC700 execution.
                    if (DebugSettings.MasterLoggingEnabled &&
                        (DebugSettings.CpuVerboseLogging || DebugSettings.Spc700VerboseLogging))
                    {
                        Console.WriteLine($"[FRAME] {TotalFrames}");
                    }

                    // Periodic autosave - see Cartridge.SaveSram's own
                    // comment for why this is safe to call this often.
                    if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();

                    double ticksToMs = 1000.0 / Stopwatch.Frequency;
                    LastFrameCpuSpc700Ms = _cpuSpc700TicksAccum * ticksToMs;
                    LastFramePpuMs = _ppuTicksAccum * ticksToMs;
                    LastFrameHdmaMs = _hdmaTicksAccum * ticksToMs;
                    _cpuSpc700TicksAccum = 0;
                    _ppuTicksAccum = 0;
                    _hdmaTicksAccum = 0;

                    return; // one full frame done - hand control back to the caller
                }
            }
        }

        // Plain RGBA8888 bytes (ScreenWidth*ScreenHeight*4 long), ready to
        // copy straight into e.g. an Avalonia WriteableBitmap or a Raylib
        // texture. No Raylib types cross this boundary.
        public byte[] GetFrameBufferRgba()
        {
            if (Renderer is null)
            {
                throw new InvalidOperationException("GetFrameBufferRgba() called before LoadRom().");
            }
            return Renderer.GetFrameBufferRgba();
        }

        public int AudioSampleRate => EmuSen.Audio.AudioSettings.SampleRate;

        // Moved from EmuSen.Hotaru/Program.cs's own PumpAudio,
        // which used to reach directly into Spc700.Dsp.AudioBuffer (a real
        // SNES/S-DSP-specific type) - the exact same "core-agnostic caller
        // shouldn't touch Venus-specific internals" gap GetFrameBufferRgba
        // above already closed for video. Same drain logic, unchanged:
        // AudioBuffer is interleaved L/R shorts, so framesAvailable is
        // half its Count; capped at <maxFrames> so a caller with its own
        // per-call limit (avoiding a huge dump after a stall) doesn't need
        // to slice the result down itself.
        public short[] DequeueAudioSamples(int maxFrames)
        {
            if (Spc700 is null) return Array.Empty<short>();

            var buffer = Spc700.Dsp.AudioBuffer;
            int framesAvailable = buffer.Count / 2;
            int framesToSend = Math.Min(framesAvailable, maxFrames);
            if (framesToSend == 0) return Array.Empty<short>();

            var data = new short[framesToSend * 2];
            for (int i = 0; i < data.Length; i++) data[i] = buffer.Dequeue();
            return data;
        }

        // Full point-in-time snapshot of everything except the renderer
        // (which holds Raylib texture/window handles that have no
        // business in a save file, and is fully re-derivable from PPU
        // state anyway - the next RunFrame() call rebuilds it). See
        // StateSerializer's own header comment for the real limitation
        // here: no version header, no field tagging - a save state only
        // loads correctly against the exact build that created it.
        public void SaveState(string path)
        {
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var stream = new FileStream(path, FileMode.Create);
            SaveState(stream);
        }

        public void LoadState(string path)
        {
            using var stream = new FileStream(path, FileMode.Open);
            LoadState(stream);
        }

        // leaveOpen: the caller owns the stream - see ICore's own comment.
        public void SaveState(Stream stream)
        {
            if (Cart is null || Cpu is null || Bus is null || Spc700 is null)
            {
                throw new InvalidOperationException("SaveState() called before LoadRom().");
            }

            using var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            w.Write(StateMagic);
            w.Write(StateVersion);
            w.Write(TotalFrames);
            w.Write(_currentScanline);
            StateSerializer.Write(w, Cart);
            StateSerializer.Write(w, Cpu);
            StateSerializer.Write(w, Bus);
            StateSerializer.Write(w, Spc700);
            if (Cart.Sa1 != null) StateSerializer.Write(w, Cart.Sa1);
            if (Cart.SuperFx != null) StateSerializer.Write(w, Cart.SuperFx);
            if (Cart.NecDsp != null) StateSerializer.Write(w, Cart.NecDsp);
        }

        public void LoadState(Stream stream)
        {
            if (Cart is null || Cpu is null || Bus is null || Spc700 is null)
            {
                throw new InvalidOperationException("LoadState() called before LoadRom().");
            }

            using var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            // Pre-v1 files start straight in on TotalFrames with no header,
            // and carry the DSP RAM aliases - see EmuSen_Save_States.md §2.
            int version = ReadHeaderVersion(r, stream);
            bool legacy = version == 0;

            TotalFrames = r.ReadInt64();
            _currentScanline = r.ReadInt32();
            StateSerializer.Read(r, Cart, legacy);
            StateSerializer.Read(r, Cpu, legacy);
            StateSerializer.Read(r, Bus, legacy);
            StateSerializer.Read(r, Spc700, legacy);

            // Only v2 onward carries it, and only for an SA-1 cartridge - a v1
            // file can't be one, since SA-1 games couldn't run when v1 was written.
            if (version >= 2 && Cart.Sa1 != null) StateSerializer.Read(r, Cart.Sa1);
            if (version >= 2 && Cart.SuperFx != null) StateSerializer.Read(r, Cart.SuperFx);

            // Same reasoning as v2's, one chip later: a v2 file can't be a NEC
            // DSP cartridge, since none could run when v2 was written.
            if (version >= 3 && Cart.NecDsp != null) StateSerializer.Read(r, Cart.NecDsp);
        }

        // Returns the format version, or 0 for a pre-v1 file (stream rewound).
        private static int ReadHeaderVersion(BinaryReader r, Stream stream)
        {
            if (!stream.CanSeek)
            {
                throw new NotSupportedException("LoadState() needs a seekable stream to tell a versioned state from a pre-v1 one.");
            }

            long start = stream.Position;
            if (stream.Length - start < sizeof(uint) + sizeof(int)) return 0;

            if (r.ReadUInt32() != StateMagic) { stream.Position = start; return 0; }

            int version = r.ReadInt32();
            if (version > StateVersion)
            {
                throw new InvalidDataException($"Save state is version {version}; this build understands up to {StateVersion}.");
            }
            // Magic matched but the version is nonsense - a pre-v1 file whose
            // TotalFrames happened to collide. Treat it as one.
            if (version < 1) { stream.Position = start; return 0; }
            return version;
        }

        public void SaveSram()
        {
            Cart?.SaveSram();
        }
    }
}
