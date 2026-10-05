using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Native
{
    // The one engine class over any core ABI v1 library: every optional interface of the engine SPI, answered neutrally where the core lacks the capability; a port's shim subclasses it - see EmuSen_CoreAPI.md §13.2, §19, §26.
    public class CoreEngine : ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IFrameProfiler, IEngineFeatures, ISnapshotCore, IFrameSerial,
        IRepeatedRows, ICoreSettings, ICoprocessorHalt, IDisposable
    {
        private CoreMachine? _machine;
        private byte[] _frame = Array.Empty<byte>();
        private CoreInterface.FrameInfo _shape;
        private readonly FrameBufferLending _lending = new();
        private readonly Dictionary<string, string> _values = new(StringComparer.Ordinal);
        private readonly List<(uint Which, BatterySave Save)> _battery = new();
        private readonly List<string> _log = new();
        private uint[] _buttons = new uint[2];
        private CheatRegistry _cheats = new();
        private CheatRegistry _hostCheats = new();
        private int _cheatVersion = -1;
        private int _audioLimitSent = -1;
        private int _audioRate;
        private bool _skipRendering;
        private double _lastFrameMs;
        private byte[] _stateBuffer = Array.Empty<byte>();
        private IReadOnlyDictionary<string, string> _notes = new Dictionary<string, string>();

        public CoreLibrary Library { get; }
        public CoreInfo Info => Library.Info;
        public CoreMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");
        public string RomPath { get; private set; } = "";
        public string Console { get; private set; } = "";

        public virtual WatchRegistry Watches { get; } = new();
        public virtual FrameLogRegistry FrameLog { get; } = new();
        public virtual BreakpointRegistry Breakpoints { get; } = new();

        // The debugger's tables and logs over DEBUG, every processor's registries in it.
        public CoreDebugBridge Debug { get; }

        // The settings a frontend stored for this engine, by key; a key the schema lacks is dropped, as §6.13 asks of a host.
        public CoreEngine(CoreLibrary library, IReadOnlyDictionary<string, string>? settings = null)
        {
            if (!library.Available) throw new InvalidOperationException($"{System.IO.Path.GetFileName(library.Path)} is not in use: {library.Report}");
            Library = library;
            Debug = new CoreDebugBridge(this);
            foreach (var s in library.Settings) _values[s.Key] = s.Default;
            if (settings is not null)
                foreach (var (key, value) in settings)
                    if (library.Settings.FirstOrDefault(s => s.Key == key) is { } known && Accepts(known, value)) _values[key] = value;
        }

        public virtual string CoreName => Info.Name;

        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;
        public virtual double FrameRateHz => _machine?.Info.FrameRateHz ?? 60.0;
        public int AudioSampleRate => _machine is null ? 44100 : _audioRate;

        // The picture's size as the last frame reported it, its rows repeated unless the frontend repeats them.
        public virtual int ScreenWidth => _shape.Width;
        public virtual int ScreenHeight => _shape.Height * (RepeatRows ? Math.Max(1, _shape.RowRepeat) : 1);

        public EngineFeatures Features => EngineFeatures.All;

        // What the last load's console logged, newest last, kept to the last few hundred lines.
        public IReadOnlyList<string> RecentLog => _log;

        // The controller a port holds by machine info, else the system's first.
        private CoreController? ControllerFor(int port)
        {
            if (_machine is null) return SystemControllers().FirstOrDefault();
            string? id = _machine.Info.Ports.FirstOrDefault(p => p.Port == port)?.Controller;
            return SystemControllers().FirstOrDefault(c => c.Id == id) ?? (_machine.Info.Ports.Count == 0 ? SystemControllers().FirstOrDefault() : null);
        }

        private IEnumerable<CoreController> SystemControllers() =>
            (Info.Systems.FirstOrDefault(s => s.Id == _machine?.Info.System) ?? Info.Systems.FirstOrDefault())?.Controllers ?? Enumerable.Empty<CoreController>();

        public IReadOnlyList<PadButton> SupportedButtons =>
            ControllerFor(0)?.Buttons.Where(b => b.Control.HasValue).Select(b => b.Control!.Value).Distinct().ToArray() ?? Array.Empty<PadButton>();

        public IReadOnlyList<PadAxis> SupportedAxes =>
            Library.Has(CoreInterface.CapAxes) ? ControllerFor(0)?.Axes.Where(a => a.Control.HasValue).Select(a => a.Control!.Value).Distinct().ToArray() ?? Array.Empty<PadAxis>() : Array.Empty<PadAxis>();

        public virtual void SetButton(int port, PadButton button, bool pressed)
        {
            if (port < 0 || port >= _buttons.Length || ControllerFor(port) is not { } pad) return;
            foreach (var b in pad.Buttons)
            {
                if (b.Control != button || b.Bit > 31) continue;
                uint bit = 1u << (int)b.Bit;
                _buttons[port] = pressed ? _buttons[port] | bit : _buttons[port] & ~bit;
                _machine?.SetButtons(port, _buttons[port], bit);
            }
        }

        public void SetAxis(int port, PadAxis axis, double value)
        {
            if (_machine is null || ControllerFor(port) is not { } pad) return;
            foreach (var a in pad.Axes)
                if (a.Control == axis) _machine.SetAxis(port, a.Axis, a.Trigger ? Math.Clamp(value, 0, 1) : Math.Clamp(value, -1, 1));
        }

        public bool SkipRendering
        {
            get => _skipRendering;
            set
            {
                _skipRendering = value;
                _machine?.SetOptions(value);
            }
        }

        // Halted in front of a breakpoint on processor HaltedProcessor, the frame left open for the next RunFrame to resume.
        public virtual bool IsHaltedAtBreakpoint { get; protected set; }
        public virtual int HaltedAddress { get; protected set; }
        public uint HaltedProcessor { get; private set; }
        public bool IsHaltedOnCoprocessor => IsHaltedAtBreakpoint && HaltedProcessor != 0;
        public string HaltedProcessorName => _machine?.Info.Processors.FirstOrDefault(p => p.Id == HaltedProcessor)?.Name ?? "CPU";

        // What the image needs, answered from its bytes without a machine; Purpose says whether the game runs without it.
        public IReadOnlyList<FirmwareRequest> GetFirmwareRequirements(string romPath) =>
            Library.FirmwareFor(File.ReadAllBytes(romPath)).Select(Request).ToArray();

        private FirmwareRequest Request(CoreFirmware f) => Request(Info.Name, f);

        // A v1 core's firmware entry as the library's request; static so the firmware page asks from a sidecar's info with no library loaded.
        public static FirmwareRequest Request(string core, CoreFirmware f) =>
            new(core, f.Label, f.Name, (int)f.Size, f.Required ? "required" : "optional")
            {
                Parts = f.Parts, Required = f.Required, ReplacementEffect = f.Replacement?.Effect, ReplacementCost = f.Replacement?.Cost,
            };

        // One line for the status bar only when a file is absent with no replacement; a running replacement is the normal path and says nothing - see VenusRT_Native.md §66.
        public string? FirmwareNotice { get; private set; }

        private string? NoticeFor(CoreMachine machine, IReadOnlyList<CoreFirmware> wanted)
        {
            foreach (CoreFirmwareSource used in machine.Info.Firmware.Reverse())
            {
                if (wanted.FirstOrDefault(f => f.Which == used.Which) is not { } f || f.Replacement is not { } r || r.Effect == "exact") continue;
                if (used.Source == "absent") return $"{r.Cost} {f.Name} would supply the chip.";
            }
            return null;
        }

        // The image, its firmware from the library and its battery files by the runtime's path rule; a second create when machine info names battery files the first could not know.
        public virtual void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            string extension = System.IO.Path.GetExtension(path).ToLowerInvariant();
            // The battery folder is the system's, as the library files it: a .gbc game's saves are under GBC, not the shelf's GB.
            string console = Info.SystemFor(extension)?.Id.ToUpperInvariant() ?? CoreCatalog.ConsoleForRom(path) ?? Info.Id;
            var firmware = new List<(uint, byte[])>();
            // Each file whole by its name and size, else its first split form found, part n as file Which + n - see EmuSen_CoreAPI.md §6.2.
            foreach (var f in Library.FirmwareFor(image))
            {
                if (FirmwareLibrary.TryLoad(Request(f)) is { } data) firmware.Add((f.Which, data));
                else if (FirmwareLibrary.TryLoadParts(Request(f)) is { } parts) firmware.AddRange(parts.Select((p, n) => (f.Which + (uint)n, p)));
            }

            var saves = new List<(uint Which, BatterySave Save)> { (0, BatteryFile(path, console, 0, EmuSen.Galaxia.Library.SaveLibrary.SramExtension)) };
            CoreMachine machine = Create(image, firmware, saves);
            var wanted = machine.Info.Battery.Select(b => (b.Which, Save: BatteryFile(path, console, b.Which, b.Suffix))).ToList();
            bool differs = wanted.Count != saves.Count || wanted.Zip(saves).Any(p => p.First.Which != p.Second.Which || p.First.Save.Path != p.Second.Save.Path);
            if (differs && wanted.Any(w => w.Save.Read() is not null))
            {
                machine.Dispose();
                machine = Create(image, firmware, wanted);
            }

            _machine?.Dispose();
            _machine = machine;
            FirmwareNotice = NoticeFor(machine, Library.FirmwareFor(image));
            IsHaltedAtBreakpoint = false;
            AttachDebugger(machine);
            _battery.Clear();
            _battery.AddRange(wanted);
            RomPath = path;
            Console = console;
            _buttons = new uint[Math.Max(2, machine.Info.Ports.Count)];
            _cheatVersion = -1;
            _audioLimitSent = -1;
            _log.Clear();
            machine.SetOptions(_skipRendering);
            _audioRate = machine.AudioSampleRate;
            string run = SettingsText(createScope: false);
            if (run.Length > 0 && Library.Has(CoreInterface.CapSettings)) machine.SetSettings(run);
            Drain();
            _shape = machine.FrameInfo;
            _frame = new byte[Math.Max(0, (int)_shape.Bytes)];
        }

        // The adapter's own debugger over a new machine; a port's shim keeps its mirror's instead.
        protected virtual void AttachDebugger(CoreMachine machine) => Debug.Attach(machine);

        // Battery file <which> of a game, by the runtime's path rule; a port's shim opens it as its oracle does.
        protected virtual BatterySave BatteryFile(string path, string console, uint which, string suffix) => BatterySave.Open(path, console, extension: suffix);

        private CoreMachine Create(byte[] image, List<(uint, byte[])> firmware, List<(uint Which, BatterySave Save)> saves)
        {
            var files = new List<(uint, byte[])>(firmware);
            foreach (var (which, save) in saves)
                if (save.Read() is { } saved) files.Add((which, saved));
            return new CoreMachine(Library, image, SettingsText(createScope: true), files);
        }

        // Every value as key=value lines: create-time and run-time keys at create, run-time keys only after it.
        private string SettingsText(bool createScope) =>
            string.Join("\n", Library.Settings.Where(s => createScope || !s.CreateScope).Select(s => $"{s.Key}={_values[s.Key]}"));

        // Advance, the frame-end work, present, then the events - the runtime's order of EmuSen_CoreAPI.md §8.2.
        public void RunFrame()
        {
            CoreMachine m = _machine ?? throw new InvalidOperationException("RunFrame() called before LoadRom().");
            RefreshCheats(m);
            SyncAudioLimit(m);
            long start = Stopwatch.GetTimestamp();
            if (!AdvanceFrame(m))
            {
                _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
                Drain();
                return;
            }
            EndFrame(m);
            if (!_skipRendering && Library.Has(CoreInterface.CapPresent)) m.Present();
            _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
            Drain();
            if (TotalFrames % 60 == 0) KeepLog(m.DrainLog());
        }

        // The machine to the frame's end; false is a halt with the frame left open, which skips the frame-end work.
        protected virtual bool AdvanceFrame(CoreMachine m)
        {
            bool resuming = IsHaltedAtBreakpoint;
            IsHaltedAtBreakpoint = false;
            if (!resuming && !Debug.Armed)
            {
                m.Advance();
                return true;
            }
            if (Debug.RunFrame(resuming, out int haltedAt, out uint processor)) return true;
            (IsHaltedAtBreakpoint, HaltedAddress, HaltedProcessor) = (true, haltedAt, processor);
            return false;
        }

        private void EndFrame(CoreMachine m)
        {
            FrameLog.RecordFrame(TotalFrames, ReadForFrameLog);
            if (_machine is not null) _hostCheats.ApplyAll(ReadSpace, WriteSpace);
            Breakpoints.NoteFrame(TotalFrames);
            if (BatterySave.IsFlushFrame(TotalFrames)) SaveSram();
        }

        // Events after a call that can raise them: a new size, a new rate, a log to drain, a battery changed, or everything read again.
        private void Drain()
        {
            if (_machine is not { } m) return;
            foreach (var e in m.DrainEvents())
            {
                switch (e.Kind)
                {
                    case CoreInterface.EventGeometry: _shape = m.FrameInfo; break;
                    case CoreInterface.EventAudioRate: _audioRate = (int)e.A; break;
                    case CoreInterface.EventLog: KeepLog(m.DrainLog()); break;
                    case CoreInterface.EventBattery: if (Library.Has(CoreInterface.CapBatteryDirty)) SaveSram(); break;
                    case CoreInterface.EventMachineInfo: m.ReadInfo(); _shape = m.FrameInfo; break;
                }
            }
            if (_frame.Length < _shape.Bytes) _frame = new byte[_shape.Bytes];
        }

        private void KeepLog(IReadOnlyList<string> lines)
        {
            _log.AddRange(lines);
            if (_log.Count > 512) _log.RemoveRange(0, _log.Count - 512);
        }

        // RGBA8888, the only format offered at create; rows repeated here unless the frontend repeats them.
        public byte[] GetFrameBufferRgba()
        {
            if (_machine is not { } m) return Array.Empty<byte>();
            _shape = m.FrameInfo;
            if (_frame.Length < _shape.Bytes) _frame = new byte[_shape.Bytes];
            m.CopyFrame(_frame);
            int width = _shape.Width, height = _shape.Height, stride = _shape.Stride == 0 ? width * 4 : _shape.Stride;
            int repeat = RepeatRows ? Math.Max(1, _shape.RowRepeat) : 1;
            byte[] buffer = _lending.Lend(width * 4 * height * repeat);
            for (int y = 0; y < height; y++)
                for (int r = 0; r < repeat; r++)
                    _frame.AsSpan(y * stride, width * 4).CopyTo(buffer.AsSpan((y * repeat + r) * width * 4));
            return buffer;
        }

        public void ReturnFrameBuffer(byte[] buffer) => _lending.Return(buffer);

        public FrameBufferLending FrameBuffers => _lending;

        public long FrameSerial => _machine?.FrameInfo.Serial ?? 0;

        public bool RepeatRows { get; set; } = true;
        public int RowRepeat => RepeatRows ? 1 : Math.Max(1, _shape.RowRepeat);

        public short[] DequeueAudioSamples(int maxFrames)
        {
            if (_machine is null) return Array.Empty<short>();
            var (samples, rate) = _machine.DrainAudio(maxFrames);
            _audioRate = rate;
            return samples;
        }

        private void SyncAudioLimit(CoreMachine m)
        {
            int limit = EmuSen.Audio.AudioSettings.AudioBufferMaxSamples;
            if (_audioLimitSent == limit) return;
            m.SetAudioLimit(limit);
            _audioLimitSent = limit;
        }

        public IReadOnlyList<(string Name, double Milliseconds)> LastFramePhases
        {
            get
            {
                long[] ns = _machine?.Phases() ?? Array.Empty<long>();
                if (ns.Length == 0) return new[] { ("frame", _lastFrameMs) };
                var names = _machine!.Info.Phases;
                return ns.Select((v, i) => (i < names.Count ? names[i] : $"phase {i}", v / 1e6)).ToArray();
            }
        }

        public virtual int StateVersion => (int)(_machine?.Info.StateVersion ?? 0);

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

        public void SaveState(Stream stream) => Write(stream, 0);

        // Kind 1 where the core claims SNAPSHOT, the whole state otherwise; LoadState reads either.
        public void SaveSnapshot(Stream stream) => Write(stream, Library.Has(CoreInterface.CapSnapshot) ? 1u : 0u);

        private void Write(Stream stream, uint kind)
        {
            CoreMachine m = _machine ?? throw new InvalidOperationException("SaveState() called before LoadRom().");
            int size = m.StateSize(kind);
            if (_stateBuffer.Length != size) _stateBuffer = new byte[size];
            m.Save(_stateBuffer, kind);
            stream.Write(_stateBuffer);
        }

        public virtual void LoadState(Stream stream)
        {
            CoreMachine m = _machine ?? throw new InvalidOperationException("LoadState() called before LoadRom().");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            m.Load(copy.ToArray());
            _cheatVersion = -1;
            IsHaltedAtBreakpoint = false;
            Drain();
        }

        // Each battery file the machine keeps: written when it changed where the core tracks changes, else whenever asked.
        public void SaveSram()
        {
            if (_machine is not { } m) return;
            foreach (var (which, save) in _battery)
            {
                if (save.Path is null) continue;
                var (data, flags) = m.Battery(which);
                if (data.Length == 0) continue;
                if (Library.Has(CoreInterface.CapBatteryDirty) && (flags & 2) != 0 && (flags & 1) == 0) continue;
                if (save.Write(data)) m.BatterySaved(which);
            }
        }

        // A registry handed over applies at once, as the next frame's refresh would apply it.
        public virtual CheatRegistry Cheats
        {
            get => _cheats;
            set
            {
                _cheats = value;
                _cheatVersion = -1;
                if (!Library.Has(CoreInterface.CapCheatPokes)) _hostCheats = value;
            }
        }

        // ROM patches within machine info's range; pokes the core gates itself as quads, the rest applied here through space_write - see EmuSen_CoreAPI.md §6.12.
        protected void RefreshCheats(CoreMachine m)
        {
            int version = _cheats.Version;
            if (version == _cheatVersion) return;
            _cheatVersion = version;
            if (m.Info.PatchLow is long low && m.Info.PatchHigh is long high) m.SetRomPatches(NativeRtCore<NativeMachine>.RomPatchTriples(_cheats, low, high));
            if (!Library.Has(CoreInterface.CapCheatPokes))
            {
                _hostCheats = _cheats;
                return;
            }
            var quads = new List<uint>();
            var host = new CheatRegistry { MasterEnabled = _cheats.MasterEnabled };
            foreach (var c in _cheats.GetCheats())
            {
                if (c.Kind != CheatKind.RamPoke || !c.Enabled) continue;
                var simple = c.Writes.Select(w => (w, SpaceOf(w.Space))).ToList();
                if (_cheats.MasterEnabled && simple.All(p => p.Item2 is not null && p.w.Type == CheatWriteType.Set && p.w.EffectiveWidth == 1 && p.w.BitPosition is null && p.w.RepeatCount <= 1))
                    foreach (var (w, space) in simple) quads.AddRange(new[] { space!.Value, (uint)w.Address, w.Value & 0xFF, c.Compare is byte b ? b : uint.MaxValue });
                else host.AddCheat(c.Kind, c.Writes, c.Compare, c.Description);
            }
            m.SetCheatPokes(quads.ToArray());
            _hostCheats = host;
        }

        // Apply now, between frames: every enabled poke, the ones the core gates at its frame's end too - see VenusRT_Native.md §66.3.
        public void ApplyCheats()
        {
            if (_machine is not { } m) return;
            RefreshCheats(m);
            _cheats.ApplyAll(ReadSpace, WriteSpace);
        }

        private uint? SpaceOf(string name) => _machine?.Info.Spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase))?.Id;

        public virtual byte ReadSpace(string spaceName, int address)
        {
            if (_machine is null || SpaceOf(spaceName) is not uint space) return 0;
            Span<byte> one = stackalloc byte[1];
            _machine.ReadSpace(space, address, one);
            return one[0];
        }

        public virtual void WriteSpace(string spaceName, int address, byte value)
        {
            if (_machine is null || SpaceOf(spaceName) is not uint space) return;
            _machine.WriteSpace(space, address, stackalloc byte[] { value });
        }

        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            long value = 0;
            for (int i = 0; i < width; i++) value |= (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        // The schema's switch, count and choice settings that are not hidden, with the core's note for the value in use.
        public virtual IReadOnlyList<CoreSetting> Settings =>
            Library.Settings.Where(s => !s.Hidden).Select(s => s.AsCoreSetting() is { } row ? row with { Note = _ => _notes.GetValueOrDefault(s.Key) } : null).OfType<CoreSetting>().ToArray();

        public IReadOnlyList<CoreSettingDescriptor> SettingDescriptors => Library.Settings;

        public virtual string Get(string key) => _values.TryGetValue(key, out var v) ? v : "";

        // A run-time key reaches the machine between frames; a create-time one is kept for the next load. A value outside its domain is refused.
        public virtual void Set(string key, string value)
        {
            var s = Library.Settings.FirstOrDefault(x => x.Key == key) ?? throw new ArgumentException($"{Info.Name} has no setting {key}.");
            if (!Accepts(s, value)) throw new ArgumentException($"{key}={value} is outside what {Info.Name} takes.");
            if (!s.CreateScope && _machine is { } m && Library.Has(CoreInterface.CapSettings))
            {
                m.SetSettings($"{key}={value}");
                _notes = m.SettingNotes();
                Drain();
            }
            _values[key] = value;
        }

        private static bool Accepts(CoreSettingDescriptor s, string value) => s.Kind switch
        {
            "switch" => value is "true" or "false",
            "count" => int.TryParse(value, out int n) && n >= s.Min && n <= s.Max,
            "choice" => s.Choices.Contains(value),
            _ => !value.Contains('\n') && !value.Contains('\r'),
        };

        public CoreDebugTarget CreateDebugTarget() => new(this);

        public void Dispose()
        {
            _machine?.Dispose();
            _machine = null;
            _lending.Close();
        }
    }
}
