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
    // MercuryRT behind the Game Boy's ICore: the generic v1 adapter, with C# Mercury's exceptions, state checks, battery rule, Model and mirror debugger kept for its oracle - see Mercury_Native.md §8.7, EmuSen_CoreAPI.md §26.
    public sealed unsafe class MercuryRtCore : PortEngine
    {
        private static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public const string SampleRateExport = "mercuryrt_core_set_sample_rate";

        private GbModel _model;
        private string? _romPath;
        private byte _cartridgeType;

        public MercuryRtCore() : base(MercuryNative.Engine, "MercuryRT", "Mercury", pads: 1) { }

        // The C# machine the debugger reads, refreshed from MercuryRT's state; its registries are this core's - see Mercury_Native.md §8.3.
        public MercuryCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override CallStackRegistry MirrorCallStack => Mirror.CallStack;
        protected override CoverageRegistry MirrorCoverage => Mirror.Coverage;
        protected override IReadOnlyList<string> SpaceNames => MercuryMachine.SpaceNames;
        protected override int CpuBusSpace => 6;

        public static bool Available => MercuryNative.Engine.Available;

        // The console running, as MercuryCore names it; a state from the other console changes it, which machine info follows - see Mercury_Model.md §5.
        public override string CoreName => IsRomLoaded && Machine.Core.Info.System == "gbc" ? "GBC" : "GB";
        public override int ScreenWidth => MercuryCore.ScreenWidthPixels;
        public override int ScreenHeight => MercuryCore.ScreenHeightPixels;
        public override double FrameRateHz => MercuryCore.CpuClockHz / (double)MercuryCore.CyclesPerFrame;
        public override int StateVersion => MercuryCore.StateVersion;

        // MercuryCore.Model: the console the next load builds, sent as the create-time setting; a change before the first frame loads the game again at once.
        public GbModel Model
        {
            get => _model;
            set
            {
                if (value == _model) return;
                _model = value;
                base.Set(MercuryCore.ModelKey, MercuryCore.ModelName(value));
                if (IsRomLoaded && TotalFrames == 0 && _romPath is { Length: > 0 } path) LoadRom(path);
            }
        }

        public override IReadOnlyList<CoreSetting> Settings => MercuryCore.ModelSettings;

        public override string Get(string key) => key == MercuryCore.ModelKey ? MercuryCore.ModelName(Model) : throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));

        public override void Set(string key, string value)
        {
            if (key != MercuryCore.ModelKey) throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));
            Model = MercuryCore.ParseModel(value);
        }

        public override WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;
        public CallStackRegistry CallStack => Mirror.CallStack;

        // MercuryRT's own depth, which the registry's pushes it and its runs move; for tests.
        public long NativeDepth => IsRomLoaded ? Debug.Counters().Depth : 0;

        internal void Listen() => Debug.Listen();

        // MercuryDebugTarget.OnWrite's two listeners: a store is reported while a watch or a data breakpoint exists.
        public bool Listening => Debug.Listening;

        // The spaces the core logs a store under, by the ABI's number: VRAM to HRAM, as C#'s bus reports them.
        protected override string? ReportedName(uint id) => id is >= 1 and <= 5 ? MercuryMachine.SpaceNames[id] : null;

        protected override uint? ReportedSpace(string name)
        {
            for (uint space = 1; space <= 5; space++)
                if (string.Equals(name, MercuryMachine.SpaceNames[space], StringComparison.OrdinalIgnoreCase)) return space;
            return null;
        }

        // MercuryCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save opened as C# opens it.
        protected override BatterySave Prepare(string path, byte[] image)
        {
            _cartridgeType = image.Length > 0x147 ? image[0x147] : (byte)0;
            Cartridge header = Cartridge.FromImage(image);
            return BatterySave.Open(path, BatterySave.GameBoyFolder(path), hasRam: header.HasBattery && header.Ram.Length > 0);
        }

        // C#'s Apu.SetSampleRate, the division and Math.Pow both evaluated here and sent down, then the mirror - see Mercury_Native.md §3.3 and §9.1.
        protected override void Loaded(string path)
        {
            double cyclesPerSample = 4194304.0 / 44100;
            ((delegate* unmanaged<nint, double, double, int>)MercuryNative.Engine.Export(SampleRateExport))(Machine.Core.Handle, cyclesPerSample,
                Math.Pow(EmuSen.Cores.Nintendo.Mercury.Audio.Apu.HighPassSeed, cyclesPerSample));
            _romPath = path;
            Mirror.LoadRom(path);
            Mirror.Model = _model;
        }

        protected override int ButtonBit(PadButton button) => Array.IndexOf(MaskOrder, button);

        // Pad 1 alone; another port is ignored, as MercuryCore.SetButton ignores it.
        protected override int PortFor(int port) => port == 0 ? 0 : -1;

        public int ControllerPorts => 1;

        // MercuryCore.LoadState's two refusals with its messages.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 4) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != MercuryCore.StateMagic) throw new InvalidDataException("Not a Mercury save state.");
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < MercuryCore.OldestReadableVersion or > MercuryCore.StateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({MercuryCore.OldestReadableVersion} to {MercuryCore.StateVersion}).");
        }

        // MercuryCore.LoadRom's exceptions with the cartridge type the header named, and an illegal opcode built from the frame's detail word, (opcode << 16) | pc.
        protected override Exception? OwnRefusal(int status, ulong detail) => status switch
        {
            -10 => new NotSupportedException($"Cartridge type ${_cartridgeType:X2} is not implemented - see Mercury_Memory.md §4."),
            -9 or -11 => new InvalidDataException($"MercuryRT refused the image: {MercuryMachine.Describe(status)}."),
            -20 => MercuryMachine.IllegalOpcode((uint)detail),
            _ => null,
        };

        protected override string? OwnWords(long status) => StatusWords(status);

        internal static string? StatusWords(long status) => status switch
        {
            -9 => "an image shorter than the 336-byte header",
            -10 => "a cartridge type no board implements",
            -11 => "a model that is not Auto, Game Boy or Game Boy Color",
            -20 => "an opcode no SM83 has",
            _ => null,
        };

        // The debugger's view: the mirror loaded from MercuryRT's state, its reads and writes sent to MercuryRT, its halts this core's - see Mercury_Native.md §8.5.
        public new MercuryRtDebugTarget CreateDebugTarget() => new(this);

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 4; i++) if (Mirror.Bus!.Apu.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
