using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Library;

namespace EmuSen.WiseMan.Fixtures
{
    // MarsRT's machine-info space numbers, as its tests name them.
    public enum MarsRtSpace : uint { Rdram = 0, Dmem = 1, Imem = 2, PifRam = 3, Rom = 4, Cpu = 5 }

    // The MI's six interrupt sources by their bits in MI_INTR, for the test hook that raises them - see Mars_Memory.md §8.
    [Flags]
    public enum MiInterrupt
    {
        None = 0,
        SignalProcessor = 1 << 0,
        SerialInterface = 1 << 1,
        AudioInterface = 1 << 2,
        VideoInterface = 1 << 3,
        PeripheralInterface = 1 << 4,
        DisplayProcessor = 1 << 5,
    }

    // MarsRT for its tests: the discovered library, its space names, and the test hooks MarsRT keeps as marsrt_core_* extensions, reached by name and never by shipped code - see EmuSen_Backend.md §18.10.
    public static unsafe class MarsRt
    {
        public const string Rdram = "RDRAM", Dmem = "DMEM", Imem = "IMEM", PifRam = "PIFRAM", Rom = "ROM", Cpu = "CPU";

        // The VI's raster width, which the picture's width is a multiple of; the Controller Pak's file beside the cartridge's.
        public const int ScreenWidthPixels = 640;
        public const string PakExtension = ".mpk";

        public const uint ViUnit = 256;
        private const int StatusRegister = 12;

        // The library beside the tests by its fixed name, whatever directories discovery is pointed at.
        private static readonly Lazy<CoreLibrary?> Opened = new(() =>
        {
            string path = Path.Combine(AppContext.BaseDirectory, OperatingSystem.IsWindows() ? "marsrt.dll" : OperatingSystem.IsMacOS() ? "libmarsrt.dylib" : "libmarsrt.so");
            return File.Exists(path) ? CoreLibrary.Open(path) : null;
        });

        public static CoreLibrary? Library => Opened.Value is { Available: true } library ? library : null;

        public static bool Available => Library is not null;

        public static string Report => Opened.Value?.Report ?? "no MarsRT library beside the tests";

        public static nint Export(string name) => Library?.Export(name) ?? 0;

        // The generic target over an engine, handed the caller's cheat registry first, as the shim's target took one.
        public static CoreDebugTarget Target(CoreEngine engine, EmuSen.Crystal.Cheats.CheatRegistry? cheats)
        {
            if (cheats is not null) engine.Cheats = cheats;
            return engine.CreateDebugTarget();
        }

        // The rows the settings window shows for the N64, from the schema.
        public static IReadOnlyList<CoreSetting> VideoSettings => Library!.Settings.Where(s => !s.Hidden).Select(s => s.AsCoreSetting()).OfType<CoreSetting>().ToArray();

        private static readonly Lazy<nint> RunStepsHook = new(() => Export("marsrt_core_run_steps"));

        internal static void RunSteps(CoreMachine m, ulong steps) => ((delegate* unmanaged<nint, ulong, int>)RunStepsHook.Value)(m.Handle, steps);

        private static long[] CountersOf(CoreEngine engine, string prefix, string? also = null)
        {
            IReadOnlyList<CoreCounter> list = engine.Machine.Info.Telemetry;
            var values = new long[list.Count];
            engine.Telemetry(values);
            return list.Select((c, i) => (c, i)).Where(x => x.c.Name.StartsWith(prefix, StringComparison.Ordinal) || x.c.Name == also).Select(x => values[x.i]).ToArray();
        }

        extension(CoreEngine engine)
        {
            public long SpaceSize(MarsRtSpace space) => engine.IsRomLoaded ? engine.Machine.SpaceSize((uint)space) : 0;

            public byte Peek(MarsRtSpace space, uint address)
            {
                Span<byte> one = stackalloc byte[1];
                if (engine.IsRomLoaded) engine.Machine.ReadSpace((uint)space, (int)address, one);
                return one[0];
            }

            public void Poke(MarsRtSpace space, uint address, byte value)
            {
                if (engine.IsRomLoaded) engine.Machine.WriteSpace((uint)space, (int)address, stackalloc byte[] { value });
            }

            public byte[] ReadSpace(MarsRtSpace space, uint address, int length)
            {
                var bytes = new byte[length];
                if (engine.IsRomLoaded && length > 0) engine.Machine.ReadSpace((uint)space, (int)address, bytes);
                return bytes;
            }

            public ulong Cop0(int register) => engine.IsRomLoaded ? ((delegate* unmanaged<nint, uint, ulong>)Export("marsrt_core_cop0"))(engine.Machine.Handle, (uint)register) : 0;

            // PC, the 32 GPRs, HI, LO and the cycle count, through DEBUG_REGISTERS.
            public ulong[] CpuRegisters() => engine.IsRomLoaded ? engine.Machine.Registers(0).Select(v => (ulong)v).ToArray() : Array.Empty<ulong>();

            // PC, halted, and the 32 scalar registers.
            public uint[] RspRegisters() => engine.IsRomLoaded ? engine.Machine.Registers(1).Select(v => (uint)v).ToArray() : Array.Empty<uint>();

            public uint[] ViRegisters() => engine.IsRomLoaded ? engine.Machine.Registers(ViUnit).Select(v => (uint)v).ToArray() : Array.Empty<uint>();

            // A counter of machine info's telemetry by its name.
            public long Counter(string name)
            {
                IReadOnlyList<CoreCounter> list = engine.Machine.Info.Telemetry;
                int at = list.Select((c, i) => (c, i)).First(x => x.c.Name == name).i;
                Span<long> values = stackalloc long[list.Count];
                engine.Telemetry(values);
                return values[at];
            }

            public long Cycles => engine.Counter("cpu.cycles");

            public long Instructions => engine.Counter("cpu.instructions");

            public long IdleTurnsPassed => engine.Counter("cpu.idle.turns");

            // The display processor's thread's counters, in the order the shim's export gave them.
            public long[] ThreadCounterValues() => CountersOf(engine, "rdp.", "vi.repeated_scans");

            public long[] BlockCounterValues() => CountersOf(engine, "blocks.");

            // The call stack's depth and unmatched returns as v1's debug_counters gives them, and the exceptions entered.
            public long[] DebugCounterValues()
            {
                long* depth = stackalloc long[2];
                engine.Library.Api.DebugCounters(engine.Machine.Handle, depth, 2);
                return new[] { depth[0], depth[1], engine.Counter("cpu.exceptions") };
            }

            // What the device setting got: the device's name from the log, the note when it got none, else off.
            public string GpuReport
            {
                get
                {
                    if (engine.IsRomLoaded && engine.Machine.SettingNotes().TryGetValue("Gpu", out string? note)) return note;
                    Match m = engine.RecentLog.Select(l => Regex.Match(l, @"The multiple is drawn on (.*)\.$")).LastOrDefault(x => x.Success) ?? Match.Empty;
                    return m.Success ? m.Groups[1].Value : "off";
                }
            }

            public string IsViewerTranscript
            {
                get
                {
                    var text = (delegate* unmanaged<nint, byte*, nuint, long>)Export("marsrt_core_is_viewer_text");
                    nint handle = engine.Machine.Handle;
                    var bytes = new byte[text(handle, null, 0)];
                    fixed (byte* b = bytes) text(handle, b, (nuint)bytes.Length);
                    return System.Text.Encoding.UTF8.GetString(bytes);
                }
            }

            public byte[] Save(bool snapshot)
            {
                uint kind = snapshot ? 1u : 0;
                var state = new byte[engine.Machine.StateSize(kind)];
                engine.Machine.Save(state, kind);
                return state;
            }

            public void LoadState(ReadOnlySpan<byte> state)
            {
                fixed (byte* data = state)
                {
                    using var stream = new UnmanagedMemoryStream(data, state.Length);
                    engine.LoadState(stream);
                }
            }

            public byte[]? BatteryContents => engine.Machine.Battery(0).Data is { Length: > 0 } data ? data : null;

            public void WriteBus32(uint physical, uint value) => ((delegate* unmanaged<nint, uint, uint, int>)Export("marsrt_core_bus_write32"))(engine.Machine.Handle, physical, value);

            public uint ReadBus32(uint physical) => ((delegate* unmanaged<nint, uint, uint>)Export("marsrt_core_bus_read32"))(engine.Machine.Handle, physical);

            public void SetCop0(int register, ulong value) => ((delegate* unmanaged<nint, uint, ulong, int>)Export("marsrt_core_set_cop0"))(engine.Machine.Handle, (uint)register, value);

            public void RaiseInterrupt(MiInterrupt sources) => ((delegate* unmanaged<nint, uint, int>)Export("marsrt_core_mi_raise"))(engine.Machine.Handle, (uint)sources);

            // The signal processor alone: started at pc, then stepped, its coverage recorded while armed, the bridge's tables pushed and its logs drained.
            public void StepRsp(uint? startAt, int steps)
            {
                CoreMachine m = engine.Machine;
                engine.Debug.PushTables(m);
                ((delegate* unmanaged<nint, uint, uint, uint, int>)Export("marsrt_core_rsp_step"))(m.Handle, startAt is null ? 0u : 1u, startAt ?? 0, (uint)Math.Max(0, steps));
                engine.Debug.Drain(m);
            }

            public CoverageRegistry Coverage => engine.Debug.Coverage[0];

            public CoverageRegistry RspCoverage => engine.Debug.Coverage[1];

            public CallStackRegistry CallStack => engine.Debug.CallStack;

            // A store is reported while a watch or a data breakpoint wants it.
            public bool Listening => engine.Watches.HasWatches || engine.Debug.Breakpoints.Any(b => b.WatchesWrites);

            public int Pc => engine.IsRomLoaded ? engine.Debug.ProgramCounter(0) : 0;

            public int EffectiveAntialiasing
            {
                get
                {
                    int scale = Math.Clamp(int.Parse(engine.Get("RenderScale")), 1, 4);
                    string chosen = engine.Get("Antialiasing");
                    int wanted = chosen is "Off" ? 1 : chosen[0] - '0';
                    return Math.Max(1, Math.Min(wanted, 4 / scale));
                }
            }

            public bool InterruptsEnabled => (engine.Cop0(StatusRegister) & 1) != 0;
        }
    }

    // MarsRT on the generic adapter with its settings as properties, the corpus's boot and the interpreter's steps, for its own tests; it adds nothing to a frame.
    public sealed class MarsRtTestCore : CoreEngine
    {
        private bool _booting;

        public MarsRtTestCore(bool expansionPak = false)
            : base(MarsRt.Library ?? throw new InvalidOperationException($"MarsRT is not in use: {MarsRt.Report}"), new Dictionary<string, string> { ["ExpansionPak"] = expansionPak ? "true" : "false" })
        {
        }

        private bool Flag(string key) => Get(key) == "true";
        private void Flag(string key, bool on) => Set(key, on ? "true" : "false");

        public bool ExpansionPak { get => Flag("ExpansionPak"); set => Flag("ExpansionPak", value); }
        public bool IdleSkip { get => Flag("IdleSkip"); set => Flag("IdleSkip", value); }
        public bool RspWhole { get => Flag("RspWhole"); set => Flag("RspWhole", value); }
        public bool Gpu { get => Flag("Gpu"); set => Flag("Gpu", value); }
        public bool ThreadedRdp { get => Flag("ThreadedRdp"); set => Flag("ThreadedRdp", value); }
        public bool DeferredPresentation { get => Flag("DeferredPresentation"); set => Flag("DeferredPresentation", value); }
        public bool SkipRepeatedScans { get => Flag("SkipRepeatedScans"); set => Flag("SkipRepeatedScans", value); }
        public bool UseBlocks { get => Flag("Recompiler"); set => Flag("Recompiler", value); }
        public bool VerifyBlocks { get => Flag("VerifyBlocks"); set => Flag("VerifyBlocks", value); }
        public bool VerifyRdp { get => Flag("VerifyRdp"); set => Flag("VerifyRdp", value); }
        public int RdpWorkers { get => int.Parse(Get("RdpWorkers")); set => Set("RdpWorkers", Math.Clamp(value, 1, 8).ToString()); }
        public int BlockTier { get => int.Parse(Get("BlockTier")); set => Set("BlockTier", Math.Clamp(value, 0, 15).ToString()); }
        public int RenderScale { get => int.Parse(Get("RenderScale")); set => Set("RenderScale", Math.Clamp(value, 1, 4).ToString()); }

        public int Antialiasing
        {
            get => Get("Antialiasing") is "Off" ? 1 : Get("Antialiasing")[0] - '0';
            set
            {
                int level = Math.Clamp(value, 1, 4);
                Set("Antialiasing", level == 1 ? "Off" : $"{level}x");
            }
        }

        // The interpreter's steps with the buttons held now, through the blocks when they are on; the corpus's way.
        public void RunSteps(ulong steps)
        {
            CoreMachine m = Machine;
            RefreshCheats(m);
            SendInput(m);
            lock (m.Gate) MarsRt.RunSteps(m, steps);
        }

        // As the corpus boots a machine: the boot code handed off, nothing loaded beside it and no battery file.
        public void Boot(string path, bool expansionPak = true)
        {
            string pak = Get("ExpansionPak");
            Keep("ExpansionPak", expansionPak ? "true" : "false");
            Keep("Boot", "true");
            _booting = true;
            try { LoadRom(path); }
            finally
            {
                _booting = false;
                Keep("Boot", "false");
                Keep("ExpansionPak", pak);
            }
        }

        protected override BatterySave BatteryFile(string path, string console, uint which, string suffix) =>
            _booting ? BatterySave.None : base.BatteryFile(path, console, which, suffix);
    }
}
