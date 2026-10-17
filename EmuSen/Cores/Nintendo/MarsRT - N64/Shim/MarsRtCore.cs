using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The memories the library reads and writes by number; Cpu is the processor's kernel view of a virtual address - see Mars_Native.md §5.5.
    public enum MarsRtSpace : uint { Rdram = 0, Dmem = 1, Imem = 2, PifRam = 3, Rom = 4, Cpu = 5 }

    // MarsRT, the N64 core in Rust, through the generic v1 adapter with its own frame end, settings and debugger - see Mars_Native.md §7.2.
    public sealed unsafe partial class MarsRtCore : CoreEngine, ICore, IFrameProfiler
    {
        private static nint X(string name) => MarsNative.Export(name);

        private static readonly delegate* unmanaged<nint, long*, nuint, long> CountersExport = (delegate* unmanaged<nint, long*, nuint, long>)X("marsrt_core_counters");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> ThreadCounters = (delegate* unmanaged<nint, long*, nuint, long>)X("marsrt_core_thread_counters");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> BlockCounters = (delegate* unmanaged<nint, long*, nuint, long>)X("marsrt_core_block_counters");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> GpuReportExport = (delegate* unmanaged<nint, byte*, nuint, long>)X("marsrt_core_gpu_report");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> IsViewerText = (delegate* unmanaged<nint, byte*, nuint, long>)X("marsrt_core_is_viewer_text");
        private static readonly delegate* unmanaged<nint, ulong, int> RunStepsExport = (delegate* unmanaged<nint, ulong, int>)X("marsrt_core_run_steps");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveCpu = (delegate* unmanaged<nint, byte*, nuint, long>)X("marsrt_core_save_cpu");
        private static readonly delegate* unmanaged<nint, uint, ulong> Cop0Export = (delegate* unmanaged<nint, uint, ulong>)X("marsrt_core_cop0");
        private static readonly delegate* unmanaged<nint, ulong*, nuint, long> CpuRegistersExport = (delegate* unmanaged<nint, ulong*, nuint, long>)X("marsrt_core_cpu_registers");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> RspRegistersExport = (delegate* unmanaged<nint, uint*, nuint, long>)X("marsrt_core_rsp_registers");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> ViRegistersExport = (delegate* unmanaged<nint, uint*, nuint, long>)X("marsrt_core_vi_registers");
        private static readonly delegate* unmanaged<nint, uint, uint, int> SetPresent = (delegate* unmanaged<nint, uint, uint, int>)X("marsrt_core_set_present");

        private const int StatusRegister = 12;
        private const int ExpandedRdram = 8 * 1024 * 1024;
        private const int NotARom = -9;

        private bool _booting;

        public static bool Available => MarsNative.Available;

        // A stock console unless asked.
        public MarsRtCore(bool expansionPak = false)
            : base(MarsNative.Library.Library ?? throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}"),
                new Dictionary<string, string> { [Key.ExpansionPak] = expansionPak ? "true" : "false" })
        {
            WireRegistries();
        }

        // The schema's keys, the console's settings and MarsRT's own - see Mars_Native.md §7.2.
        private static class Key
        {
            public const string ExpansionPak = "ExpansionPak", Boot = "Boot", ThreadedRdp = "ThreadedRdp", RdpWorkers = "RdpWorkers", DeferredPresentation = "DeferredPresentation";
            public const string SkipRepeatedScans = "SkipRepeatedScans", RenderScale = "RenderScale", Antialiasing = "Antialiasing", Gpu = "Gpu", Recompiler = "Recompiler";
            public const string IdleSkip = "IdleSkip", RspWhole = "RspWhole", VerifyBlocks = "VerifyBlocks", BlockTier = "BlockTier", VerifyRdp = "VerifyRdp";
        }

        private bool Flag(string key) => base.Get(key) == "true";
        private void Flag(string key, bool on) => base.Set(key, on ? "true" : "false");
        private int Number(string key) => int.Parse(base.Get(key));
        private void Number(string key, int value) => base.Set(key, value.ToString());

        // The Pak the next load builds; before the first frame a change rebuilds the machine at once.
        public bool ExpansionPak
        {
            get => Flag(Key.ExpansionPak);
            set
            {
                if (value == ExpansionPak) return;
                Flag(Key.ExpansionPak, value);
                if (IsRomLoaded && TotalFrames == 0 && RomPath.Length > 0) LoadRom(RomPath);
            }
        }

        public override string CoreName => "N64";
        public override int ScreenWidth => IsRomLoaded ? base.ScreenWidth : N64Console.ScreenWidthPixels;
        public override int ScreenHeight => IsRomLoaded ? base.ScreenHeight : N64Console.DefaultScreenHeight;
        // The screen's shape: a television's 4:3 at every VI width and in PAL - see EmuSen_Serenity.md §2.9.
        public override double DisplayAspect => EmuSen.Crystal.DisplayShape.Television;

        public int DisplayLines => N64Console.LinesOf(ScreenWidth, ScreenHeight);

        public override double FrameRateHz => N64Console.ProcessorClockHz / (double)(IsRomLoaded ? Counter(2) : N64Console.CycleCap);
        public long Cycles => Counter(0);
        public long Instructions => Counter(3);
        public long IdleTurnsPassed => Counter(4);
        public override int StateVersion => MarsMachine.StateKind;

        public int ControllerPorts => N64Console.Ports;

        // The idle loop passed in one piece; exact either way - see Mars_Recompiler.md.
        public bool IdleSkip
        {
            get => Flag(Key.IdleSkip);
            set => Flag(Key.IdleSkip, value);
        }

        // A running signal processor run to its next event inside the idle loop; exact either way.
        public bool RspWhole
        {
            get => Flag(Key.RspWhole);
            set => Flag(Key.RspWhole, value);
        }

        // Mars's RenderScale: the picture drawn at a multiple of the console's beside the exact drawing, on the library's threads - see Mars_Native.md §6.4.
        public int RenderScale
        {
            get => Number(Key.RenderScale);
            set => Number(Key.RenderScale, Math.Clamp(value, 1, 4));
        }

        // Mars's Antialiasing: the drawing that many times finer than shown and averaged down, held to four with the resolution - see Mars_Native.md §6.4.
        public int Antialiasing
        {
            get => base.Get(Key.Antialiasing) is "Off" ? 1 : base.Get(Key.Antialiasing)[0] - '0';
            set
            {
                int level = Math.Clamp(value, 1, 4);
                base.Set(Key.Antialiasing, level == 1 ? "Off" : $"{level}x");
            }
        }

        public int EffectiveAntialiasing => N64Console.EffectiveAntialiasingOf(RenderScale, Antialiasing);

        // Mars's Gpu: asked of the library, which reports what it got - see Mars_Native.md §6.4.
        public bool Gpu
        {
            get => Flag(Key.Gpu);
            set => Flag(Key.Gpu, value);
        }

        public string GpuReport => IsRomLoaded ? Text(GpuReportExport) : "off";

        // Mars's ThreadedRdp: the display processor's lists on a thread of MarsRT's own, behind page marks - see Mars_Native.md §5.6.
        public bool ThreadedRdp
        {
            get => Flag(Key.ThreadedRdp);
            set => Flag(Key.ThreadedRdp, value);
        }

        // Mars's RdpWorkers: processors sharing a threaded list, each shading every Nth row.
        public int RdpWorkers
        {
            get => Number(Key.RdpWorkers);
            set => Number(Key.RdpWorkers, Math.Clamp(value, 1, 8));
        }

        // Mars's DeferredPresentation: the picture walked on another thread while the next frame runs, and shown a frame late.
        public bool DeferredPresentation
        {
            get => Flag(Key.DeferredPresentation);
            set => Flag(Key.DeferredPresentation, value);
        }

        public bool SkipRepeatedScans
        {
            get => Flag(Key.SkipRepeatedScans);
            set => Flag(Key.SkipRepeatedScans, value);
        }

        // The recompiler: the processor's code compiled in blocks, validated against memory on entry - see Mars_Native.md §5.8.
        public bool UseBlocks
        {
            get => Flag(Key.Recompiler);
            set => Flag(Key.Recompiler, value);
        }

        // The interpreter run beside the blocks and compared after every instruction; for tests.
        public bool VerifyBlocks
        {
            get => Flag(Key.VerifyBlocks);
            set => Flag(Key.VerifyBlocks, value);
        }

        // Which step of the recompiler runs: 0 the furthest built, 1 decoded blocks, 2 compiled, 3 compiled with registers held.
        public int BlockTier
        {
            get => Number(Key.BlockTier);
            set => Number(Key.BlockTier, Math.Clamp(value, 0, 15));
        }

        // Every byte the display processor's thread touches checked against the marks; for tests.
        public bool VerifyRdp
        {
            get => Flag(Key.VerifyRdp);
            set => Flag(Key.VerifyRdp, value);
        }

        // The recompiler's counters: live, shaped, discarded, entries, instructions, stepped, mapped, compiled, nanoseconds, bytes, compiled entries.
        public long[] BlockCounterValues() => Longs(BlockCounters);

        // The thread's counters: running, its words, starts and time, the waits, bystanders, reads, repeated scans, and by site.
        public long[] ThreadCounterValues() => Longs(ThreadCounters);

        // MarsRT's own key, which the console's list does not offer; declared before VideoSettings, whose initializer reads it - see Mars_Native.md §5.8.
        private static readonly CoreSetting Recompiler = new(Key.Recompiler, "Compile the processor's code", "The processor's code is compiled to the host's machine code in blocks, each compared with memory before it runs, on a thread of its own. Exact: frame for frame the interpreter, which is what off leaves running, slower.", CoreSettingKind.Switch, "true");

        // The console's keys, every one honoured since stage D, with the threads' hints in MarsRT's words - see Mars_Native.md §5.5, §5.6 and §6.4.
        public static readonly IReadOnlyList<CoreSetting> VideoSettings = N64Console.VideoSettings.Select(Threads).Append(Recompiler).ToArray();

        private static CoreSetting Threads(CoreSetting s) => s.Key switch
        {
            Key.ThreadedRdp => s with { Hint = "The display processor runs its lists on a thread of its own, behind marks on the memory it reaches. Exact: frame for frame the machine on one thread." },
            Key.RdpWorkers => s with { Hint = "How many processors share each list when it runs on a thread of its own, each shading every Nth row. Exact at any count: frame for frame the machine on one thread. One per three cores is the default." },
            Key.DeferredPresentation => s with { Hint = "The picture is finished on another thread while the machine runs the next frame, so it reaches the screen one frame late, exactly the picture it would have been." },
            _ => s,
        };

        public override IReadOnlyList<CoreSetting> Settings => VideoSettings;

        public override string Get(string key) => base.Get(Setting(key).Key);

        // Checked as its kind says, text that is no value refused and a count clamped; the Expansion Pak acts - see Mars_Native.md §5.5 and §5.6.
        public override void Set(string key, string value)
        {
            CoreSetting setting = Setting(key);
            string accepted = setting.Kind switch
            {
                CoreSettingKind.Switch => bool.TryParse(value, out bool on) ? (on ? "true" : "false") : throw new ArgumentException($"{value} is not on or off."),
                CoreSettingKind.Count => int.TryParse(value, out int count) ? Math.Clamp(count, setting.Min, setting.Max).ToString() : throw new ArgumentException($"{value} is not a count."),
                _ => setting.Choices?.Contains(value) == true ? value : throw new ArgumentException($"{value} is not a choice of {setting.Label}."),
            };
            if (key == Key.ExpansionPak) ExpansionPak = accepted == "true";
            else base.Set(key, accepted);
        }

        private static CoreSetting Setting(string key) =>
            VideoSettings.FirstOrDefault(s => s.Key == key) ?? throw new ArgumentException($"MarsRT has no setting named {key}.", nameof(key));

        // Past the end reads zero and a write there is dropped - see Mars_Cheats.md §3.1.
        private byte ReadForCheat(string spaceName, int address) => CheatSpace(spaceName) is { } space ? Peek(space, (uint)address) : (byte)0;

        private void WriteForCheat(string spaceName, int address, byte value)
        {
            if (CheatSpace(spaceName) is { } space) Poke(space, (uint)address, value);
        }

        private static MarsRtSpace? CheatSpace(string spaceName)
        {
            if (string.Equals(spaceName, MarsDebugSpaces.Rdram, StringComparison.OrdinalIgnoreCase)) return MarsRtSpace.Rdram;
            if (string.Equals(spaceName, MarsDebugSpaces.Dmem, StringComparison.OrdinalIgnoreCase)) return MarsRtSpace.Dmem;
            if (string.Equals(spaceName, MarsDebugSpaces.Imem, StringComparison.OrdinalIgnoreCase)) return MarsRtSpace.Imem;
            if (string.Equals(spaceName, MarsDebugSpaces.PifRam, StringComparison.OrdinalIgnoreCase)) return MarsRtSpace.PifRam;
            return null;
        }

        // A memory's length, zero before a ROM; the processor's view is the whole 32-bit space.
        public long SpaceSize(MarsRtSpace space) => IsRomLoaded ? Machine.SpaceSize((uint)space) : 0;

        public byte Peek(MarsRtSpace space, uint address)
        {
            Span<byte> one = stackalloc byte[1];
            if (IsRomLoaded) Machine.ReadSpace((uint)space, (int)address, one);
            return one[0];
        }

        public void Poke(MarsRtSpace space, uint address, byte value)
        {
            if (IsRomLoaded) Machine.WriteSpace((uint)space, (int)address, stackalloc byte[] { value });
        }

        // A copy of a stretch of memory, zero wherever the address names nothing.
        public byte[] ReadSpace(MarsRtSpace space, uint address, int length)
        {
            var bytes = new byte[length];
            if (IsRomLoaded && length > 0) Machine.ReadSpace((uint)space, (int)address, bytes);
            return bytes;
        }

        public ulong Cop0(int register) => IsRomLoaded ? Cop0Export(Machine.Handle, (uint)register) : 0;

        // PC, the 32 GPRs, HI, LO and the cycle count.
        public ulong[] CpuRegisters()
        {
            if (!IsRomLoaded) return Array.Empty<ulong>();
            nint handle = Machine.Handle;
            var values = new ulong[CpuRegistersExport(handle, null, 0)];
            fixed (ulong* data = values) CpuRegistersExport(handle, data, (nuint)values.Length);
            return values;
        }

        // PC, halted, broke, and the 32 scalar registers.
        public uint[] RspRegisters() => Words(RspRegistersExport);

        public uint[] ViRegisters() => Words(ViRegistersExport);

        private uint[] Words(delegate* unmanaged<nint, uint*, nuint, long> export)
        {
            if (!IsRomLoaded) return Array.Empty<uint>();
            nint handle = Machine.Handle;
            var values = new uint[export(handle, null, 0)];
            fixed (uint* data = values) export(handle, data, (nuint)values.Length);
            return values;
        }

        private long[] Longs(delegate* unmanaged<nint, long*, nuint, long> export)
        {
            nint handle = Handle;
            var values = new long[export(handle, null, 0)];
            fixed (long* data = values) export(handle, data, (nuint)values.Length);
            return values;
        }

        private string Text(delegate* unmanaged<nint, byte*, nuint, long> export)
        {
            nint handle = Handle;
            var text = new byte[export(handle, null, 0)];
            fixed (byte* data = text) export(handle, data, (nuint)text.Length);
            return Encoding.UTF8.GetString(text);
        }

        // The save chip's bytes as the game left them, or null for a cartridge with none.
        public byte[]? BatteryContents => Machine.Battery(0).Data is { Length: > 0 } data ? data : null;

        // The game's save chip and first pak; none while booting as the corpus boots.
        protected override BatterySave BatteryFile(string path, string console, uint which, string suffix) =>
            _booting ? BatterySave.None : BatterySave.Open(path, BatterySave.N64, disabled: Options.BatteryRamDisabled, extension: suffix);

        public override void LoadRom(string path)
        {
            try { base.LoadRom(path); }
            catch (CoreRefusedException e) when (e.Status == NotARom) { throw new InvalidDataException($"MarsRT refused {path} as a Nintendo 64 image.", e); }
            CallStack.Reset();
            IsHaltedAtBreakpoint = false;
        }

        // As the corpus boots a machine: the boot code handed off and nothing loaded beside it.
        public void Boot(string path, bool expansionPak = true)
        {
            bool pak = ExpansionPak;
            Flag(Key.ExpansionPak, expansionPak);
            Flag(Key.Boot, true);
            _booting = true;
            try { LoadRom(path); }
            finally
            {
                _booting = false;
                Flag(Key.Boot, false);
                Flag(Key.ExpansionPak, pak);
            }
        }

        // A library built before the export answers for the first controller alone, as it always did.
        public void SetControllerConnected(int port, bool connected)
        {
            if (IsRomLoaded && port >= 0 && port < N64Console.Ports) SetPresent(Machine.Handle, (uint)port, connected ? 1u : 0);
        }

        // After the frame: the cheats, the frame log, the periodic save - see Mars_Native.md §5.5 and §6.5.
        protected override void EndFrame(CoreMachine m)
        {
            if ((Cop0(StatusRegister) & 1) != 0) Cheats.ApplyAll(ReadForCheat, WriteForCheat);
            if (Freezes.Count > 0) Freezes.Enforce(base.ReadSpace, base.WriteSpace);
            FrameLog.RecordFrame(TotalFrames, ReadWidth);
            Breakpoints.NoteFrame(TotalFrames);
            if (BatterySave.IsFlushFrame(TotalFrames)) SaveSram();
        }

        // Through the observed loop when anything can halt or record; else the plain frame.
        protected override bool AdvanceFrame(CoreMachine m)
        {
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;
            if (Observed) return RunObserved(m.Handle, resuming);
            m.Advance();
            return true;
        }

        // The library's phases for the frame just run, read into one array - see Mars_Native.md §6.6.2.
        public new IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases
        {
            get
            {
                if (!IsRomLoaded) return _phases;
                long* nanos = stackalloc long[5];
                Library.Api.Phases(Machine.Handle, nanos, 5);
                for (int p = 0; p < _phases.Length; p++) _phases[p] = (_phases[p].Name, nanos[p] / 1e6);
                return _phases;
            }
        }

        private readonly (string Name, double Milliseconds)[] _phases =
        {
            ("machine", 0), ("machine/rdp-wait", 0), ("present", 0), ("present/rdp-wait", 0), ("present/walk-join", 0),
        };

        // The samples not yet drained, left for the frontend's drain - see Mars_Native.md §6.6.2.
        public short[] PeekAudioSamples() => IsRomLoaded ? Machine.PeekAudio() : Array.Empty<short>();

        // The interpreter's steps, the given number, with the buttons held now; through the blocks when they are on.
        public void RunSteps(ulong steps)
        {
            CoreMachine m = Machine;
            RefreshCheats(m);
            SendInput(m);
            lock (m.Gate) RunStepsExport(m.Handle, steps);
        }

        public string IsViewerTranscript => Text(IsViewerText);

        // A state of the other memory size rebuilds the machine, and the Pak follows it.
        public override void LoadState(Stream stream)
        {
            if (!IsRomLoaded) throw new InvalidOperationException("LoadState() called before LoadRom().");
            long before = SpaceSize(MarsRtSpace.Rdram);
            try { base.LoadState(stream); }
            catch (InvalidDataException e) { throw new InvalidDataException($"MarsRT refused the state: {Machine.Words(CoreMachine.StatusOf(e) ?? 0)}.", e); }
            long after = SpaceSize(MarsRtSpace.Rdram);
            if (after != before) Flag(Key.ExpansionPak, after == ExpandedRdram);
        }

        // A state from bytes held in place - see Mars_SaveStates.md §1.
        public void LoadState(ReadOnlySpan<byte> state)
        {
            fixed (byte* data = state)
            {
                using var stream = new UnmanagedMemoryStream(data, state.Length);
                LoadState(stream);
            }
        }

        public byte[] Save(bool snapshot)
        {
            uint kind = snapshot ? 1u : 0;
            var state = new byte[Machine.StateSize(kind)];
            Machine.Save(state, kind);
            return state;
        }

        // The CPU's fields alone, in the state's order, for a comparison made every few instructions.
        public byte[] SaveProcessor()
        {
            nint handle = Handle;
            var bytes = new byte[SaveCpu(handle, null, 0)];
            fixed (byte* data = bytes) SaveCpu(handle, data, (nuint)bytes.Length);
            return bytes;
        }

        private long Counter(int index)
        {
            long* values = stackalloc long[6];
            CountersExport(Handle, values, 6);
            return values[index];
        }

        private nint Handle => IsRomLoaded ? Machine.Handle : throw new InvalidOperationException("MarsRT has no ROM loaded.");
    }
}
