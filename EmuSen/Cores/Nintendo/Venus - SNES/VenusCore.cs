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
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.Venus
{
    // The SNES ICore, and the one place the per-scanline loop lives - see EmuSen_Multicore.md.
    public partial class VenusCore : ICore, IFrameProfiler, ICoprocessorHalt, ICoprocessorLoad, ITraceFlushable
    {
        // Real master clocks per scanline, not back-derived CPU cycles - see Venus_CPU.md §8.
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
        // Each version appends a chip's state, only for a cart carrying it - see EmuSen_Save_States.md §3.
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

        // Per-phase breakdown of the last RunFrame, at scanline granularity - see Venus_PPU.md §13.
        public double LastFrameCpuSpc700Ms { get; private set; }
        public double LastFramePpuMs { get; private set; }
        public double LastFrameHdmaMs { get; private set; }

        // Sub-breakdown of LastFramePpuMs, straight from Renderer - see Venus_PPU.md §13.
        public double LastFrameObjEvalMs => Renderer?.LastFrameObjEvalMs ?? 0;
        public double LastFrameBlendMs => Renderer?.LastFrameBlendMs ?? 0;

        // The IFrameProfiler view of the seven fields above; "ppu/" ones are inside ppu, not beside it.
        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => new[]
        {
            ("cpu+spc700", LastFrameCpuSpc700Ms),
            ("ppu", LastFramePpuMs),
            ("hdma", LastFrameHdmaMs),
            ("ppu/obj-eval", LastFrameObjEvalMs),
            ("ppu/blend", LastFrameBlendMs),
            ("ppu/main-composite", LastFrameMainCompositeMs),
            ("ppu/sub-composite", LastFrameSubCompositeMs),
        };

        public string HaltedProcessorName => IsHaltedOnCoprocessor ? "SA-1" : "S-CPU";

        // Whether the game is starving or double-clocking the chip - see Venus_SA1.md §2.3.
        public IReadOnlyList<CoprocessorClocks> CoprocessorClocks => Cart?.Sa1 is { } sa1
            ? new[] { new CoprocessorClocks("sa-1", sa1.ExecutedMasterClocks, sa1.OfferedMasterClocks, MasterClocksPerFrame) }
            : Array.Empty<CoprocessorClocks>();

        // Both traces, so a caller does not need to know this core has two processors.
        public void FlushVerboseTrace()
        {
            Cpu?.FlushVerboseTrace();
            Spc700?.FlushVerboseTrace();
        }

        // Splits the per-scanline BG/OBJ composite into main vs sub screen - see Venus_PPU.md §13.
        public double LastFrameMainCompositeMs => Renderer?.LastFrameMainCompositeMs ?? 0;
        public double LastFrameSubCompositeMs => Renderer?.LastFrameSubCompositeMs ?? 0;

        // Pseudo-hi-res can make a frame 512 wide, so this is read, not fixed - see Venus_CPU.md §12.
        public int ScreenWidth => Renderer?.FrameWidth ?? 256;
        public int ScreenHeight => 224;

        // NTSC 21477272/(262*1364) ~= 60.098, PAL 21281370/(312*1364) ~= 50.007 - see Venus_CPU.md §8.5b.
        public double FrameRateHz => _masterClockHz / (_totalScanlines * (double)CyclesPerScanline);

        // What a coprocessor's own per-frame clock total must match - see Venus_SA1.md §2.3.
        public long MasterClocksPerFrame => (long)_totalScanlines * CyclesPerScanline;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        // All twelve; PadButton's names were chosen to match SnesButton - see EmuSen_Input.md §3.
        public static readonly PadButton[] PadButtons = (PadButton[])Enum.GetValues(typeof(PadButton));

        // Static, so the rebind window can list this console's pad without a ROM - see EmuSen_Input.md §5.1.
        public IReadOnlyList<PadButton> SupportedButtons => PadButtons;

        // Venus's own ports are 1-based, so the generic 0-based port shifts here.
        public void SetButton(int port, PadButton button, bool pressed) =>
            Bus?.Input.SetButton((Controllers.SnesButton)button, pressed, port + 1);

        // Drops the per-scanline pixel pass only - see EmuSen_Rewind_And_FastForward.md §2.2.
        public bool SkipRendering { get; set; }

        // On the concrete type, because only a frontend's debug prompt reads it - see Venus_CPU.md §12.
        public bool IsHaltedAtBreakpoint { get; private set; }

        // Only meaningful while IsHaltedAtBreakpoint is true.
        public int HaltedAddress { get; private set; }

        // Fields, because a halt can return mid-scanline - see Venus_CPU.md §12.
        private bool _scanlineStarted;
        private int _lineCycles;
        private bool _refreshedThisLine;
        private long _phaseStart;

        // Taken where the old inline block took afterCpuSpc700, so the phase split is unchanged.
        private long _phaseEndCpu;

        // Or `continue` re-halts forever on the same PC - see Venus_CPU.md §12.
        private bool _justResumedFromBreakpoint;

        // Which chip the pending halt came from, "cpu" when it was the S-CPU - see `man cpus`.
        public string HaltedCpu { get; private set; } = "cpu";

        // Lets a frontend label the halt and read the right PC - see Venus_SA1.md §11.5.
        public bool IsHaltedOnCoprocessor => IsHaltedAtBreakpoint && HaltedCpu is "sa1" or "gsu" or "dsp";

        // Fields for the same mid-frame reason; a long halt distorts them - see Venus_CPU.md §12.
        private long _cpuSpc700TicksAccum;
        private long _ppuTicksAccum;
        private long _hdmaTicksAccum;

        // A field, or the remainder resets every call and drift returns - see Venus_CPU.md §12.
        private int _spc700CycleRemainder;

        // headless skips window and texture creation entirely - see Renderer.cs.
        public VenusCore(bool headless = false)
        {
            _headless = headless;
        }

        // Delegated to Cartridge, which answers from the header alone - see EmuSen_Firmware.md §1.
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

            // The NEC DSPs have their own crystal and need the master rate - see Venus_NecDSP.md §4.1.
            if (Cart.NecDsp != null) Cart.NecDsp.MasterClockHz = _masterClockHz;

            _currentScanline = 0;
            TotalFrames = 0;
            ResetSchedule();
        }

        // Can return early mid-frame; check IsHaltedAtBreakpoint after every call - see Venus_CPU.md §12.
        public void RunFrame()
        {
            if (Bus is null || Cpu is null || Spc700 is null || Renderer is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            // Let the halted instruction run before re-arming, or `continue` loops - see Venus_CPU.md §12.
            if (IsHaltedAtBreakpoint)
            {
                IsHaltedAtBreakpoint = false;

                // The CPU the halt was on is the one that skips a check - see Venus_SA1.md §11.5.
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

                    // The NMI-enable-during-vblank quirk - see InterruptController.PendingImmediateNmi.
                    if (Bus.Interrupts.PendingImmediateNmi)
                    {
                        Bus.Interrupts.PendingImmediateNmi = false;
                        Cpu.Nmi();
                    }

                    // Before this scanline's code runs, so a split takes effect on this line, not the next.
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
                    // The overshoot carries itself now: it is the master clock minus the line's own start - see Venus_CPU.md §8.5a.
                    _lineCycles = (int)(_masterClock - _lineStartClock);
                    _refreshedThisLine = false;
                    Bus.LineCycles = _lineCycles;
                    Bus.ScanlineObserver?.Invoke(_currentScanline); // see `man runto`
                    _scanlineStarted = true;
                }

                long lineDeadline = NextScanlineBoundary;

                while (_masterClock < lineDeadline)
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

                    // DRAM refresh stops the CPU once a scanline; the spanning instruction pays - see Venus_CPU.md §8.7.
                    if (!_refreshedThisLine && _lineCycles + cpuCycles >= DramRefreshPosition)
                    {
                        _refreshedThisLine = true;
                        cpuCycles += DramRefreshClocks;
                    }

                    _masterClock += cpuCycles;
                    _lineCycles = (int)(_masterClock - _lineStartClock);
                    Bus.LineCycles = _lineCycles;

                    // Real master clocks converted with a remainder carry, not float - see Venus_CPU.md §8.5b.
                    long scaledSpc700Cycles = (long)cpuCycles * ApuClockHz + _spc700CycleRemainder;
                    _spc700CycleRemainder = (int)(scaledSpc700Cycles % _masterClockHz);
                    Spc700.CycleBudget += (int)(scaledSpc700Cycles / _masterClockHz);
                    // Running past the budget made port writes visible a whole instruction early - see Venus_APU.md §1.6.
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

                    // The SA-1 shares the master clock, so it takes the figure directly - see Venus_SA1.md §2.2.
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

                _phaseEndCpu = Stopwatch.GetTimestamp();
                _cpuSpc700TicksAccum += _phaseEndCpu - _phaseStart;

                // Everything that used to follow this loop now lives in EndScanline.
                EndScanline();

                if (_frameComplete)
                {
                    _frameComplete = false;
                    return; // one full frame done - hand control back to the caller
                }
            }
        }

        // Plain RGBA8888, ready to copy; no renderer types cross this boundary.
        public byte[] GetFrameBufferRgba()
        {
            if (Renderer is null)
            {
                throw new InvalidOperationException("GetFrameBufferRgba() called before LoadRom().");
            }
            return Renderer.GetFrameBufferRgba();
        }

        public int AudioSampleRate => EmuSen.Audio.AudioSettings.SampleRate;

        // Interleaved L/R shorts, so frames are half the count; capped for the caller.
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

        // Everything but the renderer, which is re-derivable from PPU state - see EmuSen_Save_States.md.
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

            // Pre-v1 files start at TotalFrames with no header - see EmuSen_Save_States.md §2.
            int version = ReadHeaderVersion(r, stream);
            bool legacy = version == 0;

            TotalFrames = r.ReadInt64();
            _currentScanline = r.ReadInt32();
            StateSerializer.Read(r, Cart, legacy);
            StateSerializer.Read(r, Cpu, legacy);
            StateSerializer.Read(r, Bus, legacy);
            StateSerializer.Read(r, Spc700, legacy);

            // v2 onward, and only for an SA-1 cart: a v1 file cannot be one.
            if (version >= 2 && Cart.Sa1 != null) StateSerializer.Read(r, Cart.Sa1);
            if (version >= 2 && Cart.SuperFx != null) StateSerializer.Read(r, Cart.SuperFx);

            // Same reasoning one chip later: a v2 file cannot be a NEC DSP cart.
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
            // Magic matched but the version is nonsense, so treat it as pre-v1.
            if (version < 1) { stream.Position = start; return 0; }
            return version;
        }

        public void SaveSram()
        {
            Cart?.SaveSram();
        }
    }
}
