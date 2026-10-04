using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Moon.Input;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.Moon.Processor;
using EmuSen.Cores.Nintendo.Moon.Video;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.Moon
{
    // The NES's ICore implementation; the hardware is public beyond the interface - see Moon_Core.md §1.
    public partial class MoonCore : global::EmuSen.Cores.ICore, global::EmuSen.Cores.IFrameProfiler, global::EmuSen.Cores.ICheatRegistryHost, global::EmuSen.Cores.IStateFormat
    {
        public const int MasterClockHz = 21477272;
        public const int MasterClocksPerCpuCycle = 12;
        public const int MasterClocksPerDot = 4;
        public const int DotsPerScanline = 341;
        public const int MasterClocksPerScanline = DotsPerScanline * MasterClocksPerDot;


        // "MOON" little-endian, then the format version - see EmuSen_Save_States.md §3.
        public const uint StateMagic = 0x4E4F4F4D;
        // 3 when the timeline folded into the core (Moon_Core.md §5); 4 the DMA's tail (Moon_Native.md §3.9).
        public const int StateVersion = 5;

        // Version 4 adds the DMA's tail after version 3's walks, version 5 the mixer after it; versions 3 and 4 still load - see Moon_Native.md §3.9, §3.13.
        public const int OldestReadableVersion = 3;
        int global::EmuSen.Cores.IStateFormat.StateVersion => StateVersion;

        public Cartridge? Cart { get; private set; }
        public Cpu? Cpu { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Ppu? Ppu { get; private set; }
        public Apu.Apu? Apu { get; private set; }

        // Owned here rather than in the debug target so a label set outlives any one prompt session.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();
        private CheatRegistry _cheats = new();

        // Settable so the registry a frontend already fills becomes this core's own - see EmuSen_Cheats.md §6.
        public CheatRegistry Cheats
        {
            get => _cheats;
            set
            {
                _cheats = value;
                if (Bus is not null) Bus.RomPatcher = new CheatRomPatcher(value);
            }
        }
        public CoverageRegistry Coverage { get; } = new();
        public LabelRegistry Labels { get; } = new();
        public CallStackRegistry CallStack { get; } = new();

        // The call stack serves step over and out, stamps its pushes with the frame, and shows coverage its entry points - see Moon_Debug.md §7.
        public MoonCore()
        {
            Breakpoints.CallStack = CallStack;
            CallStack.FrameNumberProvider = () => TotalFrames;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

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

        // Wall-clock cost of the last frame, by phase - see Moon_Debug.md §3.2.
        public double LastFrameCpuApuMs { get; private set; }
        public double LastFramePpuMs { get; private set; }

        // The APU is clocked from inside the CPU loop, so the two cannot be separated - see Moon_Debug.md §3.2.
        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => new[]
        {
            ("cpu+apu", LastFrameCpuApuMs),
            ("ppu", LastFramePpuMs),
        };

        // Halted in front of a breakpoint, with the frame left mid-flight for the next call to resume.
        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        public void LoadRom(string path)
        {
            Cart = Cartridge.Load(path);
            Ppu = new Ppu(Cart);
            Apu = new Apu.Apu();
            Apu.SetSampleRate(AudioSampleRate);
            // The core owns Cheats here, so ROM patches work with no debug target attached - see Moon_Cheats.md §3.
            Bus = new MemoryBus(Cart, Ppu, Apu) { WriteObserver = null, RomPatcher = new CheatRomPatcher(Cheats) };

            // DMC fetches go through the real bus, so the APU only gets a reader once one exists.

            Cpu = new Cpu(Bus)
            {
                CallObserver = (source, target) => CallStack.NotePush(source, target, CallFrameKind.Call),
                ReturnObserver = CallStack.NotePop,
                InterruptObserver = (source, target, kind) =>
                {
                    CallStack.NotePush(source, target, kind);
                    Breakpoints.NoteInterrupt(kind);
                },
            };
            Bus.Cpu = Cpu;
            CallStack.Reset();

            Ppu.Reset();
            Apu.Reset();
            Bus.Reset();
            Cpu.Reset();

            TotalFrames = 0;
            IsHaltedAtBreakpoint = false;
            ResetSchedule();
        }

        // The RESET button: the processors restart, RAM and PRG RAM keep what they held - see Moon_Core.md §6.
        public void Reset()
        {
            if (Bus is null || Cpu is null || Ppu is null || Apu is null)
            {
                throw new InvalidOperationException("Reset() called before LoadRom().");
            }

            Ppu.SoftReset();
            Apu.SoftReset();
            Bus.SoftReset();
            Cpu.SoftReset();

            IsHaltedAtBreakpoint = false;
            ResetSchedule();
        }

        public void RunFrame()
        {
            if (Bus is null || Cpu is null || Ppu is null || Apu is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            // Let the instruction we stopped in front of run before re-arming, or `continue` re-halts on it.
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;

            _frameComplete = false;
            Ppu.SkipRendering = SkipRendering;
            Bus.ApuTraceFrame = (uint)TotalFrames;

            // The halted scanline has had its cycles, so a resume must not earn them twice - see Moon_Debug.md §7.
            bool earned = resuming;
            while (!_frameComplete)
            {
                long deadline = NextScanlineBoundary;
                if (!earned) _cpuBudget += EarnCpuCycles(deadline - _masterClock);
                earned = false;

                // A halt returns without closing the phase, so the resumed frame keeps accumulating - see Moon_Debug.md §3.2.
                long phaseStart = Stopwatch.GetTimestamp();
                if (!RunCpuUntilBudgetSpent(ref resuming)) return;
                long afterCpu = Stopwatch.GetTimestamp();
                _cpuApuTicksAccum += afterCpu - phaseStart;

                EndScanline(deadline);
                _ppuTicksAccum += Stopwatch.GetTimestamp() - afterCpu;
            }
        }

        // Returns false when a breakpoint halted the frame partway through.
        private bool RunCpuUntilBudgetSpent(ref bool resuming)
        {
            while (_cpuBudget > 0)
            {
                _cpuBudget -= Bus!.TakePendingDmaCycles();
                if (_cpuBudget <= 0) break;

                // Both lines are refreshed per cycle by the bus now; this is the pre-instruction edge - see Moon_CPU.md §5.5.
                Cpu!.SetNmiLine(Ppu!.NmiOutput);

                if (!resuming && Breakpoints.ShouldBreak(Cpu.PC))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = Cpu.PC;
                    return false;
                }

                resuming = false;
                if (Coverage.IsArmed) Coverage.Record(Cpu.PC);
                if (CallStack.IsProfiling) CallStack.NoteInstruction();

                // The APU and PPU are clocked inside the CPU's own bus cycles now - see Moon_CPU.md §5.5.
                _cpuBudget -= Cpu.Step();
                _cpuBudget -= Bus.TakeStolenCycles();

                if (!Ppu.FrameComplete) continue;

                Ppu.FrameComplete = false;
                EndFrame();
                return true;
            }

            return true;
        }

        public byte[] GetFrameBufferRgba() => Ppu?.FrameRgba ?? new byte[ScreenWidth * ScreenHeight * 4];

        public short[] DequeueAudioSamples(int maxFrames) => Apu?.Drain(maxFrames) ?? Array.Empty<short>();

        // The two ports on the console's front; the Four Score is not modelled - see EmuSen_Input.md §8.1.
        public const int Ports = 2;

        public int ControllerPorts => Ports;

        public void SetButton(int port, NesButton button, bool pressed)
        {
            var controller = port switch { 0 => Bus?.Controller1, 1 => Bus?.Controller2, _ => null };
            controller?.SetButton(button, pressed);
        }

        // The eight the pad has; X/Y/L/R have no wire to reach - see EmuSen_Input.md §2.
        public static readonly PadButton[] PadButtons =
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.Select, PadButton.Start, PadButton.B, PadButton.A,
        };

        // Static, so the rebind window can list this console's pad without a ROM - see EmuSen_Input.md §5.1.
        public IReadOnlyList<PadButton> SupportedButtons => PadButtons;

        // A binding for a button this console lacks is ignored rather than mapped onto another one.
        public void SetButton(int port, PadButton button, bool pressed)
        {
            NesButton? mapped = button switch
            {
                PadButton.A => NesButton.A,
                PadButton.B => NesButton.B,
                PadButton.Select => NesButton.Select,
                PadButton.Start => NesButton.Start,
                PadButton.Up => NesButton.Up,
                PadButton.Down => NesButton.Down,
                PadButton.Left => NesButton.Left,
                PadButton.Right => NesButton.Right,
                _ => null,
            };

            if (mapped is { } nesButton) SetButton(port, nesButton, pressed);
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
            w.Write(_lineStartClock);
            w.Write(_cpuBudget);
            w.Write(_masterClock);
            w.Write(_cpuRemainder);

            StateSerializer.Write(w, Cart);
            StateSerializer.Write(w, Cart.Mapper);
            StateSerializer.Write(w, Cpu);
            StateSerializer.Write(w, Bus);
            StateSerializer.Write(w, Ppu);
            StateSerializer.Write(w, Apu);

            w.Write(Apu.Dmc.SampleBuffer);
            w.Write(Apu.Dmc.BufferFull);
            w.Write(Apu.Dmc.LoadDelay);
            w.Write(Bus.OamDmaPending);
            w.Write(Bus.OamDmaPage);
            Apu.WriteMixer(w);
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
            if (version is < OldestReadableVersion or > StateVersion)
            {
                throw new InvalidDataException($"Save state version {version} is not one this build reads ({OldestReadableVersion} to {StateVersion}).");
            }

            TotalFrames = r.ReadInt64();
            _lineStartClock = r.ReadInt64();
            _cpuBudget = r.ReadInt64();
            _masterClock = r.ReadInt64();
            _cpuRemainder = r.ReadInt64();

            StateSerializer.Read(r, Cart);
            StateSerializer.Read(r, Cart.Mapper);
            StateSerializer.Read(r, Cpu);
            StateSerializer.Read(r, Bus);
            StateSerializer.Read(r, Ppu);
            StateSerializer.Read(r, Apu);

            // Version 3 had no sample buffer and no pending OAM DMA: the reader asks for its byte at once.
            bool tail = version >= 4;
            Apu.Dmc.SampleBuffer = tail ? r.ReadByte() : (byte)0;
            Apu.Dmc.BufferFull = tail && r.ReadBoolean();
            Apu.Dmc.LoadDelay = tail ? r.ReadInt32() : 0;
            Bus.OamDmaPending = tail && r.ReadBoolean();
            Bus.OamDmaPage = tail ? r.ReadByte() : (byte)0;
            // Before version 5 the mixer was not in the state: it keeps what it held, as it did then.
            if (version >= 5) Apu.ReadMixer(r);
            Bus.ForgetLastRead();
            Bus.InternalBus = Bus.OpenBus;
            Apu.FrameIrqReadable = Apu.FrameIrqPending;
            Apu.EndLengthCycle();
        }
    }
}
