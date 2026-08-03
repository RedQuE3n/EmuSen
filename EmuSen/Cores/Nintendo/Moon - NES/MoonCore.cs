using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Input;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.Moon.Processor;
using EmuSen.Cores.Nintendo.Moon.Video;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Moon
{
    // The NES's ICore implementation; the hardware is public beyond the interface - see Moon_Core.md §1.
    public partial class MoonCore : global::EmuSen.Cores.ICore
    {
        public const int MasterClockHz = 21477272;
        public const int MasterClocksPerCpuCycle = 12;
        public const int MasterClocksPerDot = 4;
        public const int DotsPerScanline = 341;
        public const int MasterClocksPerScanline = DotsPerScanline * MasterClocksPerDot;

        private const int SaveEveryNFrames = 300;

        // "MOON" little-endian, then the format version - see EmuSen_Save_States.md §3.
        private const uint StateMagic = 0x4E4F4F4D;
        private const int StateVersion = 1;

        private int _currentScanline;

        public Cartridge? Cart { get; private set; }
        public Cpu? Cpu { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Ppu? Ppu { get; private set; }
        public Apu.Apu? Apu { get; private set; }

        // Owned here rather than in the debug target so a label set outlives any one prompt session.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();
        public CheatRegistry Cheats { get; } = new();
        public CoverageRegistry Coverage { get; } = new();
        public LabelRegistry Labels { get; } = new();

        public string CoreName => "NES";

        public int ScreenWidth => Video.Ppu.ScreenWidth;
        public int ScreenHeight => Video.Ppu.ScreenHeight;

        // 21477272 / (262 * 1364) ~= 60.0985 - see Moon_Core.md §2.
        public double FrameRateHz => MasterClockHz / (double)(Video.Ppu.TotalScanlines * MasterClocksPerScanline);

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        public bool SkipRendering { get; set; }

        // Nothing is synthesized yet, but a caller still needs a rate up front - see Moon_APU.md.
        public int AudioSampleRate => 44100;

        // Set by the debug target so its providers re-snapshot once a frame, off the emulation thread.
        [SkipInState] public Action? FrameRefresh;

        // Halted in front of a breakpoint, with the frame left mid-flight for the next call to resume.
        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        public void LoadRom(string path)
        {
            Cart = Cartridge.Load(path);
            Ppu = new Ppu(Cart);
            Apu = new Apu.Apu();
            Bus = new MemoryBus(Cart, Ppu, Apu) { WriteObserver = null };
            Cpu = new Cpu(Bus);
            Bus.Cpu = Cpu;

            Ppu.Reset();
            Apu.Reset();
            Bus.Reset();
            Cpu.Reset();

            TotalFrames = 0;
            IsHaltedAtBreakpoint = false;
            ResetSchedule();
        }

        public void RunFrame()
        {
            if (Bus is null || Cpu is null || Ppu is null || Apu is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            if (!_schedule.IsScheduled((int)MoonEvent.ScanlineBoundary))
            {
                throw new InvalidOperationException("RunFrame() called with no scanline boundary scheduled - see ResetSchedule().");
            }

            // Let the instruction we stopped in front of run before re-arming, or `continue` re-halts on it.
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;

            _frameComplete = false;
            Ppu.SkipRendering = SkipRendering;

            while (!_frameComplete)
            {
                long deadline = _schedule.NextEventTime();
                _cpuBudget += _cpuClock.Advance(deadline - _schedule.Now, CpuRatio);

                if (!RunCpuUntilBudgetSpent(ref resuming)) return;

                _schedule.RunUntil(deadline);
            }
        }

        // Returns false when a breakpoint halted the frame partway through.
        private bool RunCpuUntilBudgetSpent(ref bool resuming)
        {
            while (_cpuBudget > 0)
            {
                _cpuBudget -= Bus!.TakePendingDmaCycles();
                if (_cpuBudget <= 0) break;

                Cpu!.SetNmiLine(Ppu!.NmiOutput);
                Cpu.SetIrqLine(Apu!.FrameIrqPending || Cart!.Mapper.IrqPending);

                if (!resuming && Breakpoints.ShouldBreak(Cpu.PC))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = Cpu.PC;
                    return false;
                }

                resuming = false;
                if (Coverage.IsArmed) Coverage.Record(Cpu.PC);

                _cpuBudget -= Cpu.Step();
            }

            return true;
        }

        public byte[] GetFrameBufferRgba() => Ppu?.FrameRgba ?? new byte[ScreenWidth * ScreenHeight * 4];

        // No synthesis yet, so there is never anything to drain - see Moon_APU.md.
        public short[] DequeueAudioSamples(int maxFrames) => Array.Empty<short>();

        public void SetButton(int port, NesButton button, bool pressed)
        {
            var controller = port == 0 ? Bus?.Controller1 : Bus?.Controller2;
            controller?.SetButton(button, pressed);
        }

        public void SaveSram() => Cart?.SaveSram();

        public void SaveState(string path)
        {
            using var stream = File.Create(path);
            SaveState(stream);
        }

        public void LoadState(string path)
        {
            using var stream = File.OpenRead(path);
            LoadState(stream);
        }

        public void SaveState(Stream stream)
        {
            if (Cart is null || Cpu is null || Bus is null || Ppu is null || Apu is null)
            {
                throw new InvalidOperationException("SaveState() called before LoadRom().");
            }

            using var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            w.Write(StateMagic);
            w.Write(StateVersion);
            w.Write(TotalFrames);
            w.Write(_currentScanline);
            w.Write(_lineStartClock);
            w.Write(_cpuBudget);

            Span<long> timeline = stackalloc long[_schedule.StateLength];
            _schedule.CaptureState(timeline);
            foreach (long value in timeline) w.Write(value);

            StateSerializer.Write(w, Cart);
            StateSerializer.Write(w, Cart.Mapper);
            StateSerializer.Write(w, Cpu);
            StateSerializer.Write(w, Bus);
            StateSerializer.Write(w, Ppu);
            StateSerializer.Write(w, Apu);
        }

        public void LoadState(Stream stream)
        {
            if (Cart is null || Cpu is null || Bus is null || Ppu is null || Apu is null)
            {
                throw new InvalidOperationException("LoadState() called before LoadRom().");
            }

            using var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            uint magic = r.ReadUInt32();
            int version = r.ReadInt32();
            if (magic != StateMagic)
            {
                throw new InvalidDataException("Not a Moon save state.");
            }
            if (version != StateVersion)
            {
                throw new InvalidDataException($"Save state version {version} is not {StateVersion}.");
            }

            TotalFrames = r.ReadInt64();
            _currentScanline = r.ReadInt32();
            _lineStartClock = r.ReadInt64();
            _cpuBudget = r.ReadInt64();

            Span<long> timeline = stackalloc long[_schedule.StateLength];
            for (int i = 0; i < timeline.Length; i++) timeline[i] = r.ReadInt64();
            _schedule.RestoreState(timeline);
            _schedule.SetHandler(this);

            StateSerializer.Read(r, Cart);
            StateSerializer.Read(r, Cart.Mapper);
            StateSerializer.Read(r, Cpu);
            StateSerializer.Read(r, Bus);
            StateSerializer.Read(r, Ppu);
            StateSerializer.Read(r, Apu);
        }
    }
}
