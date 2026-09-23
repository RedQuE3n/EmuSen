using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using EmuSen.Cores.Nintendo.Mars;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Native;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The memories the library reads and writes by number; Cpu is the processor's kernel view of a virtual address - see Mars_Native.md §5.5.
    public enum MarsRtSpace : uint { Rdram = 0, Dmem = 1, Imem = 2, PifRam = 3, Rom = 4, Cpu = 5 }

    // MarsRT, the N64 core in Rust, behind the interfaces MarsCore implements; the boundary is crossed once a frame - see Mars_Native.md §5.2 and §5.5.
    public sealed unsafe partial class MarsRtCore : ICore, ISnapshotCore, IStateFormat, IFrameSerial, IRepeatedRows, IFrameBufferPool, ICoreSettings, ICheatRegistryHost, IFrameProfiler, IDisposable
    {
        private static readonly delegate* unmanaged<byte*, nuint, uint, byte*, nuint, byte*, nuint, nint> LoadRomExport = (delegate* unmanaged<byte*, nuint, uint, byte*, nuint, byte*, nuint, nint>)MarsNative.Export("mars_machine_load_rom");
        private static readonly delegate* unmanaged<byte*, nuint, uint, nint> BootExport = (delegate* unmanaged<byte*, nuint, uint, nint>)MarsNative.Export("mars_machine_boot");
        private static readonly delegate* unmanaged<nint, void> Free = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_free");
        private static readonly delegate* unmanaged<nint, void> AdvanceExport = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_advance");
        private static readonly delegate* unmanaged<nint, void> PresentExport = (delegate* unmanaged<nint, void>)MarsNative.Export("mars_machine_present");
        private static readonly delegate* unmanaged<nint, uint, uint, void> SetThreads = (delegate* unmanaged<nint, uint, uint, void>)MarsNative.Export("mars_machine_set_threads");
        private static readonly delegate* unmanaged<nint, int, int, uint, void> SetMultiple = (delegate* unmanaged<nint, int, int, uint, void>)MarsNative.Export("mars_machine_set_multiple");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> GpuReportExport = (delegate* unmanaged<nint, byte*, nuint, long>)MarsNative.Export("mars_machine_gpu_report");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> ThreadCounters = (delegate* unmanaged<nint, long*, nuint, long>)MarsNative.Export("mars_threads_counters");
        private static readonly delegate* unmanaged<nint, ulong, void> RunStepsExport = (delegate* unmanaged<nint, ulong, void>)MarsNative.Export("mars_machine_run_steps");
        private static readonly delegate* unmanaged<nint, uint, void> SetOptions = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_set_options");
        private static readonly delegate* unmanaged<nint, uint, void> SetRecompiler = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_set_recompiler");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> BlockCounters = (delegate* unmanaged<nint, long*, nuint, long>)MarsNative.Export("mars_blocks_counters");
        private static readonly delegate* unmanaged<nint, uint, uint, uint, void> PressExport = (delegate* unmanaged<nint, uint, uint, uint, void>)MarsNative.Export("mars_machine_press");
        private static readonly delegate* unmanaged<nint, uint, uint, int, void> SetStick = (delegate* unmanaged<nint, uint, uint, int, void>)MarsNative.Export("mars_machine_set_stick");
        private static readonly delegate* unmanaged<nint, ulong> AudioBuffered = (delegate* unmanaged<nint, ulong>)MarsNative.Export("mars_machine_audio_buffered");
        private static readonly delegate* unmanaged<nint, short*, ulong, ulong> DrainAudio = (delegate* unmanaged<nint, short*, ulong, ulong>)MarsNative.Export("mars_machine_drain_audio");
        private static readonly delegate* unmanaged<nint, int> SampleRate = (delegate* unmanaged<nint, int>)MarsNative.Export("mars_machine_audio_sample_rate");
        private static readonly delegate* unmanaged<nint, long*, uint> FrameInfo = (delegate* unmanaged<nint, long*, uint>)MarsNative.Export("mars_machine_frame_info");
        private static readonly delegate* unmanaged<nint, byte*, nuint, ulong> FrameBytes = (delegate* unmanaged<nint, byte*, nuint, ulong>)MarsNative.Export("mars_machine_frame_bytes");
        private static readonly delegate* unmanaged<nint, long*, void> Counters = (delegate* unmanaged<nint, long*, void>)MarsNative.Export("mars_machine_counters");
        private static readonly delegate* unmanaged<nint, byte*, nuint, ulong> IsViewerText = (delegate* unmanaged<nint, byte*, nuint, ulong>)MarsNative.Export("mars_machine_is_viewer_text");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int> RestoreState = (delegate* unmanaged<nint, byte*, nuint, int>)MarsNative.Export("mars_machine_restore_state");
        private static readonly delegate* unmanaged<nint, uint> RdramBytes = (delegate* unmanaged<nint, uint>)MarsNative.Export("mars_machine_rdram_bytes");
        private static readonly delegate* unmanaged<nint, uint, long> SizeOf = (delegate* unmanaged<nint, uint, long>)MarsNative.Export("mars_machine_save_state_size");
        private static readonly delegate* unmanaged<nint, byte*, nuint, uint, long> SaveStateExport = (delegate* unmanaged<nint, byte*, nuint, uint, long>)MarsNative.Export("mars_machine_save_state");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int*, long> SaveData = (delegate* unmanaged<nint, byte*, nuint, int*, long>)MarsNative.Export("mars_machine_save_data");
        private static readonly delegate* unmanaged<nint, byte*, nuint, int*, long> PakData = (delegate* unmanaged<nint, byte*, nuint, int*, long>)MarsNative.Export("mars_machine_pak_data");
        private static readonly delegate* unmanaged<nint, uint, void> MarkSaved = (delegate* unmanaged<nint, uint, void>)MarsNative.Export("mars_machine_mark_saved");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SaveCpu = (delegate* unmanaged<nint, byte*, nuint, long>)MarsNative.Export("mars_machine_save_cpu");
        private static readonly delegate* unmanaged<nint, uint, long> MemorySize = (delegate* unmanaged<nint, uint, long>)MarsNative.Export("mars_machine_memory_size");
        private static readonly delegate* unmanaged<nint, uint, uint, byte*, nuint, long> ReadMemory = (delegate* unmanaged<nint, uint, uint, byte*, nuint, long>)MarsNative.Export("mars_machine_read_memory");
        private static readonly delegate* unmanaged<nint, uint, uint, byte*, nuint, long> WriteMemory = (delegate* unmanaged<nint, uint, uint, byte*, nuint, long>)MarsNative.Export("mars_machine_write_memory");
        private static readonly delegate* unmanaged<nint, long*, nuint, long> FrameProfile = (delegate* unmanaged<nint, long*, nuint, long>)MarsNative.Export("mars_machine_frame_profile");
        private static readonly delegate* unmanaged<nint, short*, nuint, long> PeekAudio = (delegate* unmanaged<nint, short*, nuint, long>)MarsNative.Export("mars_machine_peek_audio");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> SetRomPatches = (delegate* unmanaged<nint, uint*, nuint, long>)MarsNative.Export("mars_machine_set_rom_patches");
        private static readonly delegate* unmanaged<nint, uint, ulong> Cop0Export = (delegate* unmanaged<nint, uint, ulong>)MarsNative.Export("mars_machine_cop0");
        private static readonly delegate* unmanaged<nint, ulong*, nuint, long> CpuRegistersExport = (delegate* unmanaged<nint, ulong*, nuint, long>)MarsNative.Export("mars_machine_cpu_registers");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> RspRegistersExport = (delegate* unmanaged<nint, uint*, nuint, long>)MarsNative.Export("mars_machine_rsp_registers");
        private static readonly delegate* unmanaged<nint, uint*, nuint, long> ViRegistersExport = (delegate* unmanaged<nint, uint*, nuint, long>)MarsNative.Export("mars_machine_vi_registers");

        private const ushort ButtonA = 0x8000, ButtonB = 0x4000, ButtonZ = 0x2000, ButtonStart = 0x1000;
        private const ushort DpadUp = 0x0800, DpadDown = 0x0400, DpadLeft = 0x0200, DpadRight = 0x0100;
        private const ushort ButtonL = 0x0020, ButtonR = 0x0010;
        private const ushort CUp = 0x0008, CDown = 0x0004, CLeft = 0x0002, CRight = 0x0001;
        private const int StatusRegister = 12;
        private const int ExpandedRdram = 8 * 1024 * 1024;

        private nint _handle;
        private string? _romPath, _savePath, _pakPath;
        private readonly bool? _batteryRamDisabled;
        private byte[] _frame = Blank(MarsCore.DefaultScreenHeight);
        private int _screenWidth = MarsCore.ScreenWidthPixels, _screenHeight = MarsCore.DefaultScreenHeight, _rowRepeat = 1;
        private long _frameSerial, _takenSerial;
        private bool _skipRendering, _idleSkip = true, _rspWhole = true, _repeatRows = true;

        // Mars's defaults since 2026-09-22, measured on the desktop and the handheld; each is exact either way - see Mars_Native.md §6.2.
        private bool _threadedRdp = true, _deferredPresentation = true, _skipRepeatedScans = true, _verifyRdp;

        // The multiple, the averaging and the device as asked for; what they got is the library's - see Mars_Native.md §6.4.
        private int _renderScale = 1, _antialiasing = 1;
        private bool _gpu;
        private int _rdpWorkers = Math.Clamp(Environment.ProcessorCount / 3, 1, 4);

        // The recompiler, on since 2026-09-22 at the tier §5.8.7 recommends; off is the interpreter, exact as well - see Mars_Native.md §5.8.
        private bool _useBlocks = true, _verifyBlocks;
        private int _blockTier;

        public static bool Available => LoadRomExport != null && MemorySize != null && SetRomPatches != null && FrameProfile != null && PeekAudio != null;

        // A stock console unless asked, as MarsCore; null defers to --nobattery.
        public MarsRtCore(bool expansionPak = false, bool? batteryRamDisabled = null)
        {
            if (!Available) throw new InvalidOperationException($"MarsRT is not in use: {MarsNative.Report}");
            _expansionPak = expansionPak;
            _batteryRamDisabled = batteryRamDisabled;
            WireRegistries();
        }

        // The Pak the next load builds; before the first frame a change rebuilds the machine at once, as MarsCore's does.
        public bool ExpansionPak
        {
            get => _expansionPak;
            set
            {
                if (value == _expansionPak) return;
                _expansionPak = value;
                if (_handle != 0 && TotalFrames == 0 && _romPath is { } path) LoadRom(path);
            }
        }

        private bool _expansionPak;

        public string CoreName => "N64";
        public int ScreenWidth => _screenWidth;
        public int ScreenHeight => _screenHeight;
        public double FrameRateHz => MarsCore.ProcessorClockHz / (double)(_handle == 0 ? MarsCore.CycleCap : Counter(2));
        public bool IsRomLoaded => _handle != 0;
        public long TotalFrames => _handle == 0 ? 0 : Counter(1);
        public long Cycles => Counter(0);
        public long Instructions => Counter(3);
        public long IdleTurnsPassed => Counter(4);
        public int StateVersion => 1;
        public long FrameSerial => _frameSerial;
        public bool RepeatRows
        {
            get => _repeatRows;
            set { _repeatRows = value; ApplyOptions(); }
        }

        public int RowRepeat => _rowRepeat;
        public IReadOnlyList<PadButton> SupportedButtons => MarsCore.PadButtons;
        public IReadOnlyList<PadAxis> SupportedAxes => MarsCore.PadAxes;
        public int AudioSampleRate => _handle == 0 ? 44100 : SampleRate(_handle);

        public bool SkipRendering
        {
            get => _skipRendering;
            set { _skipRendering = value; ApplyOptions(); }
        }

        // The idle loop passed in one piece, as C#'s Cpu.SkipIdle; exact either way.
        public bool IdleSkip
        {
            get => _idleSkip;
            set { _idleSkip = value; ApplyOptions(); }
        }

        // A running signal processor run to its next event inside the idle loop, as C#'s RunBlocks; exact either way.
        public bool RspWhole
        {
            get => _rspWhole;
            set { _rspWhole = value; ApplyOptions(); }
        }

        // What was last sent to _appliedHandle, so a setter re-sends only what changed; Mistress sets SkipRendering before every frame - see Mars_Native.md §6.4.8.
        private nint _appliedHandle;
        private (uint Options, uint Threads, uint Workers, uint Recompiler, int Scale, int Antialiasing, uint Gpu) _applied;

        private void ApplyOptions()
        {
            if (_handle == 0) return;
            var want = ((_skipRendering ? 1u : 0) | (_idleSkip ? 0 : 2u) | (_rspWhole ? 0 : 4u) | (_repeatRows ? 16u : 0),
                (_threadedRdp ? 1u : 0) | (_deferredPresentation ? 2u : 0) | (_skipRepeatedScans ? 0 : 4u) | (_verifyRdp ? 8u : 0), (uint)_rdpWorkers,
                (_useBlocks ? 1u : 0) | (_verifyBlocks ? 2u : 0) | ((uint)_blockTier << 4),
                _renderScale, _antialiasing, _gpu ? 1u : 0);
            bool fresh = _appliedHandle != _handle;
            if (!fresh && want == _applied) return;
            if (fresh || want.Item1 != _applied.Options) SetOptions(_handle, want.Item1);
            long serial = FrameSerialOf(_handle);
            if (fresh || want.Item2 != _applied.Threads || want.Item3 != _applied.Workers) SetThreads(_handle, want.Item2, want.Item3);
            if (fresh || want.Item4 != _applied.Recompiler) SetRecompiler(_handle, want.Item4);
            if (fresh || (want.Item5, want.Item6, want.Item7) != (_applied.Scale, _applied.Antialiasing, _applied.Gpu)) SetMultiple(_handle, want.Item5, want.Item6, want.Item7);
            _appliedHandle = _handle;
            _applied = want;
            if (!_skipRendering && FrameSerialOf(_handle) != serial) TakePicture();
        }

        // Mars's RenderScale: the picture drawn at a multiple of the console's beside the exact drawing, on the library's threads; takes effect at the next frame - see Mars_Native.md §6.4.
        public int RenderScale
        {
            get => _renderScale;
            set { _renderScale = Math.Clamp(value, 1, 4); ApplyOptions(); }
        }

        // Mars's Antialiasing: the drawing that many times finer than shown and averaged down, held to four with the resolution - see Mars_Native.md §6.4.
        public int Antialiasing
        {
            get => _antialiasing;
            set { _antialiasing = Math.Clamp(value, 1, 4); ApplyOptions(); }
        }

        public int EffectiveAntialiasing => Math.Max(1, Math.Min(_antialiasing, 4 / _renderScale));

        // Mars's Gpu: asked of the library, which reports what it got - see Mars_Native.md §6.4.
        public bool Gpu
        {
            get => _gpu;
            set { _gpu = value; ApplyOptions(); }
        }

        public string GpuReport
        {
            get
            {
                if (_handle == 0) return "off";
                long length = GpuReportExport(_handle, null, 0);
                var text = new byte[length];
                fixed (byte* data = text) GpuReportExport(_handle, data, (nuint)text.Length);
                return Encoding.UTF8.GetString(text);
            }
        }

        // Mars's ThreadedRdp: the display processor's lists on a thread of MarsRT's own, behind page marks - see Mars_Native.md §5.6.
        public bool ThreadedRdp
        {
            get => _threadedRdp;
            set { _threadedRdp = value; ApplyOptions(); }
        }

        // Mars's RdpWorkers: processors sharing a threaded list, each shading every Nth row.
        public int RdpWorkers
        {
            get => _rdpWorkers;
            set { _rdpWorkers = Math.Clamp(value, 1, 8); ApplyOptions(); }
        }

        // Mars's DeferredPresentation: the picture walked on another thread while the next frame runs, and shown a frame late.
        public bool DeferredPresentation
        {
            get => _deferredPresentation;
            set { _deferredPresentation = value; ApplyOptions(); }
        }

        public bool SkipRepeatedScans
        {
            get => _skipRepeatedScans;
            set { _skipRepeatedScans = value; ApplyOptions(); }
        }

        // Mars's UseBlocks: the processor's code compiled in blocks, validated against memory on entry - see Mars_Native.md §5.8.
        public bool UseBlocks
        {
            get => _useBlocks;
            set { _useBlocks = value; ApplyOptions(); }
        }

        // The interpreter run beside the blocks and compared after every instruction; for tests.
        public bool VerifyBlocks
        {
            get => _verifyBlocks;
            set { _verifyBlocks = value; ApplyOptions(); }
        }

        // Which step of the recompiler runs: 0 the furthest built, 1 decoded blocks, 2 compiled, 3 compiled with registers held.
        public int BlockTier
        {
            get => _blockTier;
            set { _blockTier = Math.Clamp(value, 0, 15); ApplyOptions(); }
        }

        // The recompiler's counters in mars_blocks_counters' order: live, shaped, discarded, entries, instructions, stepped, mapped, compiled, nanoseconds, bytes, compiled entries.
        public long[] BlockCounterValues()
        {
            long count = BlockCounters(Handle, null, 0);
            var values = new long[count];
            fixed (long* data = values) BlockCounters(Handle, data, (nuint)values.Length);
            return values;
        }

        // Every byte the display processor's thread touches checked against the marks, as Mars's EMUSEN_MARS_VERIFY_RDP; for tests.
        public bool VerifyRdp
        {
            get => _verifyRdp;
            set { _verifyRdp = value; ApplyOptions(); }
        }

        // The thread's counters in mars_threads_counters' order: running, its words, starts and time, the waits, bystanders, reads, repeated scans, and by site.
        public long[] ThreadCounterValues()
        {
            long count = ThreadCounters(Handle, null, 0);
            var values = new long[count];
            fixed (long* data = values) ThreadCounters(Handle, data, (nuint)values.Length);
            return values;
        }

        private static long FrameSerialOf(nint handle)
        {
            long* info = stackalloc long[4];
            FrameInfo(handle, info);
            return info[3];
        }

        // Declared before VideoSettings, whose initializer reads it; static fields initialise in textual order.
        private static readonly string[] Honoured = { "ExpansionPak", "ThreadedRdp", "RdpWorkers", "DeferredPresentation", "SkipRepeatedScans", "RenderScale", "Antialiasing", "Gpu" };

        // MarsRT's own key for Mars's UseBlocks, which is no setting of Mars's; before VideoSettings for the same reason - see Mars_Native.md §5.8.
        private static readonly CoreSetting Recompiler = new("Recompiler", "Compile the processor's code", "The processor's code is compiled to the host's machine code in blocks, each compared with memory before it runs, on a thread of its own. Exact: frame for frame the interpreter, which is what off leaves running, slower.", CoreSettingKind.Switch, "true");

        // Mars's keys, so one graphics tab serves either engine, every one honoured since stage D - see Mars_Native.md §5.5, §5.6 and §6.4.
        public static readonly IReadOnlyList<CoreSetting> VideoSettings =
            MarsCore.VideoSettings.Select(s => Honoured.Contains(s.Key) ? Threads(s) : s with { Hint = IgnoredHint + s.Hint }).Append(Recompiler).ToArray();

        private static CoreSetting Threads(CoreSetting s) => s.Key switch
        {
            "ThreadedRdp" => s with { Hint = "The display processor runs its lists on a thread of its own, behind marks on the memory it reaches. Exact: frame for frame the machine on one thread." },
            "RdpWorkers" => s with { Hint = "How many processors share each list when it runs on a thread of its own, each shading every Nth row. Exact at any count: frame for frame the machine on one thread. One per three cores is the default, as on Mars." },
            "DeferredPresentation" => s with { Hint = "The picture is finished on another thread while the machine runs the next frame, so it reaches the screen one frame late, exactly the picture it would have been." },
            _ => s,
        };

        private const string IgnoredHint = "MarsRT does not implement this yet and ignores it. On Mars (C#): ";

        private readonly Dictionary<string, string> _ignored = new();

        IReadOnlyList<CoreSetting> ICoreSettings.Settings => VideoSettings;

        public string Get(string key) => key switch
        {
            "ExpansionPak" => ExpansionPak ? "true" : "false",
            "ThreadedRdp" => ThreadedRdp ? "true" : "false",
            "RdpWorkers" => RdpWorkers.ToString(),
            "DeferredPresentation" => DeferredPresentation ? "true" : "false",
            "SkipRepeatedScans" => SkipRepeatedScans ? "true" : "false",
            "Recompiler" => UseBlocks ? "true" : "false",
            "RenderScale" => RenderScale.ToString(),
            "Antialiasing" => Antialiasing == 1 ? "Off" : $"{Antialiasing}x",
            "Gpu" => Gpu ? "true" : "false",
            _ => _ignored.TryGetValue(Setting(key).Key, out string? value) ? value : Setting(key).Default,
        };

        // Checked as its kind says, as MarsCore refuses text that is no value, and kept; the Expansion Pak and the thread switches act - see Mars_Native.md §5.5 and §5.6.
        public void Set(string key, string value)
        {
            CoreSetting setting = Setting(key);
            string accepted = setting.Kind switch
            {
                CoreSettingKind.Switch => bool.TryParse(value, out bool on) ? (on ? "true" : "false") : throw new ArgumentException($"{value} is not on or off."),
                CoreSettingKind.Count => int.TryParse(value, out int count) ? Math.Clamp(count, setting.Min, setting.Max).ToString() : throw new ArgumentException($"{value} is not a count."),
                _ => setting.Choices?.Contains(value) == true ? value : throw new ArgumentException($"{value} is not a choice of {setting.Label}."),
            };

            switch (key)
            {
                case "ExpansionPak": ExpansionPak = accepted == "true"; break;
                case "ThreadedRdp": ThreadedRdp = accepted == "true"; break;
                case "RdpWorkers": RdpWorkers = int.Parse(accepted); break;
                case "DeferredPresentation": DeferredPresentation = accepted == "true"; break;
                case "SkipRepeatedScans": SkipRepeatedScans = accepted == "true"; break;
                case "Recompiler": UseBlocks = accepted == "true"; break;
                case "RenderScale": RenderScale = int.Parse(accepted); break;
                case "Antialiasing": Antialiasing = accepted == "Off" ? 1 : accepted[0] - '0'; break;
                case "Gpu": Gpu = accepted == "true"; break;
                default: _ignored[key] = accepted; break;
            }
        }

        private static CoreSetting Setting(string key) =>
            VideoSettings.FirstOrDefault(s => s.Key == key) ?? throw new ArgumentException($"MarsRT has no setting named {key}.", nameof(key));

        private CheatRegistry _cheats = new();

        // Settable so the registry a frontend already fills becomes this core's own - see EmuSen_Cheats.md §6.
        public CheatRegistry Cheats
        {
            get => _cheats;
            set
            {
                _cheats = value;
                SyncRomPatches();
            }
        }

        private CheatRegistry? _patchesFrom;
        private int _patchesVersion;

        // The registry's ROM patches handed to the library whenever the registry or its version moved, before the machine runs - see Mars_Native.md §6.6.1.
        private void SyncRomPatches()
        {
            if (_handle == 0) return;
            CheatRegistry cheats = _cheats;
            int version = cheats.Version;
            if (ReferenceEquals(cheats, _patchesFrom) && version == _patchesVersion) return;
            IReadOnlyList<RomPatchByte> bytes = cheats.ResolveRomPatches(SpaceSize(MarsRtSpace.Rom));
            var words = new uint[bytes.Count * 3];
            for (int i = 0; i < bytes.Count; i++)
            {
                words[3 * i] = bytes[i].Address;
                words[3 * i + 1] = bytes[i].Value;
                words[3 * i + 2] = bytes[i].Compare is byte compare ? compare : uint.MaxValue;
            }
            fixed (uint* data = words) SetRomPatches(_handle, data, (nuint)bytes.Count);
            _patchesFrom = cheats;
            _patchesVersion = version;
        }

        // Public so a paused frontend need not wait for a frame boundary; ROM patches reach the cartridge at the next run - see Mars_Native.md §6.6.1.
        public void ApplyCheats()
        {
            if (_handle != 0) Cheats.ApplyAll(ReadForCheat, WriteForCheat);
        }

        // Held while interrupts are off, as MarsCore holds it - see Mars_Cheats.md §5.1.
        private void ApplyCheatsAtFrameEnd()
        {
            if ((Cop0(StatusRegister) & 1) != 0) ApplyCheats();
        }

        // Past the end reads zero and a write there is dropped, as MarsCore's are - see Mars_Cheats.md §3.1.
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
        public long SpaceSize(MarsRtSpace space) => _handle == 0 ? 0 : Math.Max(0, MemorySize(_handle, (uint)space));

        public byte Peek(MarsRtSpace space, uint address)
        {
            byte value = 0;
            if (_handle != 0) ReadMemory(_handle, (uint)space, address, &value, 1);
            return value;
        }

        public void Poke(MarsRtSpace space, uint address, byte value)
        {
            if (_handle != 0) WriteMemory(_handle, (uint)space, address, &value, 1);
        }

        // A copy of a stretch of memory, zero wherever the address names nothing.
        public byte[] ReadSpace(MarsRtSpace space, uint address, int length)
        {
            var bytes = new byte[length];
            if (_handle != 0 && length > 0) fixed (byte* data = bytes) ReadMemory(_handle, (uint)space, address, data, (nuint)length);
            return bytes;
        }

        public ulong Cop0(int register) => _handle == 0 ? 0 : Cop0Export(_handle, (uint)register);

        // PC, the 32 GPRs, HI, LO and the cycle count.
        public ulong[] CpuRegisters()
        {
            if (_handle == 0) return Array.Empty<ulong>();
            var values = new ulong[CpuRegistersExport(_handle, null, 0)];
            fixed (ulong* data = values) CpuRegistersExport(_handle, data, (nuint)values.Length);
            return values;
        }

        // PC, halted, broke, and the 32 scalar registers.
        public uint[] RspRegisters() => Words(RspRegistersExport);

        public uint[] ViRegisters() => Words(ViRegistersExport);

        private uint[] Words(delegate* unmanaged<nint, uint*, nuint, long> export)
        {
            if (_handle == 0) return Array.Empty<uint>();
            var values = new uint[export(_handle, null, 0)];
            fixed (uint* data = values) export(_handle, data, (nuint)values.Length);
            return values;
        }

        // The save chip's bytes as the game left them, or null for a cartridge with none.
        public byte[]? BatteryContents
        {
            get
            {
                int* info = stackalloc int[2];
                long length = SaveData(Handle, null, 0, info);
                if (length <= 0) return null;
                var contents = new byte[length];
                fixed (byte* data = contents) SaveData(_handle, data, (nuint)contents.Length, info);
                return contents;
            }
        }

        public void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            bool enabled = !(_batteryRamDisabled ?? CoreOptions.BatteryRamDisabled);
            _savePath = enabled ? SaveLibrary.SramPathFor(path) : null;
            _pakPath = enabled ? Path.ChangeExtension(_savePath!, MarsCore.PakExtension) : null;
            byte[]? saved = _savePath is null ? null : AtomicFile.TryRead(_savePath);
            byte[]? pak = _pakPath is null ? null : AtomicFile.TryRead(_pakPath);

            nint handle;
            fixed (byte* rom = image)
            fixed (byte* save = saved)
            fixed (byte* card = pak)
                handle = LoadRomExport(rom, (nuint)image.Length, ExpansionPak ? 1u : 0, saved is null ? null : save, (nuint)(saved?.Length ?? 0), pak is null ? null : card, (nuint)(pak?.Length ?? 0));
            if (handle == 0) throw new InvalidDataException($"MarsRT refused {path} as a Nintendo 64 image.");
            Replace(handle);
            _romPath = path;
        }

        // As the corpus boots a machine: the boot code handed off and nothing loaded beside it.
        public void Boot(byte[] image, bool expansionPak = true)
        {
            nint handle;
            fixed (byte* rom = image) handle = BootExport(rom, (nuint)image.Length, expansionPak ? 1u : 0);
            if (handle == 0) throw new InvalidDataException("MarsRT refused the image.");
            Replace(handle);
        }

        private void Replace(nint handle)
        {
            if (_handle != 0) Free(_handle);
            _handle = handle;
            CallStack.Reset();
            IsHaltedAtBreakpoint = false;
            ApplyOptions();
            _frame = Blank(MarsCore.DefaultScreenHeight);
            _screenWidth = MarsCore.ScreenWidthPixels;
            _screenHeight = MarsCore.DefaultScreenHeight;
            _rowRepeat = 1;
            _frameSerial++;
            _takenSerial = 0;
            _patchesFrom = null;
            SyncRomPatches();
        }

        public void SetButton(int port, PadButton button, bool pressed)
        {
            if (_handle == 0 || port < 0 || port >= 4) return;
            ushort mask = button switch
            {
                PadButton.A => ButtonA,
                PadButton.B => ButtonB,
                PadButton.Start => ButtonStart,
                PadButton.Up => DpadUp,
                PadButton.Down => DpadDown,
                PadButton.Left => DpadLeft,
                PadButton.Right => DpadRight,
                PadButton.L => ButtonL,
                PadButton.R => ButtonR,
                PadButton.L2 => ButtonZ,
                _ => 0,
            };
            if (mask != 0) PressExport(_handle, (uint)port, mask, pressed ? 1u : 0);
        }

        public void SetAxis(int port, PadAxis axis, double value)
        {
            if (_handle == 0 || port < 0 || port >= 4) return;
            value = Math.Clamp(value, -1.0, 1.0);
            switch (axis)
            {
                case PadAxis.LeftX: SetStick(_handle, (uint)port, 0, (sbyte)Math.Round(value * MarsCore.StickReach)); break;
                case PadAxis.LeftY: SetStick(_handle, (uint)port, 1, (sbyte)Math.Round(-value * MarsCore.StickReach)); break;
                case PadAxis.RightX:
                    PressExport(_handle, (uint)port, CLeft, value <= -MarsCore.CButtonThreshold ? 1u : 0);
                    PressExport(_handle, (uint)port, CRight, value >= MarsCore.CButtonThreshold ? 1u : 0);
                    break;
                case PadAxis.RightY:
                    PressExport(_handle, (uint)port, CUp, value <= -MarsCore.CButtonThreshold ? 1u : 0);
                    PressExport(_handle, (uint)port, CDown, value >= MarsCore.CButtonThreshold ? 1u : 0);
                    break;
            }
        }

        // MarsCore.RunFrame's order: the frame, the cheats, the frame log, the periodic save, then the picture - see Mars_Native.md §5.5 and §6.5.
        public void RunFrame()
        {
            nint handle = Handle;
            SyncRomPatches();
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;
            try
            {
                if (Observed) { if (!RunObserved(handle, resuming)) return; }
                else AdvanceExport(handle);
                ApplyCheatsAtFrameEnd();
                FrameLog.RecordFrame(TotalFrames, ReadWidth);
                Breakpoints.NoteFrame(TotalFrames);
                if (TotalFrames % MarsCore.SaveEveryNFrames == 0) SaveSram();
                if (_skipRendering) return;
                PresentExport(handle);
                TakePicture();
            }
            finally
            {
                ReadPhases(handle);
            }
        }

        // The library's phases for the frame just run, read once as it ends - see Mars_Native.md §6.6.2.
        private void ReadPhases(nint handle)
        {
            long* nanos = stackalloc long[5];
            FrameProfile(handle, nanos, 5);
            for (int p = 0; p < _phases.Length; p++) _phases[p] = (_phases[p].Name, nanos[p] / 1e6);
        }

        private readonly (string Name, double Milliseconds)[] _phases =
        {
            ("machine", 0), ("machine/rdp-wait", 0), ("present", 0), ("present/rdp-wait", 0), ("present/walk-join", 0),
        };

        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases => _phases;

        // The samples not yet drained, left for the frontend's drain, as Mars's Ai.Peek - see Mars_Native.md §6.6.2.
        public short[] PeekAudioSamples()
        {
            if (_handle == 0) return Array.Empty<short>();
            long count = PeekAudio(_handle, null, 0);
            var samples = new short[count];
            fixed (short* data = samples) PeekAudio(_handle, data, (nuint)samples.Length);
            return samples;
        }

        // The interpreter's steps, the given number, as MarsCorpusTests steps the C# core; through the blocks when they are on.
        public void RunSteps(ulong steps)
        {
            nint handle = Handle;
            SyncRomPatches();
            RunStepsExport(handle, steps);
        }

        public string IsViewerTranscript
        {
            get
            {
                long length = (long)IsViewerText(Handle, null, 0);
                var text = new byte[length];
                fixed (byte* data = text) IsViewerText(Handle, data, (nuint)text.Length);
                return Encoding.UTF8.GetString(text);
            }
        }

        // What the VI's scan composed, as MarsCore's Present composes it after every scan, walked or not.
        private void TakePicture()
        {
            long* info = stackalloc long[4];
            FrameInfo(_handle, info);
            if (info[3] == _takenSerial) return;
            _takenSerial = info[3];
            int width = (int)info[0], height = (int)info[1];
            long length = (long)FrameBytes(_handle, null, 0);
            if (width <= 0 || height <= 0 || length != (long)width * height * 4) return;
            if (_frame.Length != length) _frame = new byte[length];
            fixed (byte* data = _frame) FrameBytes(_handle, data, (nuint)_frame.Length);
            _screenWidth = width;
            _screenHeight = height;
            _rowRepeat = (int)Math.Max(1, info[2]);
            _frameSerial++;
        }

        // A copy in an array no one else holds: a returned one when there is one, else new - see Mars_Native.md §6.13.
        public byte[] GetFrameBufferRgba()
        {
            byte[] buffer = _lending.Lend(_frame.Length);
            _frame.AsSpan().CopyTo(buffer);
            return buffer;
        }

        public void ReturnFrameBuffer(byte[] buffer) => _lending.Return(buffer);

        private readonly FrameBufferLending _lending = new();

        // What the lending has done, for the tests and the probe.
        public FrameBufferLending FrameBuffers => _lending;

        public short[] DequeueAudioSamples(int maxFrames)
        {
            if (_handle == 0) return Array.Empty<short>();
            long wanted = Math.Min((long)maxFrames * 2, (long)AudioBuffered(_handle));
            wanted -= wanted & 1;
            if (wanted <= 0) return Array.Empty<short>();
            var samples = new short[wanted];
            fixed (short* data = samples) DrainAudio(_handle, data, (ulong)(wanted / 2));
            return samples;
        }

        public void SaveSram()
        {
            if (_handle == 0) return;
            int* info = stackalloc int[2];
            long length = SaveData(_handle, null, 0, info);
            if (_savePath != null && info[1] != 0 && length > 0)
            {
                var contents = new byte[length];
                fixed (byte* data = contents) SaveData(_handle, data, (nuint)contents.Length, info);
                AtomicFile.Write(_savePath, contents);
                MarkSaved(_handle, 1);
            }

            int dirty = 0;
            long pakLength = PakData(_handle, null, 0, &dirty);
            if (_pakPath != null && dirty != 0 && pakLength > 0)
            {
                var pak = new byte[pakLength];
                fixed (byte* data = pak) PakData(_handle, data, (nuint)pak.Length, &dirty);
                AtomicFile.Write(_pakPath, pak);
                MarkSaved(_handle, 2);
            }
        }

        public void SaveState(string path)
        {
            byte[] state = Save(snapshot: false);
            File.WriteAllBytes(path, state);
        }

        public void LoadState(string path) => LoadState(File.ReadAllBytes(path));

        public void SaveState(Stream stream) => stream.Write(Save(snapshot: false));

        // Written through an array kept for the purpose, so the rewind's capture every few frames allocates no state - see Mars_Native.md §6.6.3.
        public void SaveSnapshot(Stream stream)
        {
            long size = SizeOf(Handle, 1);
            if (size < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(size)}.");
            if (_snapshotBytes.Length != size) _snapshotBytes = new byte[size];
            long written;
            fixed (byte* data = _snapshotBytes) written = SaveStateExport(_handle, data, (nuint)size, 1);
            if (written < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(written)}.");
            stream.Write(_snapshotBytes, 0, (int)written);
        }

        private byte[] _snapshotBytes = Array.Empty<byte>(), _loadBytes = Array.Empty<byte>();

        // A stream that can say its length is read into an array kept for the purpose, as a step back is - see Mars_Native.md §6.6.3.
        public void LoadState(Stream stream)
        {
            if (!stream.CanSeek)
            {
                using var copy = new MemoryStream();
                stream.CopyTo(copy);
                LoadState(copy.ToArray());
                return;
            }
            int length = checked((int)(stream.Length - stream.Position));
            if (_loadBytes.Length < length) _loadBytes = new byte[length];
            stream.ReadExactly(_loadBytes, 0, length);
            LoadState(_loadBytes.AsSpan(0, length));
        }

        // The C# Mars's format byte for byte; a state of the other memory size rebuilds the machine and the Pak follows it, as MarsCore's does.
        public void LoadState(ReadOnlySpan<byte> state)
        {
            nint handle = Handle;
            uint before = RdramBytes(handle);
            int status;
            fixed (byte* data = state) status = RestoreState(handle, data, (nuint)state.Length);
            if (status != 0) throw new InvalidDataException($"MarsRT refused the state: {MarsMachine.Describe(status)}.");
            uint after = RdramBytes(handle);
            if (after != before) _expansionPak = after == ExpandedRdram;
            if (!_skipRendering) TakePicture();
        }

        public byte[] Save(bool snapshot)
        {
            long size = SizeOf(Handle, snapshot ? 1u : 0);
            if (size < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(size)}.");
            var state = new byte[size];
            long written;
            fixed (byte* data = state) written = SaveStateExport(Handle, data, (nuint)state.Length, snapshot ? 1u : 0);
            if (written < 0) throw new InvalidDataException($"MarsRT could not write the state: {MarsMachine.Describe(written)}.");
            return state;
        }

        // The CPU's fields alone, as StateSerializer writes a Cpu, for a comparison made every few instructions.
        public byte[] SaveProcessor()
        {
            var bytes = new byte[SaveCpu(Handle, null, 0)];
            fixed (byte* data = bytes) SaveCpu(Handle, data, (nuint)bytes.Length);
            return bytes;
        }

        private long Counter(int index)
        {
            long* values = stackalloc long[6];
            Counters(Handle, values);
            return values[index];
        }

        private static byte[] Blank(int height)
        {
            var frame = new byte[MarsCore.ScreenWidthPixels * height * 4];
            for (int alpha = 3; alpha < frame.Length; alpha += 4) frame[alpha] = 0xFF;
            return frame;
        }

        private nint Handle => _handle != 0 ? _handle : throw new InvalidOperationException("MarsRT has no ROM loaded.");

        public void Dispose()
        {
            if (_handle != 0) Free(_handle);
            _handle = 0;
            _lending.Close();
            GC.SuppressFinalize(this);
        }

        ~MarsRtCore()
        {
            if (_handle != 0) Free(_handle);
        }
    }
}
