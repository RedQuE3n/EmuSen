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

namespace EmuSen.Cores.Nintendo.Venus
{
    // The SNES's ICore implementation - owns and drives Cpu/MemoryBus/
    // Spc700/Renderer frame-by-frame. This is the ONE place the per-
    // scanline timing loop lives now; it used to exist as two separately-
    // maintained copies (the console frontend's Main loop, now
    // EmuSen.RaylibFrontend/Program.cs, and Common/EmulatorSession.cs's
    // RunFrame()) that had to be kept in sync
    // by hand - EmulatorSession's own header comment even said so
    // explicitly. Both now construct a VenusCore and call RunFrame() on
    // it instead.
    //
    // Exposes Cart/Spc700/Bus/Cpu/Renderer as public properties beyond
    // what ICore requires. This is deliberate, not a leaky abstraction:
    // Program.cs's debug toolchain (SnesDebugTarget, DebugCommandProcessor,
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
        private const int CyclesPerScanline = 1364;
        private const int TotalScanlines = 262;
        private const int SaveEveryNFrames = 300; // ~5 seconds at 60fps

        private readonly bool _headless;
        private int _currentScanline;

        public Cartridge? Cart { get; private set; }
        public Spc700? Spc700 { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Cpu? Cpu { get; private set; }
        public Renderer? Renderer { get; private set; }

        public string CoreName => "SNES";

        // Per-phase breakdown of the last completed RunFrame() call -
        // a temporary profiling aid for the "why is gameplay slower than
        // 60fps" investigation (both frontends print an [FPS] readout
        // showing RunFrame() itself taking ~17-18ms during real gameplay).
        // Timed at per-scanline granularity, not per-instruction - timing
        // every single Cpu.Step() call would add Stopwatch overhead
        // comparable to the work being measured (a fast interpreter's
        // per-instruction cost and Stopwatch.GetTimestamp()'s own cost are
        // the same order of magnitude), which would distort the very
        // numbers this exists to produce. CpuSpc700 bundles CPU+SPC700
        // stepping as one phase for the same reason - splitting further
        // would mean timing individual Cpu.Step() calls again.
        public double LastFrameCpuSpc700Ms { get; private set; }
        public double LastFramePpuMs { get; private set; }
        public double LastFrameHdmaMs { get; private set; }

        // Sub-breakdown of LastFramePpuMs, sourced straight from Renderer -
        // see that class's own comment (Renderer.Scanline.cs) for why this
        // exists: LastFramePpuMs stopped dropping as much as expected once
        // the BG1-4 double-decode fix landed, meaning sprite evaluation or
        // the final color-math/blend loop (neither touched by that fix) is
        // the actual dominant cost within it.
        public double LastFrameObjEvalMs => Renderer?.LastFrameObjEvalMs ?? 0;
        public double LastFrameBlendMs => Renderer?.LastFrameBlendMs ?? 0;

        // objEval+blend turned out tiny (~0.6ms) against ~13ms of total ppu
        // time - these two split the rest (per-scanline BG/OBJ compositing)
        // into main-screen vs sub-screen, to check whether sub-screen
        // rendering (often skippable/cheap if a scene doesn't really use
        // it) is doubling the real per-pixel BG decode cost unnecessarily.
        public double LastFrameMainCompositeMs => Renderer?.LastFrameMainCompositeMs ?? 0;
        public double LastFrameSubCompositeMs => Renderer?.LastFrameSubCompositeMs ?? 0;

        // Not a fixed constant - pseudo-hi-res (SETINI bit 3) can make a
        // frame 512 pixels wide instead of 256. See Renderer.cs's
        // FrameWidth field for the full citation. Defaults to 256 before
        // a ROM is loaded (no renderer exists yet).
        public int ScreenWidth => Renderer?.FrameWidth ?? 256;
        public int ScreenHeight => 224;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        // True from the instant RunFrame() halts on a breakpoint (or an
        // armed single-step - see BreakpointRegistry) until the next
        // RunFrame() call resumes past it. Kept on the concrete type
        // rather than ICore, same call EmulatorSession's own comment
        // already makes for the LastFrame*Ms profiling properties: nothing
        // consumes this except the console frontend's own debug prompt
        // (EmuSen.RaylibFrontend/Program.cs), which already holds a
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
        private long _phaseStart;

        // Skips exactly one breakpoint check right after resuming from a
        // halt, so `continue`ing past a breakpoint executes the instruction
        // it's sitting on instead of instantly re-halting on the same PC
        // forever. Set for exactly one instruction each time RunFrame() is
        // (re)entered while IsHaltedAtBreakpoint is true.
        private bool _justResumedFromBreakpoint;

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

        public void LoadRom(string path)
        {
            Cart = new Cartridge(path);
            Spc700 = new Spc700();
            Bus = new MemoryBus(Cart, Spc700);
            Cpu = new Cpu(Bus);
            Renderer = new Renderer(headless: _headless);

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
                _justResumedFromBreakpoint = true;
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
                    _lineCycles = 0;
                    Bus.LineCycles = 0;
                    _scanlineStarted = true;
                }

                while (_lineCycles < CyclesPerScanline)
                {
                    int pc24 = (Cpu.PB << 16) | Cpu.PC;
                    if (!_justResumedFromBreakpoint && Bus.BreakpointChecker != null && Bus.BreakpointChecker(pc24))
                    {
                        IsHaltedAtBreakpoint = true;
                        HaltedAddress = pc24;
                        return; // halted mid-scanline - _scanlineStarted/_lineCycles carry over to the next call
                    }
                    _justResumedFromBreakpoint = false;

                    int cpuCycles = Cpu.Step();
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
                    // NTSC frame rate, confirmed via EmuSen.HeadlessDebug's
                    // `audiodump` verb (see Venus_APU.md). Master clocks are
                    // now real per-instruction quantities, so SPC700 pacing
                    // tracks actual elapsed hardware time directly, the
                    // same approach MesenCE's Spc.cpp uses.
                    int scaledSpc700Cycles = cpuCycles + _spc700CycleRemainder;
                    _spc700CycleRemainder = scaledSpc700Cycles % 21;
                    Spc700.CycleBudget += scaledSpc700Cycles / 21;
                    while (Spc700.CycleBudget > 0)
                    {
                        Spc700.Step();
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

                    if (_currentScanline < 224)
                    {
                        Renderer.RenderScanline(Bus, _currentScanline);
                    }
                    _ppuTicksAccum += Stopwatch.GetTimestamp() - afterPpu;
                }

                if (_currentScanline == 225)
                {
                    Bus.Interrupts.InVBlank = true;
                    Bus.Interrupts.RaiseVBlank();
                    if (Bus.Interrupts.NmiEnabled) Cpu.Nmi();

                    // Real hardware automatically reads the controller
                    // once per frame right at the start of vblank; mirror
                    // that timing here.
                    Bus.Input.LatchAutoJoypad();
                }

                _scanlineStarted = false;
                _currentScanline++;
                if (_currentScanline >= TotalScanlines)
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

        // Full point-in-time snapshot of everything except the renderer
        // (which holds Raylib texture/window handles that have no
        // business in a save file, and is fully re-derivable from PPU
        // state anyway - the next RunFrame() call rebuilds it). See
        // StateSerializer's own header comment for the real limitation
        // here: no version header, no field tagging - a save state only
        // loads correctly against the exact build that created it.
        public void SaveState(string path)
        {
            if (Cart is null || Cpu is null || Bus is null || Spc700 is null)
            {
                throw new InvalidOperationException("SaveState() called before LoadRom().");
            }

            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            using var stream = new FileStream(path, FileMode.Create);
            using var w = new BinaryWriter(stream);

            w.Write(TotalFrames);
            w.Write(_currentScanline);
            StateSerializer.Write(w, Cart);
            StateSerializer.Write(w, Cpu);
            StateSerializer.Write(w, Bus);
            StateSerializer.Write(w, Spc700);
        }

        public void LoadState(string path)
        {
            if (Cart is null || Cpu is null || Bus is null || Spc700 is null)
            {
                throw new InvalidOperationException("LoadState() called before LoadRom().");
            }

            using var stream = new FileStream(path, FileMode.Open);
            using var r = new BinaryReader(stream);

            TotalFrames = r.ReadInt64();
            _currentScanline = r.ReadInt32();
            StateSerializer.Read(r, Cart);
            StateSerializer.Read(r, Cpu);
            StateSerializer.Read(r, Bus);
            StateSerializer.Read(r, Spc700);
        }

        public void SaveSram()
        {
            Cart?.SaveSram();
        }
    }
}
