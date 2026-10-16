using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // Which console a cartridge runs on: the one its header asks for, or the one the player chose - see Mercury_Model.md §1.
    public enum GbModel
    {
        Auto,
        GameBoy,
        GameBoyColor,
    }

    // MercuryRT behind the Game Boy's ICore: the generic v1 adapter, with the Game Boy's exceptions, state checks, battery rule, Model and debugger registries - see Mercury_Native.md §8.7, §15, EmuSen_CoreAPI.md §26.
    public sealed unsafe class MercuryRtCore : PortEngine
    {
        private static readonly PadButton[] MaskOrder = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public const string SampleRateExport = "mercuryrt_core_set_sample_rate";

        public const int CpuClockHz = 4194304;

        // 154 scanlines of 456 T-cycles, vblank included - see Mercury_Core.md §2.
        public const int CyclesPerFrame = 70224;

        public const int ScreenWidthPixels = 160;
        public const int ScreenHeightPixels = 144;

        // "MERC" little-endian, then the format version; the oldest version a load still reads - see Mercury_Native.md §9.3, §10, §13.
        public const uint StateMagic = 0x4352454D;
        public const int StateFormatVersion = 11;
        public const int OldestReadableVersion = 5;

        // What a real console's output capacitor does to the DAC's standing offset, raised to the cycles per sample here - see Mercury_Apu.md §5.2.
        public const double HighPassSeed = 0.999958;

        // The spaces, numbered as the core's ABI numbers them.
        public const string SpaceRom = "ROM", SpaceVram = "VRAM", SpaceCartRam = "CARTRAM", SpaceWram = "WRAM", SpaceOam = "OAM", SpaceHram = "HRAM", SpaceCpuBus = "CPUBUS";
        public static readonly string[] SpaceNameList = { SpaceRom, SpaceVram, SpaceCartRam, SpaceWram, SpaceOam, SpaceHram, SpaceCpuBus };

        // Read by CoreCatalog too, so a rebind UI cannot drift from what the core reads.
        public static IReadOnlyList<PadButton> PadButtons { get; } = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.B, PadButton.Select, PadButton.Start,
        };

        // The Model setting, honoured at the next load - see Mercury_Model.md §1 and EmuSen_Settings_Reference.md §4.47.
        public const string ModelKey = "Model";
        public const string ModelAuto = "Auto";
        public const string ModelGameBoy = "Game Boy";
        public const string ModelGameBoyColor = "Game Boy Color";

        public static readonly IReadOnlyList<CoreSetting> ModelSettings = new CoreSetting[]
        {
            new(ModelKey, "Model",
                "Which console a game runs on. Auto follows the cartridge: a Game Boy Color game on a Game Boy Color, the rest on a Game Boy. Game Boy runs every game on a Game Boy: a colour game in its own monochrome mode, and a game made only for the Color shows whatever that game shows a Game Boy. Game Boy Color runs every game on a Game Boy Color: a Game Boy game in the colours the Color's start-up picks for it from its title. Takes effect when a game is next loaded; a save state resumes on the console it was made on.",
                CoreSettingKind.Choice, ModelAuto, Choices: new[] { ModelAuto, ModelGameBoy, ModelGameBoyColor }),
        };

        public static GbModel ParseModel(string text) => text switch
        {
            ModelAuto => GbModel.Auto,
            ModelGameBoy => GbModel.GameBoy,
            ModelGameBoyColor => GbModel.GameBoyColor,
            _ => throw new ArgumentException($"{text} is not a Game Boy console."),
        };

        public static string ModelName(GbModel model) => model switch
        {
            GbModel.GameBoy => ModelGameBoy,
            GbModel.GameBoyColor => ModelGameBoyColor,
            _ => ModelAuto,
        };

        private GbModel _model;
        private string? _romPath;
        private byte _cartridgeType;
        private uint _mutes;
        private CheatRegistry _cheats = new();
        private MercuryStateLayout? _layout;

        public MercuryRtCore() : base(MercuryNative.Engine, "MercuryRT", "Mercury", pads: 1)
        {
            Breakpoints.CallStack = CallStack;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

        // The debugger's registries, owned here so a label set or a coverage map outlives any one prompt session.
        public CoverageRegistry Coverage { get; } = new();
        public LabelRegistry Labels { get; } = new();
        public CallStackRegistry CallStack { get; } = new();

        protected override CheatRegistry MirrorCheats { get => _cheats; set => _cheats = value; }
        protected override CallStackRegistry MirrorCallStack => CallStack;
        protected override CoverageRegistry MirrorCoverage => Coverage;
        protected override IReadOnlyList<string> SpaceNames => SpaceNameList;
        protected override int CpuBusSpace => 6;

        public static bool Available => MercuryNative.Engine.Available;

        // The header of the game loaded, read before the library saw it.
        public GbCartridge Cartridge { get; private set; }

        // The console running; a state from the other console changes it, which machine info follows - see Mercury_Model.md §5.
        public override string CoreName => IsRomLoaded && Machine.Core.Info.System == "gbc" ? "GBC" : "GB";
        public override int ScreenWidth => ScreenWidthPixels;
        public override int ScreenHeight => ScreenHeightPixels;
        public override double FrameRateHz => CpuClockHz / (double)CyclesPerFrame;

        // The LCD's 160 by 144 square pixels, a game loaded or not - see EmuSen_Serenity.md §2.9.
        public override double DisplayAspect => EmuSen.Serenity.DisplayShape.GameBoy;
        public override int StateVersion => StateFormatVersion;

        // The console the next load builds, sent as the create-time setting Model; a change before the first frame loads the game again at once.
        public GbModel Model
        {
            get => _model;
            set
            {
                if (value == _model) return;
                _model = value;
                base.Set(ModelKey, ModelName(value));
                if (IsRomLoaded && TotalFrames == 0 && _romPath is { Length: > 0 } path) LoadRom(path);
            }
        }

        public override IReadOnlyList<CoreSetting> Settings => ModelSettings;

        public override string Get(string key) => key == ModelKey ? ModelName(Model) : throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));

        public override void Set(string key, string value)
        {
            if (key != ModelKey) throw new ArgumentException($"Mercury has no setting named {key}.", nameof(key));
            Model = ParseModel(value);
        }

        // MercuryRT's own depth, which the registry's pushes it and its runs move; for tests.
        public long NativeDepth => IsRomLoaded ? Debug.Counters().Depth : 0;

        internal void Listen() => Debug.Listen();

        // A store is reported while a watch or a data breakpoint exists.
        public bool Listening => Debug.Listening;

        // The spaces the core logs a store under, by the ABI's number: VRAM to HRAM.
        protected override string? ReportedName(uint id) => id is >= 1 and <= 5 ? SpaceNameList[id] : null;

        protected override uint? ReportedSpace(string name)
        {
            for (uint space = 1; space <= 5; space++)
                if (string.Equals(name, SpaceNameList[space], StringComparison.OrdinalIgnoreCase)) return space;
            return null;
        }

        // The header's refusals before the library sees the image, and the battery file a board with battery-backed RAM keeps.
        protected override BatterySave Prepare(string path, byte[] image)
        {
            _cartridgeType = image.Length > 0x147 ? image[0x147] : (byte)0;
            GbCartridge header = GbCartridge.Read(image);
            Cartridge = header;
            return BatterySave.Open(path, BatterySave.GameBoyFolder(path), hasRam: header.HasBattery && header.RamLength > 0);
        }

        // The mixer's coefficients, the division and Math.Pow both evaluated here and sent down - see Mercury_Native.md §3.3 and §9.1.
        protected override void Loaded(string path)
        {
            double cyclesPerSample = 4194304.0 / 44100;
            lock (Machine.Core.Gate)
                ((delegate* unmanaged<nint, double, double, int>)MercuryNative.Engine.Export(SampleRateExport))(Machine.Core.Handle, cyclesPerSample, Math.Pow(HighPassSeed, cyclesPerSample));
            _romPath = path;
            _mutes = 0;
            _layout = null;
            CallStack.Reset();
        }

        protected override int ButtonBit(PadButton button) => Array.IndexOf(MaskOrder, button);

        // Pad 1 alone; another port is ignored.
        protected override int PortFor(int port) => port == 0 ? 0 : -1;

        public int ControllerPorts => 1;

        // The format's two refusals with their messages, before the bytes reach the machine.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 4) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != StateMagic) throw new InvalidDataException("Not a Mercury save state.");
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < OldestReadableVersion or > StateFormatVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({OldestReadableVersion} to {StateFormatVersion}).");
        }

        // The load's exceptions with the cartridge type the header named, and an illegal opcode built from the frame's detail word, (opcode << 16) | pc.
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

        // The debugger's view, read from MercuryRT's state through its own layout listing - see Mercury_Native.md §15.
        public new MercuryRtDebugTarget CreateDebugTarget() => new(this);

        // The machine's state now, and the listing that names its bytes; a load from the other console changes the state's length, and the listing is asked for again.
        public (byte[] State, MercuryStateLayout Layout) Snapshot()
        {
            byte[] state = Machine.Save();
            if (_layout is not { } layout || layout.Length != state.Length)
                _layout = layout = MercuryStateLayout.FromText("MercuryRT", Machine.Core.Layout());
            return (state, layout);
        }

        // A sound channel silenced for the debugger, sent to the machine at once; a load clears them all.
        public void SetChannelMuted(int channel, bool muted)
        {
            if ((uint)channel >= 4) return;
            _mutes = muted ? _mutes | (1u << channel) : _mutes & ~(1u << channel);
            SyncMutes();
        }

        public bool IsChannelMuted(int channel) => (uint)channel < 4 && (_mutes & (1u << channel)) != 0;

        protected override uint MuteMask() => _mutes;
    }
}
