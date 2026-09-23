using System;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Mars.Debug;
using EmuSen.Cores.Nintendo.Mars.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Cores.Nintendo.Mars.Rom;
using EmuSen.Cores.Nintendo.Mars.Vi;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Library;
using VideoInterface = EmuSen.Cores.Nintendo.Mars.Vi.Vi;

namespace EmuSen.Cores.Nintendo.Mars
{
    // The Nintendo 64's ICore; what the machine cannot provide yet is stubbed on purpose - see Mars_Core.md.
    public sealed partial class MarsCore : global::EmuSen.Cores.ICore, global::EmuSen.Cores.ISnapshotCore, global::EmuSen.Cores.ICoreSettings, global::EmuSen.Cores.IFrameSerial, global::EmuSen.Cores.IRepeatedRows, global::EmuSen.Cores.IStateFormat
    {
        // The VR4300's pipeline clock, which is what MemoryBus.Cycles counts - see Mars_Memory.md §3.
        public const long ProcessorClockHz = 93_750_000;

        // A little over the longest field the VI's registers can describe, so it only ends a frame the VI never would - see Mars_Core.md §3.
        public const long CycleCap = 4_100_000;

        public const int ScreenWidthPixels = VideoInterface.RasterWidth;

        // An NTSC picture, which is what the VI describes before a game programs it - see Mars_Core.md §2.
        public const int DefaultScreenHeight = 480;

        // Where each PadButton lands in the joybus's sixteen button bits - see Mars_Serial.md §3.1.
        private const ushort ButtonA = 0x8000, ButtonB = 0x4000, ButtonZ = 0x2000, ButtonStart = 0x1000;
        private const ushort DpadUp = 0x0800, DpadDown = 0x0400, DpadLeft = 0x0200, DpadRight = 0x0100;
        private const ushort ButtonL = 0x0020, ButtonR = 0x0010;
        private const ushort CUp = 0x0008, CDown = 0x0004, CLeft = 0x0002, CRight = 0x0001;

        // A stick all the way over reads the signed byte's full reach, as Project64's input plugin scales it - see Mars_Core.md §5.
        public const int StickReach = 127;

        // The right stick stands in for the four C buttons, pressed past half its travel - see Mars_Core.md §5.
        public const double CButtonThreshold = 0.5;

        // "MARS" little-endian, then the format version - see Mars_SaveStates.md §1.
        private const uint StateMagic = 0x5352_414D;
        private const int StateVersion = 1;
        int global::EmuSen.Cores.IStateFormat.StateVersion => StateVersion;

        // The same body, then the words the display processor's thread had not run - see Mars_SaveStates.md §1.
        private const int SnapshotVersion = 2;

        // How often a changed save is written without being asked, as the other cores do - see Mars_Save.md §7.
        public const int SaveEveryNFrames = 300;

        // The Controller Pak's own file beside the cartridge's - see Mars_Save.md §7.
        public const string PakExtension = ".mpk";

        private byte[] _frame = Blank(DefaultScreenHeight);
        private string? _savePath;
        private string? _pakPath;
        private int _screenHeight = DefaultScreenHeight;

        // The picture a deferred walk composes while the next frame runs, shown once the walk is joined - see Mars_Video.md §2.7.
        private byte[] _pendingFrame = Blank(DefaultScreenHeight);
        private int _pendingHeight = DefaultScreenHeight;
        private readonly ScanJob _scan = new();
        private ManualResetEventSlim? _presenting;
        private Exception? _presentationFault;
        private bool _deferred;
        private bool _threadedRdp;
        private int _rdpWorkers = 1;
        private long _lastFrameCycles = CycleCap;

        // A stock console unless asked; the factory asks, so a frontend's machine has the Pak - see Mars_Core.md §7.
        public MarsCore(bool expansionPak = false, bool? batteryRamDisabled = null)
        {
            _expansionPak = expansionPak;
            _batteryRamDisabled = batteryRamDisabled;

            // The seams `bt`, `step over`/`out` and `cov` read, as the other cores wire them - see Mars_Debug.md §2.
            Breakpoints.CallStack = CallStack;
            CallStack.FrameNumberProvider = () => TotalFrames;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

        // Owned here rather than by the debug target, so a breakpoint or a label outlives any one prompt - see Mars_Debug.md §1.
        public WatchRegistry Watches { get; } = new();
        public FrameLogRegistry FrameLog { get; } = new();
        public BreakpointRegistry Breakpoints { get; } = new();
        public CoverageRegistry Coverage { get; } = new();
        public CoverageRegistry RspCoverage { get; } = new();
        public CallStackRegistry CallStack { get; } = new();
        public LabelRegistry Labels { get; } = new();

        // Handed to every bus a load builds, so a watch survives a reload - see Mars_Debug.md §3.
        public IWriteObserver? WriteObserver
        {
            get => _writeObserver;
            set
            {
                _writeObserver = value;
                if (Bus != null) Bus.WriteObserver = value;
            }
        }

        private IWriteObserver? _writeObserver;

        public bool IsHaltedAtBreakpoint { get; private set; }
        public int HaltedAddress { get; private set; }

        // Null defers to --nobattery; a test passes its own so no other test's switch can reach it - see Mars_Save.md §7.
        private readonly bool? _batteryRamDisabled;

        // The Pak the next load builds; before the first frame a change rebuilds the machine at once - see Mars_Core.md §7.
        public bool ExpansionPak
        {
            get => _expansionPak;
            set
            {
                if (value == _expansionPak) return;
                _expansionPak = value;
                if (Bus is not null && TotalFrames == 0 && _romPath is { } path) LoadRom(path);
            }
        }

        private bool _expansionPak;
        private string? _romPath;

        public RomImage? Rom { get; private set; }
        public MemoryBus? Bus { get; private set; }
        public Cpu.Core.Cpu? Cpu { get; private set; }

        public string CoreName => "N64";

        public int ScreenWidth => _frameWidth;

        // The picture's width, which a walk at a multiple multiplies - see Mars_Video.md §2.9.
        [EmuSen.Common.SkipInState] private int _frameWidth = ScreenWidthPixels;

        // Advanced wherever the shown frame is replaced, and nowhere else - see EmuSen_Multicore.md §14.
        [EmuSen.Common.SkipInState] private long _frameSerial;

        public long FrameSerial => _frameSerial;

        // A progressive field's rows once each, for a frontend that stretches them itself; the frame's own repeat travels with it - see EmuSen_Multicore.md §15.
        public bool RepeatRows { get; set; } = true;

        [EmuSen.Common.SkipInState] private int _rowRepeat = 1, _pendingRowRepeat = 1;

        public int RowRepeat => _rowRepeat;
        [EmuSen.Common.SkipInState] private int _pendingWidth = ScreenWidthPixels;
        private int _renderScale = 1;

        // The multiple the display processor draws the picture at beside the machine's own, exact in memory and only the picture grows - see Mars_Rdp.md §11.
        public int RenderScale
        {
            get => _renderScale;
            set
            {
                _renderScale = Math.Clamp(value, 1, 4);
                ApplyMultiple();
            }
        }

        private int _antialiasing = 1;

        // How many times finer each way the picture is drawn than it is shown, and averaged down - see Mars_Video.md §2.10.
        public int Antialiasing
        {
            get => _antialiasing;
            set
            {
                _antialiasing = Math.Clamp(value, 1, 4);
                ApplyMultiple();
            }
        }

        // The drawing is the resolution times the averaging, held to four: the averaging gives way first - see Mars_Video.md §2.10.
        public int EffectiveAntialiasing => Math.Max(1, Math.Min(_antialiasing, 4 / _renderScale));

        private bool _gpu;

        // Shade the multiple on a compute device instead of the processor's own threads; the machine's picture is never the device's - see Mars_Gpu.md §11.
        public bool Gpu
        {
            get => _gpu;
            set
            {
                _gpu = value;
                ApplyMultiple();
            }
        }

        // What the device setting actually got, which is a sentence when it got nothing.
        public string GpuReport => Bus?.Dp.GpuReport ?? "off";

        private void ApplyMultiple()
        {
            if (Bus is not { } bus) return;

            // A walk still out reads the device's picture where the device left it, so the device outlives it - see Mars_Gpu.md §14.
            JoinPresentation();
            bus.Dp.Scale = _renderScale * EffectiveAntialiasing;
            bus.Dp.Gpu = _gpu;
            bus.Vi.Average = EffectiveAntialiasing;
        }

        // Set with the buffer it describes, so the two cannot disagree between frames - see Mars_Core.md §2.
        public int ScreenHeight => _screenHeight;

        // The rate of the frames RunFrame actually produced, whichever boundary ended them - see Mars_Core.md §3.
        public double FrameRateHz => ProcessorClockHz / (double)_lastFrameCycles;

        public bool IsRomLoaded => Bus != null;
        public long TotalFrames { get; private set; }

        public bool SkipRendering { get; set; }

        // The rate the game set the DAC to, which a frontend's audio sink follows when it changes - see Mars_Core.md §4.
        public int AudioSampleRate => Bus?.Ai.SampleRate ?? AiInterface.DefaultSampleRate;

        // The pad's buttons on the generic template, with L2 as Z; the C buttons arrive on the right stick - see Mars_Core.md §5.
        public static IReadOnlyList<PadButton> PadButtons { get; } = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.B, PadButton.Start, PadButton.L, PadButton.R, PadButton.L2,
        };

        public IReadOnlyList<PadButton> SupportedButtons => PadButtons;

        public static IReadOnlyList<PadAxis> PadAxes { get; } = new[] { PadAxis.LeftX, PadAxis.LeftY, PadAxis.RightX, PadAxis.RightY };

        public IReadOnlyList<PadAxis> SupportedAxes => PadAxes;

        public void LoadRom(string path)
        {
            var rom = RomImage.Load(path);

            // A machine being replaced gives up its threads first - see Mars_Core.md §7.
            JoinPresentation();
            if (Bus is { } replaced) replaced.Dp.Threaded = false;
            _romPath = path;

            var bus = new MemoryBus(ExpansionPak);
            bus.RomPatcher = new global::EmuSen.Cores.CheatRomPatcher(Cheats);
            var cpu = new Cpu.Core.Cpu(bus);
            Boot.HandOff(bus, cpu, rom);
            LoadSaves(bus, rom, path);

            cpu.CallObserver = (source, target) => CallStack.NotePush((int)(uint)source, (int)(uint)target, CallFrameKind.Call);
            cpu.ReturnObserver = CallStack.NotePop;
            cpu.InterruptObserver = () => Breakpoints.NoteInterrupt(CallFrameKind.Irq);
            bus.Sp.Processor.Coverage = RspCoverage;
            bus.WriteObserver = _writeObserver;
            CallStack.Reset();
            IsHaltedAtBreakpoint = false;

            Rom = rom;
            Bus = bus;
            Cpu = cpu;
            bus.Dp.Threaded = _threadedRdp;
            bus.Dp.Workers = _rdpWorkers;
            ApplyMultiple();

            TotalFrames = 0;
            _lastFrameCycles = CycleCap;
            _screenHeight = DefaultScreenHeight;
            _frame = Blank(DefaultScreenHeight);
            _frameSerial++;
            _pendingFrame = Blank(DefaultScreenHeight);
            _pendingHeight = DefaultScreenHeight;
            _rowRepeat = _pendingRowRepeat = 1;
        }

        // A binding for a button the pad lacks is dropped rather than moved onto another one - see Mars_Core.md §5.
        public void SetButton(int port, PadButton button, bool pressed)
        {
            Controller[]? ports = Bus?.Si.Controllers;
            if (ports is null || port < 0 || port >= ports.Length) return;

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

            if (mask == 0) return;
            Press(ports[port], mask, pressed);
        }

        // The left stick is the stick, turned over because the N64's up is positive; the right stick is the C buttons - see Mars_Core.md §5.
        public void SetAxis(int port, PadAxis axis, double value)
        {
            Controller[]? ports = Bus?.Si.Controllers;
            if (ports is null || port < 0 || port >= ports.Length) return;

            Controller controller = ports[port];
            value = Math.Clamp(value, -1.0, 1.0);

            switch (axis)
            {
                case PadAxis.LeftX: controller.StickX = (sbyte)Math.Round(value * StickReach); break;
                case PadAxis.LeftY: controller.StickY = (sbyte)Math.Round(-value * StickReach); break;
                case PadAxis.RightX:
                    Press(controller, CLeft, value <= -CButtonThreshold);
                    Press(controller, CRight, value >= CButtonThreshold);
                    break;
                case PadAxis.RightY:
                    Press(controller, CUp, value <= -CButtonThreshold);
                    Press(controller, CDown, value >= CButtonThreshold);
                    break;
            }
        }

        private static void Press(Controller controller, ushort mask, bool pressed) =>
            controller.Buttons = pressed ? (ushort)(controller.Buttons | mask) : (ushort)(controller.Buttons & ~mask);

        public void RunFrame()
        {
            if (Bus is null || Cpu is null)
            {
                throw new InvalidOperationException("RunFrame() called before LoadRom().");
            }

            // The instruction a halt stopped in front of runs before anything is checked again - see Mars_Debug.md §2.
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;

            long start = Bus.Cycles;
            long fields = Bus.Vi.Fields;

            // Nothing armed can halt or record anything this frame, so the loop is the processor alone - see Mars_Performance.md §13.
            if (Breakpoints.IsQuiet && !Coverage.IsArmed && !CallStack.IsProfiling) RunQuietly(Bus, Cpu, start, fields, UseBlocks);

            // The VI's field is the frame; the cap only ends one a VI nobody has programmed never will - see Mars_Core.md §3.
            while (Bus.Vi.Fields == fields && Bus.Cycles - start < CycleCap)
            {
                int pc = (int)(uint)Cpu.Pc;

                // Before the instruction, so a breakpoint stops in front of what it names - see Mars_Debug.md §2.
                if (!resuming && Breakpoints.CouldBreak && Breakpoints.ShouldBreak(pc))
                {
                    IsHaltedAtBreakpoint = true;
                    HaltedAddress = pc;
                    return;
                }

                resuming = false;
                if (Coverage.IsArmed) Coverage.Record(pc);
                if (CallStack.IsProfiling) CallStack.NoteInstruction();

                Cpu.Step();
            }

            _lastFrameCycles = Bus.Cycles - start;
            TotalFrames++;
            ApplyCheatsAtFrameEnd();

            FrameLog.RecordFrame(TotalFrames, (space, address, width) => MarsDebugSpaces.ReadWidth(this, space, address, width));
            Breakpoints.NoteFrame(TotalFrames);

            if (TotalFrames % SaveEveryNFrames == 0) SaveSram();

            if (SkipRendering) return;
            if (_deferred) PresentDeferred(Bus.Vi);
            else Present(Bus.Vi);
        }

        // The display processor's lists run on a pool thread behind marks on the pages they reach - see Mars_Rdp.md §2.6.
        public bool ThreadedRdp
        {
            get => Bus?.Dp.Threaded ?? _threadedRdp;
            set
            {
                _threadedRdp = value;
                if (Bus is { } bus) bus.Dp.Threaded = value;
            }
        }

        // The settings a frontend can offer, with the defaults Mistress used to set by hand - see Mars_Core.md §10.
        public static readonly IReadOnlyList<global::EmuSen.Cores.CoreSetting> VideoSettings = new global::EmuSen.Cores.CoreSetting[]
        {
            new("ThreadedRdp", "Draw on a separate thread", "The display processor runs its lists on a thread of its own, behind marks on the memory it reaches. Exact; faster on any machine with two cores to spare.", global::EmuSen.Cores.CoreSettingKind.Switch, "true"),
            new("RdpWorkers", "Rasteriser threads", "How many processors share each list, each shading every Nth row. Exact at any count; past two, each adds less. One per three cores is the default.", global::EmuSen.Cores.CoreSettingKind.Count, Math.Clamp(Environment.ProcessorCount / 3, 1, 4).ToString(), 1, 8),
            new("DeferredPresentation", "Scan out while the next frame runs", "The picture is finished on another thread while the machine runs the next frame, so it reaches the screen one frame late. Off, the frame waits for its picture.", global::EmuSen.Cores.CoreSettingKind.Switch, "true"),
            new("SkipRepeatedScans", "Skip a scan that repeats the last", "A scan whose registers and bytes match the last walk is not walked again. Exact; the picture is the same either way.", global::EmuSen.Cores.CoreSettingKind.Switch, "true"),
            new("RenderScale", "Internal resolution", "The picture drawn at a multiple of the console's, beside the exact drawing games read back. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen. Threads help; 2x is what most machines can hold at full speed.", global::EmuSen.Cores.CoreSettingKind.Choice, "1", Choices: new[] { "1", "2", "3", "4" }),
            new("Antialiasing", "Antialiasing", "Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. It multiplies the drawing's cost like the internal resolution, and the two together are held to four: at 2x resolution the most is 2x.", global::EmuSen.Cores.CoreSettingKind.Choice, "Off", Choices: new[] { "Off", "2x", "3x", "4x" }),
            new("Gpu", "Draw the multiple on the graphics card", "The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's.", global::EmuSen.Cores.CoreSettingKind.Switch, "false"),
            new("ExpansionPak", "Expansion Pak", "The memory accessory that doubles the console's 4MB. A few games refuse to start without it and more use it when it is there. Takes effect when a game is next loaded; a save state resumes with the memory it was made with.", global::EmuSen.Cores.CoreSettingKind.Switch, "true"),
        };

        IReadOnlyList<global::EmuSen.Cores.CoreSetting> global::EmuSen.Cores.ICoreSettings.Settings => VideoSettings;

        public string Get(string key) => key switch
        {
            "ThreadedRdp" => ThreadedRdp ? "true" : "false",
            "RdpWorkers" => RdpWorkers.ToString(),
            "DeferredPresentation" => DeferredPresentation ? "true" : "false",
            "SkipRepeatedScans" => SkipRepeatedScans ? "true" : "false",
            "RenderScale" => RenderScale.ToString(),
            "Antialiasing" => Antialiasing == 1 ? "Off" : $"{Antialiasing}x",
            "Gpu" => Gpu ? "true" : "false",
            "ExpansionPak" => ExpansionPak ? "true" : "false",
            _ => throw new ArgumentException($"Mars has no setting named {key}.", nameof(key)),
        };

        // A count outside its range is clamped and a switch is read as a boolean; text that is neither is refused - see Mars_Core.md §10.
        public void Set(string key, string value)
        {
            switch (key)
            {
                case "ThreadedRdp": ThreadedRdp = Switch(value); break;
                case "RdpWorkers": RdpWorkers = Math.Clamp(Count(value), 1, 8); break;
                case "DeferredPresentation": DeferredPresentation = Switch(value); break;
                case "SkipRepeatedScans": SkipRepeatedScans = Switch(value); break;
                case "RenderScale": RenderScale = Math.Clamp(Count(value), 1, 4); break;
                case "Antialiasing": Antialiasing = value == "Off" ? 1 : value is "2x" or "3x" or "4x" ? value[0] - '0' : throw new ArgumentException($"{value} is not a level of antialiasing."); break;
                case "Gpu": Gpu = Switch(value); break;
                case "ExpansionPak": ExpansionPak = Switch(value); break;
                default: throw new ArgumentException($"Mars has no setting named {key}.", nameof(key));
            }

            static bool Switch(string text) => bool.TryParse(text, out bool on) ? on : throw new ArgumentException($"{text} is not on or off.");
            static int Count(string text) => int.TryParse(text, out int count) ? count : throw new ArgumentException($"{text} is not a count.");
        }

        // How many processors share the display processor's list, each shading its own rows, when the list runs threaded - see Mars_Rdp.md §2.8.
        public int RdpWorkers
        {
            get => Bus?.Dp.Workers ?? _rdpWorkers;
            set
            {
                _rdpWorkers = value;
                if (Bus is { } bus) bus.Dp.Workers = value;
            }
        }

        // The scan-out walks on another thread while the next frame runs, and the picture shown is the last one joined - see Mars_Video.md §2.7.
        // A deferred scan whose registers and bytes repeat the last walk's is skipped; the counter is what a bench reads - see Mars_Video.md §2.8.
        public bool SkipRepeatedScans { get; set; } = true;

        [EmuSen.Common.SkipInState] public long RepeatedScans;

        public bool DeferredPresentation
        {
            get => _deferred;
            set
            {
                JoinPresentation();
                _deferred = value;
            }
        }

        // Compiled blocks between the checks the frame makes; the interpreter alone when switched off, or where no code can be emitted - see Mars_Recompiler.md §3.
        public bool UseBlocks { get; set; } = System.Runtime.CompilerServices.RuntimeFeature.IsDynamicCodeSupported;

        // The same frame end as RunFrame's own loop, with no debugger between the instructions - see Mars_Performance.md §13.
        private static void RunQuietly(MemoryBus bus, global::EmuSen.Cores.Nintendo.Mars.Cpu.Core.Cpu cpu, long start, long fields, bool blocks)
        {
            long capAt = start + CycleCap;

            if (blocks) while (bus.Vi.Fields == fields && bus.Cycles < capAt) cpu.StepBlock(capAt, fields);
            else while (bus.Vi.Fields == fields && bus.Cycles < capAt) cpu.Step();
        }

        public byte[] GetFrameBufferRgba() => _frame;

        public short[] DequeueAudioSamples(int maxFrames) => Bus?.Ai.Drain(maxFrames) ?? Array.Empty<short>();

        // Whatever the game changed since the last write, and nothing it did not - see Mars_Save.md §7.
        public void SaveSram()
        {
            if (Bus is null) return;

            SaveChip chip = Bus.Save;
            if (_savePath != null && chip.Dirty && chip.Contents is { } contents)
            {
                AtomicFile.Write(_savePath, contents);
                chip.Saved();
            }

            ControllerPak? pak = Bus.Si.Controllers[0].Pak;
            if (_pakPath != null && pak is { Dirty: true })
            {
                AtomicFile.Write(_pakPath, pak.Data);
                pak.Dirty = false;
            }
        }

        // The image's word, the table, the last save's length, and otherwise the game's first move - see Mars_Save.md §1.
        private void LoadSaves(MemoryBus bus, RomImage rom, string romPath)
        {
            // Latched here, as a cartridge does, so --nobattery holds for the whole run - see Mars_Save.md §7.
            bool enabled = !(_batteryRamDisabled ?? CoreOptions.BatteryRamDisabled);
            _savePath = enabled ? SaveLibrary.SramPathFor(romPath) : null;
            _pakPath = enabled ? Path.ChangeExtension(_savePath!, PakExtension) : null;

            byte[]? saved = Read(_savePath);
            N64SaveType type = SaveTypes.Declared(rom);
            if (type == N64SaveType.Unknown && saved != null) type = SaveChip.FromSaveLength(saved.Length);

            bus.Save = new SaveChip(type, saved);
            bus.Si.Controllers[0].Pak = new ControllerPak(Read(_pakPath));
        }

        private static byte[]? Read(string? path) => path is null ? null : AtomicFile.TryRead(path);

        // Refused before any file exists, so a failed save leaves no empty state behind - see Mars_SaveStates.md §1.
        public void SaveState(string path)
        {
            RequireRom(nameof(SaveState));

            using var stream = File.Create(path);
            SaveState(stream);
        }

        public void LoadState(string path)
        {
            RequireRom(nameof(LoadState));

            using var stream = File.OpenRead(path);
            LoadState(stream);
        }

        // A header, the processor, then the bus and everything it owns - see Mars_SaveStates.md §1.
        public void SaveState(Stream stream) => Write(stream, snapshot: false);

        // Written without waiting for the display processor's thread, for the rewind buffer - see Mars_SaveStates.md §1.
        public void SaveSnapshot(Stream stream) => Write(stream, snapshot: true);

        private void Write(Stream stream, bool snapshot)
        {
            RequireRom(nameof(SaveState));

            using var w = new BinaryWriter(stream, System.Text.Encoding.UTF8, leaveOpen: true);
            w.Write(StateMagic);
            w.Write(snapshot ? SnapshotVersion : StateVersion);
            w.Write(Bus!.Rdram.Length);
            w.Write(TotalFrames);
            w.Write(_lastFrameCycles);

            StateSerializer.Write(w, Cpu!);
            Bus.WriteState(w, snapshot);
        }

        // A state made with the other amount of memory rebuilds the machine to it; any other size is refused before anything is read - see Mars_SaveStates.md §1.
        public void LoadState(Stream stream)
        {
            RequireRom(nameof(LoadState));

            using var r = new BinaryReader(stream, System.Text.Encoding.UTF8, leaveOpen: true);
            if (r.ReadUInt32() != StateMagic) throw new InvalidDataException("Not a Mars save state.");

            int version = r.ReadInt32();
            if (version != StateVersion && version != SnapshotVersion) throw new InvalidDataException($"Mars save state version {version} is not {StateVersion}.");

            int rdram = r.ReadInt32();
            if (rdram != Bus!.Rdram.Length)
            {
                bool known = rdram == MemoryBus.RdramSize || rdram == MemoryBus.RdramSizeExpanded;
                if (!known || _romPath is null) throw new InvalidDataException($"The state was saved with {rdram / (1024 * 1024)}MB of RDRAM and this machine has {Bus.Rdram.Length / (1024 * 1024)}MB.");
                _expansionPak = rdram == MemoryBus.RdramSizeExpanded;
                LoadRom(_romPath);
            }

            TotalFrames = r.ReadInt64();
            _lastFrameCycles = r.ReadInt64();

            StateSerializer.Read(r, Cpu!);
            Bus.ReadState(r, snapshot: version == SnapshotVersion);

            // The timer's due cycle and the interrupt check are derived from what was just read - see Mars_Performance.md §10.
            Cpu.Cop0Written();
            _scan.Forget();

            if (!SkipRendering) Present(Bus.Vi);
        }

        private void Present(VideoInterface vi)
        {
            JoinPresentation();
            vi.Scan();
            vi.RasterEdited = false;
            _frameWidth = Compose(vi, vi.FrameHeight, vi.Serrate, RepeatRows, ref _frame);
            _frameSerial++;
            _rowRepeat = vi.Serrate || RepeatRows ? 1 : 2;
            _screenHeight = _frame.Length / (_frameWidth * 4);
        }

        // The walk and the composition go to the pool; what they read was captured, what they write is the pending picture - see Mars_Video.md §2.7.
        private void PresentDeferred(VideoInterface vi)
        {
            JoinPresentation();

            bool walk = vi.Prepare(_scan);
            if (walk) vi.Capture(_scan, walkRepeats: !SkipRepeatedScans || vi.RasterEdited);

            // A scan that would write the raster already there is not walked, and the picture on show is already it unless the raster was edited since - see Mars_Video.md §2.8.
            if (walk && _scan.Repeats && SkipRepeatedScans && !vi.RasterEdited)
            {
                RepeatedScans++;
                return;
            }

            vi.RasterEdited = false;

            int rows = vi.FrameHeight;
            bool serrate = vi.Serrate, repeatRows = RepeatRows;

            var done = new ManualResetEventSlim(false);
            _presenting = done;
            ThreadPool.UnsafeQueueUserWorkItem(_ =>
            {
                try
                {
                    if (walk) vi.Walk(_scan);
                    _pendingWidth = Compose(vi, rows, serrate, repeatRows, ref _pendingFrame);
                    _pendingHeight = rows * (serrate || !repeatRows ? 1 : 2) * vi.OutputScale;
                    _pendingRowRepeat = serrate || repeatRows ? 1 : 2;
                }
                catch (Exception fault)
                {
                    _presentationFault = fault;
                }
                finally
                {
                    done.Set();
                }
            }, null);
        }

        // Waits for the walk that is out, if one is, and makes its picture the shown one - see Mars_Video.md §2.7.
        private void JoinPresentation()
        {
            ManualResetEventSlim? presenting = _presenting;
            if (presenting is null) return;

            presenting.Wait();
            presenting.Dispose();
            _presenting = null;

            if (_presentationFault is { } fault)
            {
                _presentationFault = null;
                throw new InvalidOperationException("The deferred scan-out failed.", fault);
            }

            (_frame, _pendingFrame) = (_pendingFrame, _frame);
            _frameSerial++;
            _screenHeight = _pendingHeight;
            _frameWidth = _pendingWidth;
            _rowRepeat = _pendingRowRepeat;
        }

        // The raster's fourth byte is coverage, not opacity, and a progressive field is every other line - see Mars_Core.md §2.
        private static int Compose(VideoInterface vi, int rows, bool serrate, bool repeatRows, ref byte[] frame)
        {
            ReadOnlySpan<byte> raster = vi.Raster(rows);
            int repeat = serrate || !repeatRows ? 1 : 2;
            int width = vi.OutputWidth;
            int rowBytes = width * 4;
            rows *= vi.OutputScale;

            if (frame.Length != rowBytes * rows * repeat) frame = new byte[rowBytes * rows * repeat];

            for (int row = 0; row < rows; row++)
            {
                Span<byte> line = frame.AsSpan(row * repeat * rowBytes, rowBytes);
                Opaque(raster.Slice(row * rowBytes, rowBytes), line);
                for (int copy = 1; copy < repeat; copy++) line.CopyTo(frame.AsSpan((row * repeat + copy) * rowBytes, rowBytes));
            }

            return width;
        }

        // Each pixel copied with its fourth byte made opaque, a vector at a time - see Mars_Gpu.md §14.5.
        private static void Opaque(ReadOnlySpan<byte> source, Span<byte> into)
        {
            ReadOnlySpan<uint> from = System.Runtime.InteropServices.MemoryMarshal.Cast<byte, uint>(source);
            Span<uint> to = System.Runtime.InteropServices.MemoryMarshal.Cast<byte, uint>(into);
            uint alpha = BitConverter.IsLittleEndian ? 0xFF00_0000u : 0xFFu;
            int i = 0;

            if (System.Numerics.Vector.IsHardwareAccelerated)
            {
                var alphas = new System.Numerics.Vector<uint>(alpha);
                for (; i <= from.Length - System.Numerics.Vector<uint>.Count; i += System.Numerics.Vector<uint>.Count)
                    (new System.Numerics.Vector<uint>(from.Slice(i)) | alphas).CopyTo(to.Slice(i));
            }

            for (; i < from.Length; i++) to[i] = from[i] | alpha;
        }

        private static byte[] Blank(int height)
        {
            var frame = new byte[ScreenWidthPixels * height * 4];
            for (int alpha = 3; alpha < frame.Length; alpha += 4) frame[alpha] = 0xFF;
            return frame;
        }

        private void RequireRom(string member)
        {
            if (Bus is null) throw new InvalidOperationException($"{member}() called before LoadRom().");
        }

    }
}
