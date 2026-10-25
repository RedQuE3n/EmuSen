using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using EmuSen.Common.Firmware;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Galaxia.Input;
using EmuSen.Galaxia.Media;

namespace EmuSen.Cores.Native
{
    // The one engine class over any core ABI v1 library: every optional interface of the engine SPI, answered neutrally where the core lacks the capability; a port's shim subclasses it - see EmuSen_CoreAPI.md §13.2, §19, §26.
    public class CoreEngine : ICore, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IFrameProfiler, IEngineFeatures, ISnapshotCore, IFrameSerial,
        IRepeatedRows, ICoreSettings, ICoprocessorHalt, IDevicePicture, IDisposable
    {
        private CoreMachine? _machine;
        private byte[] _frame = Array.Empty<byte>();
        private CoreInterface.FrameInfo _shape;
        private readonly FrameBufferLending _lending = new();
        private readonly FrameBufferLending _rawLending = new();
        private string? _rawWanted, _rawAsked;
        private bool _rawCurrent;
        private int _rawIndex = -1;
        private readonly Dictionary<string, string> _values = new(StringComparer.Ordinal);
        private readonly List<(uint Which, BatterySave Save)> _battery = new();
        private readonly List<string> _log = new();
        private readonly InputLatch _input = new();
        private CheatRegistry _cheats = new();
        private CheatRegistry _hostCheats = new();
        private int _cheatVersion = -1;
        private int _audioLimitSent = -1;
        private int _audioRate;
        private bool _skipRendering;
        private double _lastFrameMs;
        private byte[] _stateBuffer = Array.Empty<byte>(), _loadBuffer = Array.Empty<byte>();
        private IReadOnlyDictionary<string, string> _notes = new Dictionary<string, string>();
        private DiscSet? _set;
        private List<string> _discs = new();
        private long _closeLidAt = -1;

        // Whether the machine is as its create made it: no frame run and no state loaded, so a create-time setting may make it again.
        private bool _fresh;

        // Each save medium's mode the frontend stored for the console, by medium id, read at each load; Per game where absent - see EmuSen_SaveMedia.md §3.3.
        public IReadOnlyDictionary<string, CardChoice> Media { get; set; } = new Dictionary<string, CardChoice>();

        // Set before LoadRom or StartConsole: what the frontend made this core with - see EmuSen_Multicore.md §6.
        public CoreOptions Options { get; set; } = CoreOptions.Default;

        // Why a slot runs on the game's own file this load, or null: its shared file is in another slot or another program - see EmuSen_SaveMedia.md §3.5.
        public string? MediaNotice { get; private set; }

        private MediaPlacement? _placement;
        private bool _bigEndian;

        public CoreLibrary Library { get; }
        public CoreInfo Info => Library.Info;
        public CoreMachine Machine => _machine ?? throw new InvalidOperationException("No ROM is loaded.");
        public string RomPath { get; private set; } = "";
        public string Console { get; private set; } = "";

        public virtual WatchRegistry Watches { get; } = new();
        public virtual FrameLogRegistry FrameLog { get; } = new();
        public virtual BreakpointRegistry Breakpoints { get; } = new();

        // The debugger's labels, the engine's so a label outlives the target it was made through - see EmuSen_Backend.md §22.
        public LabelRegistry Labels { get; } = new();

        // Pinned bytes, written back at each frame's end, since a v1 core reports stores only after the fact - see `man freeze`.
        public FreezeRegistry Freezes { get; } = new();

        // The debugger's tables and logs over DEBUG, every processor's registries in it.
        public CoreDebugBridge Debug { get; }

        // The frame-end work's readers and writers, made once so a frame allocates none - see EmuSen_Backend.md §14.
        private readonly Func<string, int, int, long> _readForFrameLog;
        private readonly Func<string, int, byte> _read;
        private readonly Action<string, int, byte> _write;

        // The settings a frontend stored for this engine, by key; a key the schema lacks is dropped, as §6.13 asks of a host.
        public CoreEngine(CoreLibrary library, IReadOnlyDictionary<string, string>? settings = null)
        {
            _readForFrameLog = ReadForFrameLog;
            _read = ReadSpace;
            _write = WriteSpace;
            if (!library.Available) throw new InvalidOperationException($"{System.IO.Path.GetFileName(library.Path)} is not in use: {library.Report}");
            Library = library;
            Debug = new CoreDebugBridge(this);
            Watches.Say = Say;
            foreach (var s in library.Settings) _values[s.Key] = s.Default;
            if (settings is not null)
                foreach (var (key, value) in settings)
                    if (library.Settings.FirstOrDefault(s => s.Key == key) is { } known && Accepts(known, value)) _values[key] = value;
        }

        public virtual string CoreName => Info.Name;

        public bool IsRomLoaded => _machine != null;
        public long TotalFrames => _machine?.TotalFrames ?? 0;
        public virtual double FrameRateHz => _machine?.Info.FrameRateHz ?? 60.0;
        public string? TelevisionStandard => _machine?.Info.Region;
        public int AudioSampleRate => _machine is null ? 44100 : _audioRate;

        // The picture's size as the last frame reported it, its rows repeated unless the frontend repeats them.
        public virtual int ScreenWidth => _shape.Width;
        public virtual int ScreenHeight => _shape.Height * (RepeatRows ? Math.Max(1, _shape.RowRepeat) : 1);

        // The frame's own shape when the core gives one, else its machine's; 0 when it gives neither - see EmuSen_Serenity.md §2.9.
        // The console's lines in a picture drawn at a whole multiple of them, counted in the rows ScreenHeight holds - see EmuSen_CoreAPI.md §6.35.
        public int DisplayLines => _shape.Lines * (RepeatRows ? Math.Max(1, _shape.RowRepeat) : 1);

        public virtual double DisplayAspect =>
            _shape.AspectNum > 0 && _shape.AspectDen > 0 ? _shape.AspectNum / (double)_shape.AspectDen
            : _machine?.Info is { AspectNum: > 0, AspectDen: > 0 } info ? info.AspectNum / (double)info.AspectDen : 0;

        public EngineFeatures Features => EngineFeatures.All;

        // What the last load's console logged, newest last, kept to the last few hundred lines.
        public IReadOnlyList<string> RecentLog => _log;

        // Every line kept since the engine was made, so a reader can tell which of RecentLog are new.
        public long LoggedLines { get; private set; }

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

        // Held here from any thread and sent to the machine on its own as the next frame starts - see Diamond_Players.md §12.7.
        public virtual void SetButton(int port, PadButton button, bool pressed)
        {
            if (port < 0 || port >= _input.Ports || ControllerFor(port) is not { } pad) return;
            foreach (var b in pad.Buttons)
            {
                if (b.Control != button || b.Bit > 31) continue;
                uint bit = 1u << (int)b.Bit;
                _input.Hold(port, pressed ? bit : 0, bit);
            }
        }

        // A controller plugged in or pulled out, sent to a core claiming PRESENCE as the next frame starts - see EmuSen_CoreAPI.md §6.36.
        public void SetControllerConnected(int port, bool connected) => _input.Connect(port, connected);

        public void SetAxis(int port, PadAxis axis, double value)
        {
            if (_machine is null || ControllerFor(port) is not { } pad) return;
            foreach (var a in pad.Axes)
                if (a.Control == axis) _input.Move(port, a.Axis, a.Trigger ? Math.Clamp(value, 0, 1) : Math.Clamp(value, -1, 1));
        }

        // A shim's own pad routing into the same latch: the bits of changed take mask's values at the next frame.
        protected void HoldButtons(int port, uint mask, uint changed) => _input.Hold(port, mask, changed);

        // The latch sent now rather than as the next frame starts, for a shim that runs the machine by instructions between frames.
        protected void SendInput(CoreMachine m) => _input.Send(m);

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

        // What the image play opens needs, answered from its mapped bytes without a machine; Purpose says whether the game runs without it.
        public IReadOnlyList<FirmwareRequest> GetFirmwareRequirements(string romPath)
        {
            using DiscFiles disc = DiscSet.Of(romPath, CoreCatalog.IsImageExtension).Open(0);
            return Library.FirmwareFor(Held(disc.Image)).Select(Request).ToArray();
        }

        // The required entries the image asks for, answered without reading it when the core's info leaves none possible - see Diamond_Players.md §12.8.2.
        public IReadOnlyList<FirmwareRequest> RequiredFirmwareFor(string romPath) =>
            CanRequireFirmware(Info) ? GetFirmwareRequirements(romPath).Where(r => r.Required).ToArray() : Array.Empty<FirmwareRequest>();

        // False only when every system lists its firmware and none of it is required, the lists naming every file firmware_for can - see EmuSen_CoreAPI.md §6.3.
        public static bool CanRequireFirmware(CoreInfo info) =>
            info.Systems.Count == 0 || info.Systems.Any(s => s.Firmware.Count == 0 || s.Firmware.Any(f => f.Required));

        private FirmwareRequest Request(CoreFirmware f) => Request(Info.Name, f);

        // A v1 core's firmware entry as the library's request; static so the firmware page asks from a sidecar's info with no library loaded.
        public static FirmwareRequest Request(string core, CoreFirmware f) =>
            new(core, f.Label, f.Name, (int)f.Size, f.Required ? "required" : "optional")
            {
                Parts = f.Parts, Required = f.Required, ReplacementEffect = f.Replacement?.Effect, ReplacementCost = f.Replacement?.Cost, Md5 = f.Md5,
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

        // Where each entry passed to the last machine was found, by its Which, for the session's log.
        private readonly Dictionary<uint, FirmwareLocated> _firmwareFolders = new();

        // Each entry whole by its name and size, else its first split form, from the first folder holding it; part n as file Which + n - see EmuSen_CoreAPI.md §6.2 and EmuSen_Firmware.md §2.3.
        private List<(uint, byte[])> FirmwareFiles(IEnumerable<CoreFirmware> entries)
        {
            IReadOnlyList<string> folders = FirmwareLibrary.Folders;
            var firmware = new List<(uint, byte[])>();
            _firmwareFolders.Clear();
            foreach (var f in entries)
            {
                if (FirmwareLibrary.Find(Request(f), folders) is not { } found) continue;
                firmware.AddRange(found.Parts.Select((p, n) => (f.Which + (uint)n, p)));
                _firmwareFolders[f.Which] = new FirmwareLocated(found.Folder, found.Names, found.Whole, found.ByContent);
            }
            return firmware;
        }

        // Which firmware the machine runs on, one line each for the session's log: the player's file and its folder, or the core's own - see EmuSen_Firmware.md §3.1.
        public static IEnumerable<string> FirmwareLines(string core, IReadOnlyList<CoreFirmwareSource> used, IReadOnlyList<CoreFirmware> wanted, IReadOnlyDictionary<uint, FirmwareLocated>? folders = null) =>
            used.Select(u => (u.Source, u.Which, F: wanted.FirstOrDefault(f => f.Which == u.Which)))
                .Where(u => u.F is not null)
                .Select(u => $"[Firmware] {u.F!.Label}: " + u.Source switch
                {
                    "file" => folders?.GetValueOrDefault(u.Which) is { } at
                        ? $"the player's {(at.ByContent ? at.Names[0] + ", recognised by its content" : u.F.Name)}, in {at.Folder}"
                        : $"the player's {u.F.Name}",
                    "replacement" => $"{core}'s own replacement",
                    _ => "absent",
                });

        // The discs of the game: an .m3u's, else the one image; each unpacked where it is an archive - see EmuSen_Galaxia.md §3.8.
        public static IReadOnlyList<string> DiscsOf(string path) => DiscSet.Of(path, CoreCatalog.IsImageExtension).Images;

        private static unsafe CoreBytes Held(MappedFile file) => new(file.Pointer, file.Length);

        // A disc's companions as create and media_insert take them, numbered from EMUSEN_FILE_MEDIA in Galaxia's order.
        private static List<(uint, CoreBytes)> MediaOf(DiscFiles disc) => disc.Companions.Select((c, n) => (CoreInterface.FileMedia + (uint)n, Held(c))).ToList();

        // Whether the core reads a disc where Galaxia has it mapped, which the host then keeps so until the loan ends - see EmuSen_CoreAPI.md §6.24.
        private bool Lends => Library.Has(CoreInterface.CapBorrow);

        // The disc's files from Galaxia, its firmware from the library and its battery files by the runtime's path rule: those core info lists for the system, else file 0 and a second create when machine info names others that exist.
        public virtual void LoadRom(string path)
        {
            DiscSet set = DiscSet.Of(path, CoreCatalog.IsImageExtension);
            // The battery folder is the system's, as the library files it: a .gbc game's saves are under GBC, not the shelf's GB; a system only discovery runs, its pack's console's.
            CoreSystem? system = CoreCatalog.SystemOfFile(path, Info);
            string console = CoreCatalog.DiscoveredConsoleForSystem(system?.Id) ?? system?.Id?.ToUpperInvariant() ?? CoreCatalog.ConsoleForRom(path) ?? Info.Id;
            List<(uint, byte[])> firmware;
            IReadOnlyList<CoreFirmware> asked;
            CoreMachine machine;
            List<(uint Which, BatterySave Save)> wanted;
            MediaPlacement? placement = null;
            DiscFiles disc = set.Open(0);
            // Whether a machine has taken the keeping of the disc's files, as one that borrows does.
            bool kept = false;
            try
            {
                asked = Library.FirmwareFor(Held(disc.Image));
                firmware = FirmwareFiles(asked);
                ReleaseMedia();
                placement = MemoryCards.Place(console, system, Media, System.IO.Path.GetFileNameWithoutExtension(path), b => BatteryFile(path, console, b.Which, b.Suffix), Options.BatteryRamDisabled);
                BatterySave SaveFor(CoreBatteryFile b) => placement!.Files.TryGetValue(b.Which, out BatterySave? placed) ? placed : BatteryFile(path, console, b.Which, b.Suffix);
                var saves = system?.Battery is { } listed
                    ? listed.Select(b => (b.Which, Save: SaveFor(b))).ToList()
                    : new List<(uint Which, BatterySave Save)> { (0, BatteryFile(path, console, 0, EmuSen.Galaxia.Library.SaveLibrary.SramExtension)) };
                machine = Create(disc, firmware, saves, placement, system?.Id);
                kept = Lends;
                wanted = machine.Info.Battery.Select(b => (b.Which, Save: SaveFor(b))).ToList();
                // Machine info is the authority: a file it names that exists and was not passed means the machine is made again with it.
                if (wanted.Any(w => w.Save.Read() is not null && !saves.Any(s => s.Which == w.Which && s.Save.Path == w.Save.Path)))
                {
                    machine.Dispose();
                    kept = false;
                    if (Lends) disc = set.Open(0);
                    machine = Create(disc, firmware, wanted, placement, system?.Id);
                    kept = Lends;
                }
            }
            catch
            {
                placement?.Dispose();
                throw;
            }
            finally
            {
                // A core that copies has its copy by now; one that borrows holds the files through its machine.
                if (!kept) disc.Dispose();
            }
            Adopt(machine, placement!, asked, wanted, set, path, console, null);
        }

        // The console of systemId with nothing in it, its firmware and save media placed as Diamond_Shell.md §11.2 gives.
        public virtual void StartConsole(string systemId)
        {
            CoreSystem system = Info.Systems.FirstOrDefault(s => s.Id == systemId && s.StartsEmpty)
                ?? throw new NotSupportedException($"{Info.DisplayName} does not start {systemId} with nothing in it.");
            string console = CoreCatalog.DiscoveredConsoleForSystem(systemId) ?? CoreCatalog.ConsoleForSystem(systemId) ?? systemId.ToUpperInvariant();
            List<(uint, byte[])> firmware = FirmwareFiles(system.Firmware);
            ReleaseMedia();
            var choices = MemoryCards.MediaOf(console, system).ToDictionary(m => m.Id, m => Media.TryGetValue(m.Id, out CardChoice? c) && c.Apart ? c : new CardChoice(MemoryCards.Shared, c?.Names ?? Array.Empty<string>()));
            MediaPlacement placement = MemoryCards.Place(console, system, choices, ConsoleSession.NameOf(console), _ => BatterySave.None, Options.BatteryRamDisabled);
            List<(uint Which, BatterySave Save)> saves;
            CoreMachine machine;
            try
            {
                saves = (system.Battery ?? Array.Empty<CoreBatteryFile>()).Select(b => (b.Which, Save: placement.Files.TryGetValue(b.Which, out BatterySave? placed) ? placed : BatterySave.None)).ToList();
                machine = new CoreMachine(Library, default, Array.Empty<(uint, CoreBytes)>(), SettingsText(createScope: true), FilesFor(firmware, saves, placement), null, placement.RemovableApart, placement.ConsoleApart, Library.Has(CoreInterface.CapDevicePicture) ? DeviceLender?.Invoke() : null, system: systemId);
            }
            catch
            {
                placement.Dispose();
                throw;
            }
            Adopt(machine, placement, system.Firmware, saves, null, "", console, systemId);
        }

        // The system a console session was started as, or null for a game's - see Diamond_Shell.md §1.5.
        public string? StartedEmpty { get; private set; }

        // The new machine in place of the last, with its media, discs, firmware words, debugger, input and frame.
        private void Adopt(CoreMachine machine, MediaPlacement placement, IReadOnlyList<CoreFirmware> asked, List<(uint Which, BatterySave Save)> wanted, DiscSet? set, string path, string console, string? emptySystem)
        {
            _placement = placement;
            MediaNotice = placement.Notices.Count == 0 ? null : string.Join(" ", placement.Notices);
            _set = set;
            _discs = set?.Images.ToList() ?? new List<string>();
            DiscIndex = set is null ? -1 : 0;
            _closeLidAt = -1;
            StartedEmpty = emptySystem;

            _machine?.Dispose();
            _machine = machine;
            _bigEndian = EmuSen.Crystal.Systems.SystemPacks.BigEndian(machine.Info.System);
            FirmwareNotice = NoticeFor(machine, asked);
            foreach (string line in FirmwareLines(Info.DisplayName, machine.Info.Firmware, asked, _firmwareFolders)) Say(line);
            foreach (string notice in placement.Notices) Say($"[MemoryCards] {notice}");
            IsHaltedAtBreakpoint = false;
            AttachDebugger(machine);
            _battery.Clear();
            _battery.AddRange(wanted);
            foreach (var (_, save) in wanted) save.Settle(Say);
            RomPath = path;
            Console = console;
            _input.Reset(Math.Max(2, machine.Info.Ports.Count), AxesOf(machine));
            _fresh = true;
            _cheatVersion = -1;
            _audioLimitSent = -1;
            (_rawCurrent, _rawIndex) = (false, -1);
            _log.Clear();
            machine.SetOptions(_skipRendering);
            _audioRate = machine.AudioSampleRate;
            string run = SettingsText(createScope: false);
            if (run.Length > 0 && Library.Has(CoreInterface.CapSettings)) machine.SetSettings(run);
            // The notes for the values the machine was made with, so a row shows its note from the first frame - see EmuSen_Backend.md §18.10.
            _notes = machine.SettingNotes();
            Drain();
            _shape = machine.FrameInfo;
            _frame = new byte[Math.Max(0, (int)_shape.Bytes)];
        }

        // One past the highest axis any controller of the machine's system names.
        private int AxesOf(CoreMachine machine) =>
            (Info.Systems.FirstOrDefault(s => s.Id == machine.Info.System) ?? Info.Systems.FirstOrDefault())?.Controllers.SelectMany(c => c.Axes).Select(a => (int)a.Axis + 1).DefaultIfEmpty(0).Max() ?? 0;

        // The adapter's own debugger over a new machine; a port's shim keeps its mirror's instead.
        protected virtual void AttachDebugger(CoreMachine machine) => Debug.Attach(machine);

        // Battery file <which> of a game, by the runtime's path rule; a port's shim opens it as its oracle does.
        protected virtual BatterySave BatteryFile(string path, string console, uint which, string suffix) => BatterySave.Open(path, console, disabled: Options.BatteryRamDisabled, extension: suffix, packageRoot: Options.PackageRoot);

        // Lets go of the locks on the game's save media, which a frontend calls when it leaves the game; a load takes them again - see EmuSen_SaveMedia.md §3.5.
        public void ReleaseMedia()
        {
            _placement?.Dispose();
            _placement = null;
        }

        // The machine for a disc, as the system it was routed to: lent where the core borrows, else read by the core within the call - see EmuSen_CoreAPI.md §6.40.
        private CoreMachine Create(DiscFiles disc, List<(uint, byte[])> firmware, List<(uint Which, BatterySave Save)> saves, MediaPlacement placement, string? system)
        {
            var lend = Library.Has(CoreInterface.CapDevicePicture) ? DeviceLender?.Invoke() : null;
            return new CoreMachine(Library, Held(disc.Image), MediaOf(disc), SettingsText(createScope: true), FilesFor(firmware, saves, placement), Lends ? disc : null, sharedRemovable: placement.RemovableApart, sharedConsole: placement.ConsoleApart, device: lend, system: system);
        }

        // The firmware, then each battery file that holds bytes or a console medium's formatted blank.
        private static List<(uint, byte[])> FilesFor(List<(uint, byte[])> firmware, List<(uint Which, BatterySave Save)> saves, MediaPlacement placement)
        {
            var files = new List<(uint, byte[])>(firmware);
            foreach (var (which, save) in saves)
                if ((save.Read() ?? (placement.Fresh.TryGetValue(which, out ISaveMediumFormat? fresh) ? fresh.Format() : null)) is { } saved) files.Add((which, saved));
            return files;
        }

        // Every value as key=value lines: create-time and run-time keys at create, run-time keys only after it.
        private string SettingsText(bool createScope) =>
            string.Join("\n", Library.Settings.Where(s => createScope || !s.CreateScope).Select(s => $"{s.Key}={_values[s.Key]}"));

        // The discs of the game, an .m3u's in its order, and the one in the drive, by index - see EmuSen_CoreAPI.md §6.23.
        public IReadOnlyList<string> Discs => _discs;
        public int DiscIndex { get; private set; }

        // Whether the core changes discs: it claims MEDIA and names a drive.
        public bool CanChangeDisc => Library.Has(CoreInterface.CapMedia) && _machine?.Info.Drives.Count > 0;

        public bool LidOpen => _machine?.Info.Drives.FirstOrDefault()?.LidOpen ?? false;

        // How long Change disc holds the lid open, in emulated seconds: as long as ps1-tests' disc-swap leaves it - see Diamond_Players.md §3.
        public const double LidOpenSeconds = 1.0;

        // The player's lid, opened or closed at once.
        public void SetLid(bool open)
        {
            if (!CanChangeDisc) throw new InvalidOperationException($"{Info.DisplayName} cannot change discs.");
            Machine.MediaLid(Machine.Info.Drives[0].Drive, open);
            _closeLidAt = -1;
        }

        // Disc index of the session's discs into the drive, or -1 to empty it, its lid open, or for a state about to load with restore.
        public void InsertDisc(int index, bool restore = false)
        {
            if (!CanChangeDisc) throw new InvalidOperationException($"{Info.DisplayName} cannot change discs.");
            if (index < -1 || index >= _discs.Count) throw new ArgumentOutOfRangeException(nameof(index), $"the session has {_discs.Count} disc(s)");
            uint drive = Machine.Info.Drives[0].Drive;
            if (index < 0)
            {
                Machine.MediaInsert(drive, null, Array.Empty<(uint, byte[])>(), restore);
                DiscIndex = -1;
                return;
            }
            DiscFiles disc = _set is not null && index < _set.Images.Count ? _set.Open(index) : DiscSet.Of(_discs[index], CoreCatalog.IsImageExtension).Open(0);
            try { Machine.MediaInsert(drive, Held(disc.Image), MediaOf(disc), restore, Lends ? disc : null); }
            finally
            {
                if (!Lends) disc.Dispose();
            }
            DiscIndex = index;
        }

        // Any disc file put in as a player would, the lid opened and closed LidOpenSeconds later; the core judges it, and a refused disc leaves the lid open - see Diamond_Shell.md §1.5.
        public void InsertDisc(string path)
        {
            int index = _discs.FindIndex(d => string.Equals(d, path, StringComparison.Ordinal));
            if (index < 0)
            {
                _discs.Add(path);
                index = _discs.Count - 1;
            }
            try { ChangeDisc(index); }
            catch
            {
                if (index == _discs.Count - 1 && index != DiscIndex && (_set is null || index >= _set.Images.Count)) _discs.RemoveAt(index);
                throw;
            }
        }

        // The drive emptied as a player would: the lid opened, the disc taken out, the lid closed LidOpenSeconds later.
        public void EjectDisc() => ChangeDisc(-1);

        // Change disc as one action: the lid opened, disc index in, the lid closed after LidOpenSeconds of the game's own time.
        public void ChangeDisc(int index)
        {
            if (!LidOpen) SetLid(true);
            InsertDisc(index);
            _closeLidAt = TotalFrames + Math.Max(1, (long)Math.Round(LidOpenSeconds * FrameRateHz));
        }

        // Advance, the frame-end work, present, then the events - the runtime's order of EmuSen_CoreAPI.md §8.2.
        public void RunFrame()
        {
            CoreMachine m = _machine ?? throw new InvalidOperationException("RunFrame() called before LoadRom().");
            if (_closeLidAt >= 0 && TotalFrames >= _closeLidAt) SetLid(false);
            _fresh = false;
            RefreshCheats(m);
            SyncAudioLimit(m);
            _input.Send(m);
            SyncRaw(m);
            long start = Stopwatch.GetTimestamp();
            if (!AdvanceFrame(m))
            {
                _lastFrameMs = Stopwatch.GetElapsedTime(start).TotalMilliseconds;
                Drain();
                return;
            }
            EndFrame(m);
            if (!_skipRendering && Library.Has(CoreInterface.CapPresent)) m.Present();
            // A frame not drawn is not asked for, so the core is bound to nothing for it - see EmuSen_CoreAPI.md §6.29.
            PictureOnDevice = !_skipRendering && m.TakeDeviceFrame();
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

        // The frame-end work in the runtime's order; a port whose oracle's order differs overrides it - see EmuSen_NativeCores.md §8.2's R4.
        protected virtual void EndFrame(CoreMachine m)
        {
            FrameLog.RecordFrame(TotalFrames, _readForFrameLog);
            // A frame the core says its console's cheat devices hold off for gets none of the host's - see EmuSen_CoreAPI.md §6.37.
            if (_machine is not null && _hostCheats.MasterEnabled && (m.FrameInfo.Flags & CoreInterface.FrameCheatsHeld) == 0) _hostCheats.ApplyAll(_read, _write);
            if (Freezes.Count > 0) Freezes.Enforce(_read, _write);
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
                    case CoreInterface.EventMediaSeek: ReadAhead(m, e); break;
                    case CoreInterface.EventMachineInfo:
                        m.ReadInfo();
                        _shape = m.FrameInfo;
                        _rawCurrent = false;
                        // A port added while the game runs, as by an adapter plugged in, has its buttons held from now.
                        _input.Grow(m.Info.Ports.Count);
                        break;
                }
            }
            if (_frame.Length < _shape.Bytes) _frame = new byte[_shape.Bytes];
        }

        // A drive's named place handed to the reader of the files Galaxia holds for it; a drive lent nothing has none, and a place outside its file is dropped - see EmuSen_CoreAPI.md §6.25.
        public static void ReadAhead(CoreMachine machine, CoreInterface.Event place)
        {
            if (machine.LoanOf((uint)(place.A >> 32)) is not DiscFiles disc) return;
            uint file = (uint)place.A;
            MappedFile? held = file == CoreInterface.FileImage ? disc.Image
                : file >= CoreInterface.FileMedia && file - CoreInterface.FileMedia < disc.Companions.Count ? disc.Companions[(int)(file - CoreInterface.FileMedia)] : null;
            if (held is not null && place.B >= 0 && place.B < held.Length) disc.ReadAhead(held, place.B);
        }

        private readonly List<string> _said = new();

        // A line of the runtime's own, kept until the session raises it as a log event of source "runtime" - see EmuSen_Backend.md §29.11, Q2.
        public void Say(string line)
        {
            lock (_said) _said.Add(line);
        }

        // The runtime's lines said since the last call, moved into into; false when there were none, which allocates nothing.
        public bool TakeSaid(List<string> into)
        {
            lock (_said)
            {
                if (_said.Count == 0) return false;
                into.AddRange(_said);
                _said.Clear();
                return true;
            }
        }

        private void KeepLog(IReadOnlyList<string> lines)
        {
            if (lines.Count == 0) return;
            LoggedLines += lines.Count;
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

        // The picture into the caller's buffer, rows repeated unless the frontend repeats them, the bytes ScreenWidth * 4 * ScreenHeight; one copy where the core's rows are packed - see EmuSen_Backend.md §14.
        public int CopyPicture(Span<byte> into)
        {
            if (_machine is not { } m) return 0;
            _shape = m.FrameInfo;
            int width = _shape.Width, height = _shape.Height, stride = _shape.Stride == 0 ? width * 4 : _shape.Stride;
            int repeat = RepeatRows ? Math.Max(1, _shape.RowRepeat) : 1;
            int bytes = width * 4 * height * repeat;
            if (into.Length < bytes) return bytes;
            if (repeat == 1 && stride == width * 4 && _shape.Bytes <= into.Length)
            {
                m.CopyFrame(into[..(int)_shape.Bytes]);
                return bytes;
            }
            if (_frame.Length < _shape.Bytes) _frame = new byte[_shape.Bytes];
            m.CopyFrame(_frame);
            for (int y = 0; y < height; y++)
                for (int r = 0; r < repeat; r++)
                    _frame.AsSpan(y * stride, width * 4).CopyTo(into.Slice((y * repeat + r) * width * 4, width * 4));
            return bytes;
        }

        // Stereo frames the core holds undrained.
        public int BufferedAudioFrames => _machine is { } m ? m.BufferedSamples / 2 : 0;

        // DequeueAudioSamples into the caller's buffer, allocating nothing; the stereo frames written and their rate.
        public int DrainAudio(Span<short> into, out int rate)
        {
            if (_machine is not { } m)
            {
                rate = AudioSampleRate;
                return 0;
            }
            int frames = m.DrainAudio(into, out rate);
            _audioRate = rate;
            return frames;
        }

        // The console's reset button; the halt a breakpoint left is let go with the frame it held - see EmuSen_CoreAPI.md §6.2.
        public virtual void Reset()
        {
            CoreMachine m = Machine;
            m.Reset();
            IsHaltedAtBreakpoint = false;
            Drain();
        }

        public void ReturnFrameBuffer(byte[] buffer)
        {
            if (_rawLending.Holds(buffer)) _rawLending.Return(buffer);
            else _lending.Return(buffer);
        }

        // The encoding of the picture before the palette a filter reads, kept by the core from the next frame where it offers it; null keeps none - see EmuSen_CoreAPI.md §6.30.
        public string? RawEncoding
        {
            get => System.Threading.Volatile.Read(ref _rawWanted);
            set => System.Threading.Volatile.Write(ref _rawWanted, value);
        }

        // The choice sent to the machine before a frame, and only when it changed; an encoding the machine does not list keeps nothing.
        private void SyncRaw(CoreMachine m)
        {
            string? wanted = RawEncoding;
            if (_rawCurrent && string.Equals(wanted, _rawAsked, StringComparison.Ordinal)) return;
            int index = -1;
            for (int i = 0; i < Math.Min(64, m.Info.Raw.Count) && wanted is not null; i++)
                if (m.Info.Raw[i].Encoding == wanted) { index = i; break; }
            m.RawSelect(index < 0 ? 0 : 1UL << index);
            (_rawAsked, _rawCurrent, _rawIndex) = (wanted, true, index);
        }

        // RawPicture into the caller's buffer, allocating nothing; its byte count, or -1 where the core kept none, written only when the buffer holds it.
        public long CopyRaw(Span<byte> into, out CoreInterface.RawFrame shape)
        {
            shape = default;
            if (_machine is not { } m || _rawIndex < 0 || _rawAsked is null || m.RawInfo((uint)_rawIndex) is not { } f) return -1;
            shape = f;
            if (into.Length >= f.Bytes) m.CopyRaw((uint)_rawIndex, into[..(int)f.Bytes]);
            return f.Bytes;
        }

        // The last frame's picture before the palette in RawEncoding, copied into an array lent as the RGBA frame is; null where the core kept none.
        public CoreRawPicture? RawPicture
        {
            get
            {
                if (_machine is not { } m || _rawIndex < 0 || _rawAsked is not { } encoding || m.RawInfo((uint)_rawIndex) is not { } f) return null;
                byte[] values = _rawLending.Lend((int)f.Bytes);
                m.CopyRaw((uint)_rawIndex, values);
                return new CoreRawPicture(encoding, f.Width, f.Height, (int)f.Element, f.Stride, values);
            }
        }

        public Func<IDeviceLend?>? DeviceLender { get; set; }

        public IDeviceLend? Lend => _machine?.Lend;

        public bool PictureOnDevice { get; private set; }

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
            int limit = Options.AudioBufferMaxSamples;
            if (_audioLimitSent == limit) return;
            m.SetAudioLimit(limit);
            _audioLimitSent = limit;
        }

        // LastFramePhases' times into the caller's buffer, allocating nothing; named false where the core times none and the one value is the whole frame.
        private readonly long[] _phaseScratch = new long[32];

        public int PhaseMilliseconds(Span<double> into, out bool named)
        {
            int n = _machine?.Phases(_phaseScratch) ?? 0;
            named = n > 0;
            if (n == 0)
            {
                if (into.Length > 0) into[0] = _lastFrameMs;
                return 1;
            }
            if (n > _phaseScratch.Length || n > into.Length) return n;
            for (int i = 0; i < n; i++) into[i] = _phaseScratch[i] / 1e6;
            return n;
        }

        // TELEMETRY's counters, in machine info's order, into the caller's buffer, allocating nothing; how many there are - see EmuSen_CoreAPI.md §6.34.
        public int Telemetry(Span<long> into) => _machine?.Telemetry(into) ?? 0;

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

        // A stream that can say its length is read into an array kept for the purpose, so a step of rewind allocates no state - see Mars_Native.md §7.2.
        public virtual void LoadState(Stream stream)
        {
            CoreMachine m = _machine ?? throw new InvalidOperationException("LoadState() called before LoadRom().");
            if (stream.CanSeek)
            {
                int length = checked((int)(stream.Length - stream.Position));
                if (_loadBuffer.Length < length) _loadBuffer = new byte[length];
                stream.ReadExactly(_loadBuffer, 0, length);
                LoadOverItsDisc(m, _loadBuffer.AsSpan(0, length));
            }
            else
            {
                using var copy = new MemoryStream();
                stream.CopyTo(copy);
                LoadOverItsDisc(m, copy.ToArray());
            }
            if (Library.Has(CoreInterface.CapMedia)) m.ReadInfo();
            _input.ResendPresence();
            _fresh = false;
            _cheatVersion = -1;
            IsHaltedAtBreakpoint = false;
            Drain();
        }

        // A state made over another disc of the set loads with that disc put back first; with none of the set, the drive is left as it was - see EmuSen_CoreAPI.md §6.23.
        private void LoadOverItsDisc(CoreMachine m, ReadOnlySpan<byte> state)
        {
            try
            {
                m.Load(state);
                return;
            }
            catch (InvalidDataException e) when (CoreMachine.StatusOf(e) == CoreInterface.StatusWrongMedia && CanChangeDisc)
            {
                int was = DiscIndex;
                foreach (int i in Enumerable.Range(0, _discs.Count).Append(-1).Where(i => i != was))
                {
                    InsertDisc(i, restore: true);
                    try
                    {
                        m.Load(state);
                        _closeLidAt = -1;
                        return;
                    }
                    catch (InvalidDataException again) when (CoreMachine.StatusOf(again) == CoreInterface.StatusWrongMedia) { }
                    catch
                    {
                        InsertDisc(was, restore: true);
                        throw;
                    }
                }
                InsertDisc(was, restore: true);
                throw;
            }
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
            if (m.Info.PatchLow is long low && m.Info.PatchHigh is long high) m.SetRomPatches(RomPatchTriples(_cheats, low, high));
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

        // (address, value, compare) for each byte ResolveRomPatches lists in [lo, hi], in its order; uint.MaxValue is no compare.
        public static uint[] RomPatchTriples(CheatRegistry cheats, long lo, long hi)
        {
            var words = new List<uint>();
            foreach (var patch in cheats.ResolveRomPatches(hi + 1))
            {
                if (patch.Address < lo) continue;
                words.Add(patch.Address);
                words.Add(patch.Value);
                words.Add(patch.Compare is byte compare ? compare : uint.MaxValue);
            }
            return words.ToArray();
        }

        // Apply now, between frames: every enabled poke, the ones the core gates at its frame's end too - see VenusRT_Native.md §66.3.
        public void ApplyCheats()
        {
            if (_machine is not { } m) return;
            RefreshCheats(m);
            _cheats.ApplyAll(ReadSpace, WriteSpace);
        }

        private uint? SpaceOf(string name) => SpaceNamed(name)?.Id;

        private CoreSpace? SpaceNamed(string name)
        {
            if (_machine is null) return null;
            IReadOnlyList<CoreSpace> spaces = _machine.Info.Spaces;
            for (int i = 0; i < spaces.Count; i++)
                if (string.Equals(spaces[i].Name, name, StringComparison.OrdinalIgnoreCase)) return spaces[i];
            return null;
        }

        // The ROM patches are brought up to the registry first, a version compare when nothing changed, so a read between frames answers as the next frame will.
        public virtual byte ReadSpace(string spaceName, int address)
        {
            if (_machine is null || SpaceOf(spaceName) is not uint space) return 0;
            RefreshCheats(_machine);
            Span<byte> one = stackalloc byte[1];
            _machine.ReadSpace(space, address, one);
            return one[0];
        }

        public virtual void WriteSpace(string spaceName, int address, byte value)
        {
            if (_machine is null || SpaceOf(spaceName) is not uint space) return;
            _machine.WriteSpace(space, address, stackalloc byte[] { value });
        }

        // A value in its space's byte order where the core states one, else the system's - see EmuSen_CoreAPI.md §6.39.
        private long ReadForFrameLog(string spaceName, int address, int width)
        {
            bool big = SpaceNamed(spaceName)?.ByteOrder switch { "big" => true, "little" => false, _ => _bigEndian };
            long value = 0;
            for (int i = 0; i < width; i++) value = big ? value << 8 | ReadSpace(spaceName, address + i) : value | (long)ReadSpace(spaceName, address + i) << (8 * i);
            return value;
        }

        // The schema's switch, count and choice settings that are not hidden, with the core's note for the value in use.
        public virtual IReadOnlyList<CoreSetting> Settings =>
            Library.Settings.Where(s => !s.Hidden).Select(s => s.AsCoreSetting() is { } row ? row with { Note = _ => _notes.GetValueOrDefault(s.Key) } : null).OfType<CoreSetting>().ToArray();

        public IReadOnlyList<CoreSettingDescriptor> SettingDescriptors => Library.Settings;

        public virtual string Get(string key) => _values.TryGetValue(key, out var v) ? v : "";

        // A run-time key reaches the machine between frames; a create-time one makes a fresh machine again, else waits for the next load - see EmuSen_Backend.md §22.
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
            string was = _values[key];
            _values[key] = value;
            if (s.CreateScope && _machine is not null && _fresh && was != value) MakeAgain(key, was);
        }

        // A value held for the next load alone, for a shim that sets a create-time key around its own load.
        protected void Keep(string key, string value) => _values[key] = value;

        // The machine made again as it was made, with the values now held; a refusal puts the old value back and keeps the old machine.
        private void MakeAgain(string key, string was)
        {
            try
            {
                if (StartedEmpty is { } system) StartConsole(system);
                else LoadRom(RomPath);
            }
            catch (CoreRefusedException e)
            {
                _values[key] = was;
                throw new ArgumentException($"{Info.Name} refused {key} at create: {e.Message}", e);
            }
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
            ReleaseMedia();
            _machine?.Dispose();
            _machine = null;
            _lending.Close();
            _rawLending.Close();
        }
    }
}
