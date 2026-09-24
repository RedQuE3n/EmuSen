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
    public sealed partial class MercuryCore : global::EmuSen.Cores.ICore, global::EmuSen.Cores.ICheatRegistryHost, global::EmuSen.Cores.IStateFormat, global::EmuSen.Cores.ICoreSettings
    {
        public const int CpuClockHz = 4194304;

        // 154 scanlines of 456 T-cycles, vblank included - see Mercury_Core.md §2.
        public const int CyclesPerFrame = 70224;

        // "MERC" little-endian, then the format version - see EmuSen_Save_States.md §3.
        private const uint StateMagic = 0x4352454D;

        // 2 PPU, 3 colour banks and HDMA, 4 APU, 5 serial, 6 no save path or cartridge copies, 7 the console - see EmuSen_Save_States.md §7.
        private const int StateVersion = 7;

        // Version 6 dropped the save path and the cartridge copies; version 7 names the console before the walks - see Mercury_Model.md §5.
        private const int RetiredCartCopies = 6;
        private const int ConsoleInHeader = 7;

        // The oldest version LoadState still reads, its retired fields walked and dropped - see Mercury_Native.md §9.3.
        private const int OldestReadableVersion = 5;
        int global::EmuSen.Cores.IStateFormat.StateVersion => StateVersion;

        private const int SaveEveryNFrames = 300;

        public Cartridge? Cart { get; private set; }
        public Cpu.Core.Cpu? Cpu { get; private set; }
        public MemoryBus? Bus { get; private set; }

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
                if (Bus is not null) Bus.RomPatcher = new global::EmuSen.Cores.CheatRomPatcher(value);
            }
        }
        public CoverageRegistry Coverage { get; } = new();
        public LabelRegistry Labels { get; } = new();
        public CallStackRegistry CallStack { get; } = new();

        // Kept here and handed to each new bus, so a watch outlives a reload - see Mercury_Debug.md §7.
        private Memory.IWriteObserver? _writeObserver;

        public Memory.IWriteObserver? WriteObserver
        {
            get => _writeObserver;
            set
            {
                _writeObserver = value;
                if (Bus is not null) Bus.WriteObserver = value;
            }
        }

        public MercuryCore()
        {
            Breakpoints.CallStack = CallStack;
            CallStack.FrameNumberProvider = () => TotalFrames;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

        // Handed out before a ROM exists; once one does, the PPU's own buffer is returned instead.
        private readonly byte[] _frame = new byte[ScreenWidthPixels * ScreenHeightPixels * 4];

        private long _cyclesIntoFrame;
        private bool _skipRendering;

        public const int ScreenWidthPixels = 160;
        public const int ScreenHeightPixels = 144;

        // The console running, not the cartridge's flag: the Model setting can put either on either - see Mercury_Model.md §5.
        public string CoreName => Bus?.CgbHardware == true ? "GBC" : "GB";

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

        // Fixed for the session, and told to the APU at load rather than read back from it - see Mercury_Core.md §4.
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
            Bus = null;
            Build(ConsoleFor(Model, Cart.Cgb));
            CallStack.Reset();

            TotalFrames = 0;
            _cyclesIntoFrame = 0;
            IsHaltedAtBreakpoint = false;
        }

        // The machine a load builds on one console; a rebuild for a state keeps the host's side of the old one - see Mercury_Model.md §5.
        private void Build(bool cgbHardware)
        {
            MemoryBus? old = Bus;
            Bus = new MemoryBus(Cart!, cgbHardware) { RomPatcher = new global::EmuSen.Cores.CheatRomPatcher(Cheats), WriteObserver = _writeObserver };
            Cpu = new Cpu.Core.Cpu(Bus)
            {
                CallObserver = (source, target) => CallStack.NotePush(source, target, CallFrameKind.Call),
                ReturnObserver = CallStack.NotePop,
                InterruptObserver = (source, target) =>
                {
                    CallStack.NotePush(source, target, CallFrameKind.Irq);
                    Breakpoints.NoteInterrupt(CallFrameKind.Irq);
                },
            };

            Bus.Reset();
            if (Bus.DmgCompat) Cpu.ResetForCompatibility(Video.CompatibilityPalettes.HandOffChecksum(Cart!.Rom));
            else Cpu.Reset(Bus.Cgb);
            Bus.Ppu.SkipRendering = _skipRendering;
            Bus.Apu.SetSampleRate(AudioSampleRate);

            if (old is null) return;
            Bus.Joypad = old.Joypad;
            Bus.Apu.MaxBufferedSamples = old.Apu.MaxBufferedSamples;
            for (int i = 0; i < Audio.Apu.ChannelCount; i++) Bus.Apu.SetChannelMuted(i, old.Apu.IsChannelMuted(i));
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

            // Double speed spends twice the CPU cycles on the same frame, so the watchdog has to double too.
            long budget = Bus.DoubleSpeed ? CyclesPerFrame * 2 : CyclesPerFrame;

            while (_cyclesIntoFrame < budget)
            {
                if (!resuming && Breakpoints.ShouldBreak(Cpu.PC))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = Cpu.PC;
                    return;
                }

                resuming = false;
                if (Coverage.IsArmed) Coverage.Record(Cpu.PC);
                if (CallStack.IsProfiling) CallStack.NoteInstruction();

                // Step ticks the bus itself, one machine cycle at a time - see Mercury_Cpu.md §3.
                int cycles = Cpu.Step(Bus.InterruptEnable, Bus.InterruptFlags, out int serviced);
                if (serviced >= 0) Bus.InterruptFlags &= (byte)~(1 << serviced);

                // A general-purpose HDMA takes the bus away from the CPU - see Mercury_Cgb.md §4.1.
                int stall = Bus.TakePendingStall();
                if (stall > 0) Bus.Tick(stall);

                _cyclesIntoFrame += cycles + stall;

                // The PPU is the clock a frame actually ends on; the budget below only covers an off LCD.
                if (Bus.Ppu.FrameComplete)
                {
                    Bus.Ppu.FrameComplete = false;
                    _cyclesIntoFrame = 0;
                    EndFrame();
                    return;
                }
            }

            _cyclesIntoFrame -= budget;
            EndFrame();
        }

        private void EndFrame()
        {
            TotalFrames++;

            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            ApplyCheats();
            Breakpoints.NoteFrame(TotalFrames);

            if (TotalFrames % SaveEveryNFrames == 0) Cart!.SaveSram();
        }

        // Public so a paused frontend need not wait for a frame boundary - see EmuSen_Cheats.md §6.
        public void ApplyCheats() => Cheats.ApplyAll(ReadForCheat, WriteForCheat);

        public byte[] GetFrameBufferRgba() => Bus?.Ppu.FrameRgba ?? _frame;

        public short[] DequeueAudioSamples(int maxFrames) => Bus?.Apu.Drain(maxFrames) ?? Array.Empty<short>();

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
            w.Write(Bus.CgbHardware);
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
            if (version is < OldestReadableVersion or > StateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({OldestReadableVersion} to {StateVersion}).");
            bool retired = version < RetiredCartCopies;

            // Older states were made on the console the header chose; a state from the other console rebuilds the machine as that one.
            bool cgbHardware = version >= ConsoleInHeader ? r.ReadBoolean() : Cart.Cgb != CgbSupport.None;
            if (cgbHardware != Bus.CgbHardware) Build(cgbHardware);

            TotalFrames = r.ReadInt64();
            _cyclesIntoFrame = r.ReadInt64();

            StateSerializer.Read(r, Cart, includeRetired: retired);
            StateSerializer.Read(r, Cart.Mapper, includeRetired: retired);
            StateSerializer.Read(r, Cpu, includeRetired: retired);
            StateSerializer.Read(r, Bus, includeRetired: retired);
        }
    }
}
