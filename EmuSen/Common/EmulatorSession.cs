using System;
using System.IO;
using EmuSen.Memory;
using EmuSen.Apu;
using EmuSen.Processor;
using EmuSen.Video;
using EmuSen.Debug;

namespace EmuSen.Common
{
    // Drives one SNES core (CPU/PPU/APU/memory) frame-by-frame, independent of
    // any particular windowing/UI toolkit. Extracted from Frontend/Program.cs's
    // Main loop so a non-Raylib frontend (the Avalonia EmuSen.Frontend project)
    // can call LoadRom() once and RunFrame() on its own schedule, instead of
    // the core owning its own window and while-loop the way Program.cs's
    // console/Raylib build still does.
    //
    // Frontend/Program.cs is untouched and doesn't use this class - the two
    // entry points run independently for now. Unifying them so the Raylib
    // build also goes through EmulatorSession is a reasonable follow-up, not
    // required for this to work.
    //
    // Known gap: real keyboard/gamepad input isn't wired up here.
    // InputBindings.ApplyInput() (used by Program.cs) reads Raylib's key
    // state directly, which only works with a live Raylib window - not
    // available in this headless path. A UI-driven frontend needs its own
    // input plumbing (e.g. Avalonia key events writing into Bus.Input
    // directly) - RunFrame() runs the core forward either way, controller
    // state just won't move yet.
    public class EmulatorSession
    {
        // ScreenWidth used to be a compile-time constant (always 256). It's
        // now an instance property delegating to the renderer's actual
        // current frame width, since pseudo-hi-res (SETINI bit 3) can make
        // a frame 512 pixels wide - see Renderer.cs's FrameWidth field for
        // the full citation. Defaults to 256 before a ROM is loaded (no
        // renderer exists yet). ScreenHeight stays a plain constant -
        // hi-res only ever affects horizontal width in this project (see
        // Renderer.cs), vertical scanline count is untouched by it.
        public int ScreenWidth => _renderer?.FrameWidth ?? 256;
        public const int ScreenHeight = 224;

        private const int CyclesPerScanline = 227;
        private const int TotalScanlines = 262;
        private const int SaveEveryNFrames = 300; // ~5 seconds at 60fps, matches Program.cs

        private Cartridge? _cart;
        private Spc700? _spc700;
        private MemoryBus? _bus;
        private Cpu? _cpu;
        private Renderer? _renderer;

        private int _currentScanline;

        public long TotalFrames { get; private set; }
        public bool IsRomLoaded => _bus != null;
        public MemoryBus Bus => _bus ?? throw new InvalidOperationException("LoadRom() hasn't been called yet.");

        public void LoadRom(string path)
        {
            _cart = new Cartridge(path);
            _spc700 = new Spc700();
            _bus = new MemoryBus(_cart, _spc700);
            _cpu = new Cpu(_bus);

            // headless: true - no window/textures. This session's caller
            // supplies its own presentation surface via GetFrameBufferRgba().
            _renderer = new Renderer(headless: true);

            _currentScanline = 0;
            TotalFrames = 0;
        }

        // Runs exactly one frame's worth of scanlines: CPU/APU stepping, HDMA,
        // NMI, and PPU scanline compositing into the renderer's pixel buffer.
        // This is a straight extraction of the loop body in Program.cs's
        // Main - see that file for the original window-owning version, and
        // keep the two in sync if the timing logic changes in either place.
        public void RunFrame()
        {
            if (_bus is null || _cpu is null || _spc700 is null || _renderer is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            while (true)
            {
                if (_currentScanline == 0)
                {
                    _bus.Interrupts.EndVBlank();
                    _bus.Dma.InitHdma();
                }

                // Documented NMI-enable-during-vblank quirk - see
                // PendingImmediateNmi's comment in InterruptController.cs.
                // Checked here at the same once-per-scanline granularity as
                // the H/V-IRQ check just below, for the same reason.
                if (_bus.Interrupts.PendingImmediateNmi)
                {
                    _bus.Interrupts.PendingImmediateNmi = false;
                    _cpu.Nmi();
                }

                // See Program.cs for the full explanation of this check's timing
                // and the H/V-IRQ regression note - kept identical here.
                if (DebugSettings.HvIrqEnabled)
                {
                    if (_bus.Interrupts.HIrqEnabled && !_bus.Interrupts.VIrqEnabled)
                    {
                        _bus.Interrupts.RaiseTimerIrq();
                        _cpu.Irq();
                    }
                    else if (_bus.Interrupts.VIrqEnabled && _currentScanline == _bus.Interrupts.VTime)
                    {
                        _bus.Interrupts.RaiseTimerIrq();
                        _cpu.Irq();
                    }
                }

                int lineCycles = 0;
                _bus.LineCycles = 0;
                while (lineCycles < CyclesPerScanline)
                {
                    int cpuCycles = _cpu.Step();
                    lineCycles += cpuCycles;
                    _bus.LineCycles = lineCycles;

                    _spc700.CycleBudget += cpuCycles;
                    while (_spc700.CycleBudget >= 21)
                    {
                        _spc700.Step();
                    }
                }

                _bus.CurrentScanline = _currentScanline;

                if (_currentScanline < 225)
                {
                    if (_currentScanline < 224)
                    {
                        _renderer.RenderScanline(_bus, _currentScanline);
                    }
                    _bus.Dma.ExecuteHdma();
                }

                if (_currentScanline == 225)
                {
                    _bus.Interrupts.InVBlank = true;
                    _bus.Interrupts.RaiseVBlank();
                    if (_bus.Interrupts.NmiEnabled) _cpu.Nmi();
                    _bus.Input.LatchAutoJoypad();
                }

                _currentScanline++;
                if (_currentScanline >= TotalScanlines)
                {
                    _currentScanline = 0;
                    TotalFrames++;

                    // Periodic autosave - see Cartridge.SaveSram's own comment
                    // for why this is safe to call this often. Same interval
                    // as the console/Raylib build (Program.cs) for consistency.
                    if (TotalFrames % SaveEveryNFrames == 0) _cart!.SaveSram();

                    return; // one full frame done - hand control back to the caller
                }
            }
        }

        // Call when the frontend is closing/unloading, for a final flush -
        // periodic autosave above covers most cases, but this catches
        // whatever's changed since the last interval on a clean exit.
        public void SaveSram()
        {
            _cart?.SaveSram();
        }

        // Save states: a full point-in-time snapshot of everything except
        // the renderer (which holds Raylib texture/window handles that have
        // no business in a save file, and is fully re-derivable from PPU
        // state anyway - the next RunFrame() call rebuilds it). Walks
        // cart/cpu/bus/spc700 individually via StateSerializer rather than
        // the whole EmulatorSession object specifically to avoid ever
        // reaching _renderer through reflection.
        //
        // See StateSerializer's own header comment for the real limitation
        // here: no version header, no field tagging - a save state only
        // loads correctly against the exact build that created it.
        public void SaveState(string path)
        {
            if (_cart is null || _cpu is null || _bus is null || _spc700 is null)
            {
                throw new InvalidOperationException("SaveState() called before LoadRom().");
            }

            using var stream = new FileStream(path, FileMode.Create);
            using var w = new BinaryWriter(stream);

            w.Write(TotalFrames);
            w.Write(_currentScanline);
            StateSerializer.Write(w, _cart);
            StateSerializer.Write(w, _cpu);
            StateSerializer.Write(w, _bus);
            StateSerializer.Write(w, _spc700);
        }

        public void LoadState(string path)
        {
            if (_cart is null || _cpu is null || _bus is null || _spc700 is null)
            {
                throw new InvalidOperationException("LoadState() called before LoadRom().");
            }

            using var stream = new FileStream(path, FileMode.Open);
            using var r = new BinaryReader(stream);

            TotalFrames = r.ReadInt64();
            _currentScanline = r.ReadInt32();
            StateSerializer.Read(r, _cart);
            StateSerializer.Read(r, _cpu);
            StateSerializer.Read(r, _bus);
            StateSerializer.Read(r, _spc700);
        }

        // Plain RGBA8888 bytes (ScreenWidth*ScreenHeight*4 long), ready to copy
        // straight into e.g. an Avalonia WriteableBitmap. No Raylib types cross
        // this boundary.
        public byte[] GetFrameBufferRgba()
        {
            if (_renderer is null)
            {
                throw new InvalidOperationException("GetFrameBufferRgba() called before LoadRom().");
            }
            return _renderer.GetFrameBufferRgba();
        }
    }
}
