using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // MercuryRT behind the Game Boy's ICore, on the common native host: the machine in Rust, the registries, saves and cheats' rules in C# - see Mercury_Native.md §8.7.
    public sealed class MercuryRtCore : NativeRtCore<MercuryMachine>, ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, ICoreSettings, IDisposable
    {
        private static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        private readonly NativeDebugBridge _debug;
        private GbModel _model;
        private string? _romPath;

        // The debugger's bridge over the mirror's registries, its pushes stamped with the frame being drained - see Mercury_Native.md §8.5.
        public MercuryRtCore() : base(MercuryMachine.FrameBytes, ports: 1)
        {
            _debug = new NativeDebugBridge(() => _machine, Mirror.Breakpoints, Mirror.Watches, Mirror.CallStack,
                new[] { Mirror.Coverage }, new[] { 0x10000 / 8 }, ReportedName, ReportedSpace);
            Mirror.CallStack.FrameNumberProvider = () => _debug.EventFrame;
            DebugBridge = _debug;
        }

        // The C# machine the debugger reads, refreshed from MercuryRT's state; its registries are this core's - see Mercury_Native.md §8.3.
        public MercuryCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override IReadOnlyList<string> SpaceNames => MercuryMachine.SpaceNames;
        protected override int CpuBusSpace => 6;
        protected override long PatchLow => 0;
        protected override long PatchHigh => 0x7FFF;

        public static bool Available => MercuryMachine.Complete;

        // The console running, as MercuryCore names it; a state from the other console changes it - see Mercury_Model.md §5.
        public override string CoreName => _machine?.CgbHardware == true ? "GBC" : "GB";
        public override int ScreenWidth => MercuryCore.ScreenWidthPixels;
        public override int ScreenHeight => MercuryCore.ScreenHeightPixels;
        public override double FrameRateHz => MercuryCore.CpuClockHz / (double)MercuryCore.CyclesPerFrame;
        public override IReadOnlyList<PadButton> SupportedButtons => MercuryCore.PadButtons;
        public override int StateVersion => MercuryCore.StateVersion;

        // MercuryCore.Model: the console the next load builds, and a change before the first frame loads the game again at once.
        public GbModel Model
        {
            get => _model;
            set
            {
                if (value == _model) return;
                _model = value;
                if (_machine is not null && TotalFrames == 0 && _romPath is { Length: > 0 } path) LoadRom(path);
            }
        }

        public IReadOnlyList<CoreSetting> Settings => MercuryCore.ModelSettings;

        public string Get(string key) => key == MercuryCore.ModelKey ? MercuryCore.ModelName(Model) : throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));

        public void Set(string key, string value)
        {
            if (key != MercuryCore.ModelKey) throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));
            Model = MercuryCore.ParseModel(value);
        }

        public WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;
        public CallStackRegistry CallStack => Mirror.CallStack;

        public NativeDebugBridge Debug => _debug;

        // The instruction the processor is about to run, live, as the registry compares it.
        public int Pc => _debug.ProgramCounter();

        // MercuryRT's own depth, which the registry's pushes it and its runs move; for tests.
        public long NativeDepth => _machine is null ? 0 : _debug.Counters().Depth;

        internal void Listen() => _debug.Listen();

        // MercuryDebugTarget.OnWrite's two listeners: a store is reported while a watch or a data breakpoint exists.
        public bool Listening => _debug.Listening;

        // The spaces the core logs a store under, by the C ABI's number: VRAM to HRAM, as C#'s bus reports them.
        private static string? ReportedName(uint id) => id is >= 1 and <= 5 ? MercuryMachine.SpaceNames[id] : null;

        private static uint? ReportedSpace(string name)
        {
            for (uint space = 1; space <= 5; space++)
                if (string.Equals(name, MercuryMachine.SpaceNames[space], StringComparison.OrdinalIgnoreCase)) return space;
            return null;
        }

        // MercuryCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save opened as C# opens it.
        protected override (BatterySave Battery, string Settings) Prepare(string path, byte[] image)
        {
            Cartridge header = Cartridge.FromImage(image);
            return (BatterySave.Open(path, BatterySave.GameBoyFolder(path), hasRam: header.HasBattery && header.Ram.Length > 0), "");
        }

        protected override MercuryMachine CreateMachine(byte[] image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files) =>
            new(image, _model, files.Count > 0 ? files[0].Data : null);

        protected override void Loaded(string path)
        {
            _romPath = path;
            Mirror.LoadRom(path);
            Mirror.Model = _model;
        }

        protected override int ButtonBit(PadButton button) => Array.IndexOf(MaskOrder, button);

        // Pad 1 alone; another port is ignored, as MercuryCore.SetButton ignores it.
        protected override int PortFor(int port) => port == 0 ? 0 : -1;

        // MercuryCore.LoadState's two refusals with its messages.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 4) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != MercuryCore.StateMagic) throw new InvalidDataException("Not a Mercury save state.");
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < MercuryCore.OldestReadableVersion or > MercuryCore.StateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({MercuryCore.OldestReadableVersion} to {MercuryCore.StateVersion}).");
        }

        // A store to the bus while a debugger listens goes through the observed path, as C#'s bus reports it.
        public override void WriteSpace(string spaceName, int address, byte value)
        {
            if (_machine is not null && SpaceNumber(spaceName) == CpuBusSpace && Listening)
            {
                _debug.Observed(() => base.WriteSpace(spaceName, address, value));
                return;
            }
            base.WriteSpace(spaceName, address, value);
        }

        // The debugger's view: the mirror loaded from MercuryRT's state, its reads and writes sent to MercuryRT, its halts this core's - see Mercury_Native.md §8.5.
        public MercuryRtDebugTarget CreateDebugTarget() => new(this);

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 4; i++) if (Mirror.Bus!.Apu.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
