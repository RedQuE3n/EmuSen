using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Core;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.Mercury
{
    // The Game Boy's ICore implementation; colour is an additive mode on this same core - see Mercury_Core.md §1.
    public sealed partial class MercuryCore : global::EmuSen.Cores.ICore
    {
        public const int CpuClockHz = 4194304;

        // 154 scanlines of 456 T-cycles, vblank included - see Mercury_Core.md §2.
        public const int CyclesPerFrame = 70224;

        // "MERC" little-endian, then the format version - see EmuSen_Save_States.md §3.
        private const uint StateMagic = 0x4352454D;

        // 2 added the PPU's fields to the bus walk - see EmuSen_Save_States.md §1.
        private const int StateVersion = 2;

        private const int SaveEveryNFrames = 300;

        public Cartridge? Cart { get; private set; }
        public Cpu.Core.Cpu? Cpu { get; private set; }
        public MemoryBus? Bus { get; private set; }

        // Owned here rather than in the debug target so a label set outlives any one prompt session.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();
        public CheatRegistry Cheats { get; } = new();
        public CoverageRegistry Coverage { get; } = new();
        public LabelRegistry Labels { get; } = new();

        // Handed out before a ROM exists; once one does, the PPU's own buffer is returned instead.
        private readonly byte[] _frame = new byte[ScreenWidthPixels * ScreenHeightPixels * 4];

        private long _cyclesIntoFrame;
        private bool _skipRendering;

        public const int ScreenWidthPixels = 160;
        public const int ScreenHeightPixels = 144;

        public string CoreName => "GB";

        public int ScreenWidth => ScreenWidthPixels;
        public int ScreenHeight => ScreenHeightPixels;

        // 4194304 / 70224 ~= 59.7275 - see Mercury_Core.md §2.
        public double FrameRateHz => CpuClockHz / (double)CyclesPerFrame;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        public bool SkipRendering
        {
            get => _skipRendering;
            set
            {
                _skipRendering = value;
                if (Bus != null) Bus.Ppu.SkipRendering = value;
            }
        }

        // No synthesis yet, but a caller still needs a rate up front - see Mercury_Core.md §4.
        public int AudioSampleRate => 44100;

        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        // Read by CoreCatalog too, so a rebind UI cannot drift from what the core actually reads.
        public static IReadOnlyList<PadButton> PadButtons { get; } = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.B, PadButton.Select, PadButton.Start,
        };

        public IReadOnlyList<PadButton> SupportedButtons => PadButtons;

        public void LoadRom(string path)
        {
            Cart = Cartridge.Load(path);
            Bus = new MemoryBus(Cart);
            Cpu = new Cpu.Core.Cpu(Bus);

            Bus.Reset();
            Cpu.Reset();
            Bus.Ppu.SkipRendering = _skipRendering;

            TotalFrames = 0;
            _cyclesIntoFrame = 0;
            IsHaltedAtBreakpoint = false;
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            if (port != 0) return;
            Bus?.Joypad.Set(button, pressed);
        }

        public void RunFrame()
        {
            if (Bus is null || Cpu is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            // Let the instruction we stopped in front of run before re-arming, or `continue` re-halts on it.
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;

            while (_cyclesIntoFrame < CyclesPerFrame)
            {
                if (!resuming && Breakpoints.ShouldBreak(Cpu.PC))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = Cpu.PC;
                    return;
                }

                resuming = false;
                if (Coverage.IsArmed) Coverage.Record(Cpu.PC);

                int cycles = Cpu.Step(Bus.InterruptEnable, Bus.InterruptFlags, out int serviced);
                if (serviced >= 0) Bus.InterruptFlags &= (byte)~(1 << serviced);

                Bus.Tick(cycles);
                _cyclesIntoFrame += cycles;

                // The PPU is the clock a frame actually ends on; the budget below only covers an off LCD.
                if (Bus.Ppu.FrameComplete)
                {
                    Bus.Ppu.FrameComplete = false;
                    _cyclesIntoFrame = 0;
                    EndFrame();
                    return;
                }
            }

            _cyclesIntoFrame -= CyclesPerFrame;
            EndFrame();
        }

        private void EndFrame()
        {
            TotalFrames++;

            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            Cheats.ApplyAll(ReadForCheat, WriteForCheat);

            if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();
        }

        public byte[] GetFrameBufferRgba() => Bus?.Ppu.FrameRgba ?? _frame;

        public short[] DequeueAudioSamples(int maxFrames) => Array.Empty<short>();

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
            if (Cart is null || Cpu is null || Bus is null)
            {
                throw new InvalidOperationException("SaveState() called before LoadRom().");
            }

            using var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            w.Write(StateMagic);
            w.Write(StateVersion);
            w.Write(TotalFrames);
            w.Write(_cyclesIntoFrame);

            StateSerializer.Write(w, Cart);
            StateSerializer.Write(w, Cart.Mapper);
            StateSerializer.Write(w, Cpu);
            StateSerializer.Write(w, Bus);
        }

        public void LoadState(Stream stream)
        {
            if (Cart is null || Cpu is null || Bus is null)
            {
                throw new InvalidOperationException("LoadState() called before LoadRom().");
            }

            using var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);

            if (r.ReadUInt32() != StateMagic) throw new InvalidDataException("Not a Mercury save state.");

            int version = r.ReadInt32();
            if (version != StateVersion) throw new InvalidDataException($"Save state version {version} is not {StateVersion}.");

            TotalFrames = r.ReadInt64();
            _cyclesIntoFrame = r.ReadInt64();

            StateSerializer.Read(r, Cart);
            StateSerializer.Read(r, Cart.Mapper);
            StateSerializer.Read(r, Cpu);
            StateSerializer.Read(r, Bus);
        }
    }
}
