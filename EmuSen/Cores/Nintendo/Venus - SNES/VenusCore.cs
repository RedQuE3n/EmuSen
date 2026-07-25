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
        private const int CyclesPerScanline = 227;
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

        // Not a fixed constant - pseudo-hi-res (SETINI bit 3) can make a
        // frame 512 pixels wide instead of 256. See Renderer.cs's
        // FrameWidth field for the full citation. Defaults to 256 before
        // a ROM is loaded (no renderer exists yet).
        public int ScreenWidth => Renderer?.FrameWidth ?? 256;
        public int ScreenHeight => 224;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

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

        // Runs exactly one frame's worth of scanlines: CPU/APU stepping,
        // HDMA, NMI, and PPU scanline compositing into the renderer's
        // pixel buffer. Doesn't touch any window/presentation surface -
        // every caller (Program.cs's FramePresenter, Avalonia's
        // EmulatorSession) gets pixels via GetFrameBufferRgba() afterward
        // and presents them itself.
        public void RunFrame()
        {
            if (Bus is null || Cpu is null || Spc700 is null || Renderer is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            long cpuSpc700Ticks = 0;
            long ppuTicks = 0;
            long hdmaTicks = 0;

            while (true)
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

                long phaseStart = Stopwatch.GetTimestamp();

                int lineCycles = 0;
                Bus.LineCycles = 0;
                while (lineCycles < CyclesPerScanline)
                {
                    int cpuCycles = Cpu.Step();
                    lineCycles += cpuCycles;
                    Bus.LineCycles = lineCycles;

                    Spc700.CycleBudget += cpuCycles;
                    while (Spc700.CycleBudget >= 21)
                    {
                        Spc700.Step();
                    }
                }

                long afterCpuSpc700 = Stopwatch.GetTimestamp();
                cpuSpc700Ticks += afterCpuSpc700 - phaseStart;

                Bus.CurrentScanline = _currentScanline;

                if (_currentScanline < 225)
                {
                    if (_currentScanline < 224)
                    {
                        Renderer.RenderScanline(Bus, _currentScanline);
                    }
                    long afterPpu = Stopwatch.GetTimestamp();
                    ppuTicks += afterPpu - afterCpuSpc700;

                    Bus.Dma.ExecuteHdma();
                    hdmaTicks += Stopwatch.GetTimestamp() - afterPpu;
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

                _currentScanline++;
                if (_currentScanline >= TotalScanlines)
                {
                    _currentScanline = 0;
                    TotalFrames++;
                    Bus.FrameCount = TotalFrames;
                    Bus.FrameObserver?.OnFrame(TotalFrames);

                    // Periodic autosave - see Cartridge.SaveSram's own
                    // comment for why this is safe to call this often.
                    if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();

                    double ticksToMs = 1000.0 / Stopwatch.Frequency;
                    LastFrameCpuSpc700Ms = cpuSpc700Ticks * ticksToMs;
                    LastFramePpuMs = ppuTicks * ticksToMs;
                    LastFrameHdmaMs = hdmaTicks * ticksToMs;

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
