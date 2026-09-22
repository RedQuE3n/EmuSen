using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Common;
using EmuSen.Common.Firmware;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus.Debug;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Serenity.Dashboards;
using EmuSen.Mistress.Input;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Threading;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Galaxia.Input;
using EmuSen.Audio;

namespace EmuSen.Mistress.Views
{
    public partial class MainWindow : ToolWindow
    {
        // Paced off the core's own rate, not a flat 60 - see Venus_CPU.md §8.5b.
        private TimeSpan FrameInterval => TimeSpan.FromSeconds(1.0 / (_session?.FrameRateHz ?? 60.0988));

        // One object per command, carrying its own enabled and checked state - see EmuSen_Settings_Reference.md §4.12.
        private readonly LunaAction _pause;
        private readonly LunaAction _reset;
        private readonly LunaAction _closeGame;
        private readonly LunaAction _saveState;
        private readonly LunaAction _loadState;
        private readonly LunaAction _speedMenu;
        private readonly LunaAction _slotMenu;
        private readonly LunaAction _fullscreen;
        private readonly LunaAction _hardwareDashboard;
        private readonly ActionGroup _speeds = new();
        private readonly ActionGroup _slots = new();

        private EmulatorSession? _session;

        // Over the frame only, so the pointer stays visible on the menu - see §4.20.
        private IdleCursor? _idleCursor;
        private FileDrop? _fileDrop;

        // Held, not edge-triggered - see EmuSen_Rewind_And_FastForward.md §4.
        private volatile bool _turboHeld;
        private volatile bool _rewindHeld;

        private readonly EmuSen.Common.SpeedController _speed = new();
        private readonly EmuSen.Common.RewindBuffer _rewind = new() { Enabled = true };

        // What the Speed menu picked, read by EmulationLoop - see EmuSen_Settings_Reference.md §4.13.
        private volatile int _baseSpeedPercent = EmuSen.Common.SpeedController.NormalPercent;

        // Which save slot Save/Load State act on, 1-8 - see §4.13.
        private int _stateSlot = 1;
        private const int StateSlots = 8;

        // Gamepad polling only, on the UI thread - see EmuSen_Settings_Reference.md §4.10.
        private DispatcherTimer? _timer;

        // RunFrame off the UI thread entirely - see EmuSen_Settings_Reference.md §4.21.
        private Thread? _emuThread;
        private volatile bool _running;

        // Signalled means running; a stop always Sets before Join - see `man pause` and §4.21.
        private readonly ManualResetEventSlim _pauseSignal = new(initialState: true);

        public bool IsPaused => !_pauseSignal.IsSet;

        // Save and load, handed to the thread that owns the core and run between frames or woken while paused - see §4.21a.
        private readonly System.Collections.Concurrent.ConcurrentQueue<Action<EmulatorSession>> _coreRequests = new();
        private readonly ManualResetEventSlim _requestSignal = new(initialState: false);
        private WaitHandle[]? _parkedWakes;

        // Coalescing hand-off to the UI thread - see EmuSen_Serenity.md §4.
        private sealed class FrameData
        {
            public required byte[] Pixels;
            public required int Width;
            public required int Height;
            public required int RowRepeat;
        }
        // Was written out here, in Hotaru and in Serenity, all three with one defect - LunaP.md §22.1.
        private readonly Latest<FrameData> _frames;

        // Rebuilt on every LoadRom, so the console never sees a swapped-out core - see §4.12.
        private IDebugTarget? _debugTarget;

        // At most one, reused and brought forward rather than duplicated; cleared on Closed.
        private readonly WindowSlot<DianaOSConsoleWindow> _consoleWindow = new();

        // Same rule, opened by typing `coretop` rather than from a menu item.
        private readonly WindowSlot<CoretopWindow> _coretopWindow = new();

        // Same rule, with no target to update - see `man vstop`.
        private readonly WindowSlot<VstopWindow> _vstopWindow = new();

        // Same rule, refreshed rather than re-targeted - see EmuSen_Settings_Reference.md §4.14.
        private readonly WindowSlot<ActiveCheatsWindow> _activeCheatsWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern, retained so a console switch can retarget it.
        private readonly WindowSlot<CheatDatabaseWindow> _cheatDatabaseWindow = new();

        // Owned here so a cheat list outlives the core a Reset rebuilds - see §4.14.
        private readonly CheatRegistry _cheats = new();

        // Which ROM the cheats are for; not _currentRomPath, which a close resets - see §4.14.
        private string? _cheatsRomPath;

        // Cheats are poked from the thread that owns the core - see §4.15.
        private volatile bool _applyCheatsPending;

        private string? _currentRomPath;
        private string? _currentDisplayName; // for restoring StatusText's "Running: ..." text exactly after a pause, without reformatting from _currentRomPath
        private readonly ControllerKeyBindings _keyBindings =
            ControllerKeyBindings.Load(EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));
        private readonly GamepadBindings _gamepadBindings =
            GamepadBindings.Load(EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));

        // Follows the loaded ROM; catalog order stands in before one loads - see EmuSen_Input.md §5.1.
        private string _activeConsole = EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder[0].Console;
        private readonly HotkeyBindingMap _hotkeyBindings = HotkeyBindingMap.Load();
        private readonly AppSettings _appSettings = AppSettings.Load();
        private readonly GraphicsConfig _graphics = GraphicsConfig.Load();
        private readonly GamepadManager _gamepad;

        // Constructed once and reused across every load, same lifetime as _gamepad.
        private readonly AudioPlayer _audioPlayer;

        // Captured before anything redirects - see EmuSen_Settings_Reference.md §4.22.
        private readonly TextWriter _originalConsoleOut = Console.Out;
        private CategorizedLogWriter? _activeLogWriter;

        // Tracked separately and OR'd: either device works at any time.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<PadControl>().Length];
        private readonly bool[] _gamepadHeld = new bool[Enum.GetValues<PadButton>().Length];

        public MainWindow()
        {
            _frames = new Latest<FrameData>(PresentPendingFrame);
            _pause = new LunaAction("_Pause", _ => TogglePause()) { IsCheckable = true };
            _reset = new LunaAction("_Reset", ResetEmulation);
            _closeGame = new LunaAction("_Close Game", ShowLibrary);
            _saveState = new LunaAction("Save _State", SaveState);
            _loadState = new LunaAction("_Load State", LoadState);
            _speedMenu = new LunaAction("Spee_d", () => { });
            _slotMenu = new LunaAction("State Sl_ot", () => { });
            _fullscreen = new LunaAction("_Fullscreen", a => IsFullScreen = a.IsChecked) { IsCheckable = true };
            _hardwareDashboard = new LunaAction("_Hardware Dashboard...", () => OpenCoretopWindow(_debugTarget));
            InitializeComponent();
            SetUpLibraryScreen();
            BuildMenus();
            // The tick follows the window, so a window-manager key cannot leave it lying - see §4.19.
            FullScreenChanged += on => _fullscreen.IsChecked = on;
            _gamepad = new GamepadManager(_gamepadBindings.For(_activeConsole));
            // Endymion is a leaf and reads no globals, so the settings come from here - see EmuSen_Audio_Sync.md §7.1.
            _audioPlayer = new AudioPlayer(
                AudioSettings.SampleRate, AudioSettings.OutputTargetLatencyMs, AudioSettings.RateControlMaxDeviation);
            // On GameFrame, not the window: a window-level flag cannot say that - see §4.20.
            _idleCursor = new IdleCursor(GameFrame);
            SetUpHud();
            SetUpBackgroundPause();
            _fileDrop = new FileDrop(this, paths => _ = OpenDroppedRomAsync(paths[0]))
            {
                Accept = paths => paths.Count == 1,
            };

            Closing += (_, _) =>
            {
                // Both hold handlers on controls that outlive this scope - see §4.20.
                _idleCursor?.Dispose();
                _fileDrop?.Dispose();
                _timer?.Stop();
                _padTimer?.Stop();
                StopEmulationThread();
                _session?.SaveSram();
                WriteResumeState();
                RecordPlayTime();
                StopOnlineCovers();
                CloseRecords();
                _gamepad.Dispose();
                _audioPlayer.Dispose();
                StopLogging();

                // Here rather than on Changed, which fires per keystroke - see EmuSen_Settings_Reference.md §4.23.
                _appSettings.LibrarySearch = LibraryFilter.SearchText;
                _appSettings.Save();
            };

            _gamepad.AnalogStickAsDpad = _appSettings.AnalogStickAsDpad;
            _gamepad.StickDeadzone = _appSettings.StickDeadzone;

            RefreshLibrary();
            BuildSaveSlotItems();
            StartPadNavigation();

            // Tunnel, not bubbling, or focus navigation eats the arrows - see EmuSen_Settings_Reference.md §4.2.
            AddHandler(KeyDownEvent, (_, e) => SetButtonFromKey(e.Key, pressed: true, e), RoutingStrategies.Tunnel, handledEventsToo: true);
            AddHandler(KeyUpEvent, (_, e) => SetButtonFromKey(e.Key, pressed: false, e), RoutingStrategies.Tunnel, handledEventsToo: true);
        }

        // A focused text field owns the whole keyboard - see EmuSen_Settings_Reference.md §4.17.
        private static bool TypingIntoATextField(RoutedEventArgs e) =>
            e.Source is Visual source && source.FindAncestorOfType<TextBox>(includeSelf: true) is not null;

        private void SetButtonFromKey(Key key, bool pressed, KeyEventArgs e)
        {
            if (TypingIntoATextField(e)) return;

            // A suspended game must not collect the keys used to browse the library - see EmuSen_Settings_Reference.md §4.18.
            if (!LibraryView.IsVisible && _keyBindings.For(_activeConsole).TryGetControl(key, out PadControl control))
            {
                _keyboardHeld[(int)control] = pressed;
                if (PadControls.IsButton(control, out PadButton button)) ApplyButtonState(button);
                ApplyAxes();
                // Or the menu bar, the only focusable control here, also gets the key - see EmuSen_Settings_Reference.md §4.24.
                e.Handled = true;
                return;
            }

            if (!_hotkeyBindings.TryGetAction(key, out HotkeyAction action)) return;

            // Held vs one-shot - see EmuSen_Rewind_And_FastForward.md §4, EmuSen_Settings_Reference.md §4.3.
            if (HotkeyBindingMap.IsHeld(action))
            {
                if (action == HotkeyAction.FastForward)
                {
                    if (pressed && !_turboHeld) Notify("Fast forward");
                    _turboHeld = pressed;
                }
                else if (action == HotkeyAction.Rewind) _rewindHeld = pressed;
                e.Handled = true;
                return;
            }

            if (!pressed) return;
            switch (action)
            {
                case HotkeyAction.SaveState: SaveState(); break;
                case HotkeyAction.LoadState: LoadState(); break;
                case HotkeyAction.ExitToLibrary: ToggleLibrary(); break;
                case HotkeyAction.Screenshot: TakeScreenshot(); break;
                // Nothing to pause while the library is up: it is already suspended - see §4.18.
                case HotkeyAction.TogglePause: if (!LibraryView.IsVisible) TogglePause(); break;
                case HotkeyAction.ToggleFullscreen:
                    WindowState = WindowState == WindowState.FullScreen ? WindowState.Normal : WindowState.FullScreen;
                    break;
            }
            e.Handled = true;
        }

        // Through ICore, not the bus - which console's pad this reaches is the core's business.
        private void ApplyButtonState(PadButton button)
        {
            if (_session is not { IsRomLoaded: true }) return;
            bool held = _keyboardHeld[(int)PadControls.From(button)] || _gamepadHeld[(int)button];
            _session.SetButton(0, button, held);
            if (_appSettings.MirrorPlayer1ToPlayer2) _session.SetButton(1, button, held);
        }

        // Every axis the console reads, from the pad's own axes and the keys standing in for them - see EmuSen_Input.md §7.3.
        private void ApplyAxes()
        {
            if (_session is not { IsRomLoaded: true }) return;

            foreach (PadAxis axis in _session.SupportedAxes)
            {
                double value = PadControls.Resolve(axis, _gamepad.Axis(axis), Held);
                _session.SetAxis(0, axis, value);
                if (_appSettings.MirrorPlayer1ToPlayer2) _session.SetAxis(1, axis, value);
            }
        }

        private bool Held(PadControl control) =>
            _keyboardHeld[(int)control] || (PadControls.IsButton(control, out PadButton button) && _gamepadHeld[(int)button]);

        private void PollGamepad()
        {
            _gamepad.Poll();

            // A menu over the game, or the button that closed one, is not the game's - see §4.29.
            if (PadBelongsToTheInterface()) return;

            foreach (PadButton button in Enum.GetValues<PadButton>())
            {
                bool held = _gamepad.IsPressed(button);
                if (held != _gamepadHeld[(int)button])
                {
                    _gamepadHeld[(int)button] = held;
                    ApplyButtonState(button);
                }
            }

            // A stick moves without crossing any threshold, so its axes are sent every poll rather than on change.
            ApplyAxes();
        }

        private async Task OpenRomAsync()
        {
            // One entry per core in this build, so a new core needs no edit here.
            FilePickerFileType[] types = EmuSen.Cores.CoreCatalog.Cores
                .Select(c => new FilePickerFileType($"{c.DisplayName} ROMs")
                {
                    Patterns = c.Extensions.Select(e => "*" + e).ToArray()
                })
                .Append(new FilePickerFileType("All files") { Patterns = new[] { "*" } })
                .ToArray();

            string? startIn = Directory.Exists(_appSettings.RomDirectory) ? _appSettings.RomDirectory : null;
            if (await Dialogs.PickFileAsync(this, "Open ROM", types, startIn) is not { } file) return;

            await StartGameAsync(file.Path, file.Name);
        }

        // Runs before LoadRom because afterwards is too late; declining is fine - see EmuSen_Firmware.md §3.
        private async Task PromptForMissingFirmwareAsync(string romPath)
        {
            foreach (FirmwareRequest request in EmulatorSession.MissingFirmwareFor(romPath))
            {
                var types = new[]
                {
                    new FilePickerFileType($"{request.ChipName} firmware") { Patterns = new[] { request.FileName, "*.rom", "*.bin" } },
                    new FilePickerFileType("All files") { Patterns = new[] { "*" } },
                };

                string title = $"{request.ChipName} firmware needed - pick {request.FileName} ({request.Size:N0} bytes), or cancel to play without it";
                if (await Dialogs.PickFileAsync(this, title, types) is not { } chosen)
                {
                    StatusText.Text = $"No {request.ChipName} firmware selected - that chip will not be emulated.";
                    continue;
                }

                StatusText.Text = FirmwareLibrary.Install(request, chosen.Path)
                    ? $"Installed {request.ChipName} firmware."
                    : $"{chosen.Name} is not a {request.ChipName} dump ({request.Size:N0} bytes expected) - that chip will not be emulated.";
            }
        }

        // The same sequence Open ROM... runs, so a drop cannot skip the firmware prompt - see §4.20.
        private async Task OpenDroppedRomAsync(string path)
        {
            await StartGameAsync(path, System.IO.Path.GetFileName(path));
        }

        // Lists the ROM directory directly, as an alternative to the OS picker - see §4.11.
        private async Task BrowseRomsAsync()
        {
            string? selected = await new RomBrowserWindow(_appSettings.RomDirectory).ShowDialog<string?>(this);
            if (selected is null) return;

            await StartGameAsync(selected, Path.GetFileName(selected));
        }

        private void ShowControllerBindings()
        {
            var window = new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad, _appSettings, _hotkeyBindings,
                _session is null ? null : _activeConsole);
            // A rebind has to reach the menu, or it advertises the old key - see §4.19.
            window.Closed += (_, _) => SyncMenuState();
            window.Show(this);
        }

        private void ShowGraphicsSettings()
        {
            var window = new GraphicsSettingsWindow(_graphics, console =>
            {
                // On the emulation thread, between frames, and only when the running game is that console's - see §4.26.
                if (_session is not null && console == _activeConsole) RequestOnEmulationThread(session => ApplyConsoleSettings(session, console));
                if (_session is not null && console == _activeConsole) ApplyScreenFilter(console);
            }, _session is null ? null : _activeConsole);
            window.Show(this);
        }

        // On the UI thread: the filter belongs to the control that draws, not to the core - see EmuSen_Settings_Reference.md §4.40.
        private void ApplyScreenFilter(string console)
        {
            GameFrame.ActiveEffect = EmuSen.Serenity.Shaders.ScreenFilters.ByName(_graphics.Value(console, GraphicsSettingsWindow.ScreenFilterKey));
            GameFrame.InvalidateVisual();
        }

        // Every setting the core offers, from the config or its default; a value the core refuses falls back to the default - see §4.26.
        private void ApplyConsoleSettings(EmulatorSession session, string console)
        {
            if (session.Core is not EmuSen.Cores.ICoreSettings settings) return;

            foreach (EmuSen.Cores.CoreSetting setting in settings.Settings)
            {
                string value = _graphics.Value(console, setting.Key) ?? setting.Default;
                try { settings.Set(setting.Key, value); }
                catch (ArgumentException) { settings.Set(setting.Key, setting.Default); }
            }
        }

        private void ShowPreferences()
        {
            // Non-modal, so re-scan on close rather than leaving a stale library behind it.
            var window = new PreferencesWindow(_appSettings);
            window.Closed += (_, _) => { ScanArtwork(); ApplyOnlineCovers(); if (LibraryView.IsVisible) RefreshLibrary(); };
            window.Show(this);
        }

        private void ShowDebugLogging()
        {
            new DebugSettingsWindow().Show(this);
        }

        // Never needs a ROM: it manages a folder and a list, not a session. See §4.14.
        private void ShowCheatDatabase()
        {
            _cheatDatabaseWindow.Show(this, () => new CheatDatabaseWindow(_appSettings, () => _cheats,
                ConsoleCodecs(SelectedConsole).AutoDetect,
                () => _activeCheatsWindow.Current?.Refresh(),
                ShowActiveCheats,
                () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems,
                SelectedConsole));
        }

        

        // The menu item and the database window's button are the same door - see §4.14.
        private void ShowActiveCheats()
        {
            var codecs = ConsoleCodecs(SelectedConsole);
            _activeCheatsWindow.Show(this, () => new ActiveCheatsWindow(_cheats,
                codecs.AutoDetect,
                codecs.Explicit,
                RequestCheatApply,
                () => CheatListName(_cheatsRomPath),
                SelectedConsole),
                refresh: w => w.Refresh());
        }

        // Hands the poke to the emulation thread; false means there was no core - see §4.15.
        private bool RequestCheatApply()
        {
            if (_debugTarget is null) return false;

            // Paused parks that thread outside the core, so this can happen now - see §4.15.
            if (IsPaused) _debugTarget.ApplyCheats();
            else _applyCheatsPending = true;

            return true;
        }

        // Sanitized rather than validated, because this name is derived and not typed - see §4.15.
        private static string? CheatListName(string? romPath)
        {
            if (string.IsNullOrEmpty(romPath)) return null;

            var name = new string(System.IO.Path.GetFileNameWithoutExtension(romPath)
                .Select(c => System.IO.Path.GetInvalidFileNameChars().Contains(c) ? '_' : c).ToArray());
            return CheatFile.IsValidName(name) ? name : null;
        }

        // Whatever was saved for this ROM, brought back; returns how many - see §4.15.
        private int LoadSavedCheatsFor(string path)
        {
            if (CheatListName(path) is not string name) return 0;

            CheatFile? saved = CheatFile.For(name).Load();
            if (saved is null) return 0;

            (int loaded, _) = _cheats.LoadFrom(saved);
            _activeCheatsWindow.Current?.Refresh();
            return loaded;
        }

        private void ShowShellConsole()
        {
            _consoleWindow.Show(this, () => new DianaOSConsoleWindow(_debugTarget, MakeEmulationControlCommands(),
                _session?.CheatAutoDetectCodec, _session?.CheatExplicitCodec, _session?.CpuTraceSwitch));
        }

        // Built fresh per console window, so the delegates never outlive a MainWindow.
        private IEnumerable<IDianaOSCommand> MakeEmulationControlCommands() => new IDianaOSCommand[]
        {
            new PauseCommand(PauseEmulation, () => IsPaused),
            new ResumeCommand(ResumeEmulation, () => IsPaused),
            new CoretopWindowCommand(OpenCoretopWindow),
            new VstopWindowCommand(OpenVstopWindow),
            new FeedCommand(() => Activate()),
            new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.StateCommand(SaveStateFromConsole, LoadStateFromConsole, () => CurrentStatePath ?? ""),
        };

        // Not method groups: _session can still be null at construction time - see `man state`.
        private void SaveStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.SaveState(path);
        }

        private void LoadStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.LoadState(path);
            _rewind.Clear(); // a discontinuous jump - see §1.4
        }

        // Triggered from inside the console window itself (typing `coretop`) rather than a menu item.
        private void OpenCoretopWindow(IDebugTarget? target) =>
            _coretopWindow.Show(this, () => new CoretopWindow(target), refresh: w => w.UpdateTarget(target));

        // No target to hand over or refresh, unlike OpenCoretopWindow above.
        private void OpenVstopWindow() => _vstopWindow.Show(this, () => new VstopWindow());

        // The same tree `state save` writes to, overridable in Preferences.
        private string? CurrentStatePath => StatePathForSlot(_stateSlot);

        // Slot 1 is the plain <rom>.state every other frontend writes - see §4.13.
        private string? StatePathForSlot(int slot) =>
            _currentRomPath is null
                ? null
                : SaveLibrary.StatePathFor(_currentRomPath, slot, _appSettings.StateDirectory);

        private void SaveState()
        {
            if (_session is not { IsRomLoaded: true } || CurrentStatePath is not string path)
            {
                StatusText.Text = "Save State: no ROM loaded";
                return;
            }

            int slot = _stateSlot;
            string rom = _currentRomPath!;
            RequestOnEmulationThread(session =>
            {
                string status;
                try
                {
                    System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
                    session.SaveState(path);
                    WriteStatePicture(session, path);
                    WriteStateRecord(session, path, rom);
                    status = $"State saved to slot {slot}: {System.IO.Path.GetFileName(path)}";
                }
                catch (Exception ex)
                {
                    status = $"Save State failed: {ex.Message}";
                }

                // SyncMenuState because that slot has a timestamp now.
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; SyncMenuState(); Notify(status.StartsWith("State saved", StringComparison.Ordinal) ? $"State saved to slot {slot}" : status); });
            });
        }

        private void LoadState() => _ = LoadStateAsync();

        // Runs straight through to the request unless the state's record gives a reason to stop or to ask - see EmuSen_Settings_Reference.md §4.37.
        private async Task LoadStateAsync()
        {
            if (_session is not { IsRomLoaded: true } || CurrentStatePath is not string path || _currentRomPath is not string rom)
            {
                StatusText.Text = "Load State: no ROM loaded";
                return;
            }

            if (!System.IO.File.Exists(path))
            {
                StatusText.Text = $"Load State: slot {_stateSlot} is empty";
                return;
            }

            StateRecord? record = StateRecord.Read(path);
            if (Refusal(record, rom, _session) is string refused)
            {
                StatusText.Text = $"Load State: {refused}";
                Notify("State not loaded");
                return;
            }
            if (!await SameGameOrConfirmedAsync(record, rom)) return;

            int slot = _stateSlot;
            RequestOnEmulationThread(session =>
            {
                string status;
                try
                {
                    session.LoadState(path);
                    _rewind.Clear(); // a discontinuous jump - see §1.4
                    _audioPlayer.RateControl.Reset();
                    status = $"State loaded from slot {slot}: {System.IO.Path.GetFileName(path)}";
                }
                catch (Exception ex)
                {
                    status = $"Load State failed: {ex.Message}{Provenance(record)}";
                }

                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(status.StartsWith("State loaded", StringComparison.Ordinal) ? $"State loaded from slot {slot}" : status); });
            });
        }

        private void RequestOnEmulationThread(Action<EmulatorSession> request)
        {
            _coreRequests.Enqueue(request);
            _requestSignal.Set();
        }

        // Only on the emulation thread; reset first, so a request queued while these run wakes the next wait - see §4.21a.
        private void RunCoreRequests(EmulatorSession session)
        {
            _requestSignal.Reset();
            while (_coreRequests.TryDequeue(out Action<EmulatorSession>? request)) request(session);
        }

        // The four menus - shortcuts stay with HotkeyBindingMap, see EmuSen_Settings_Reference.md §4.19.
        private void BuildMenus()
        {
            _speedMenu.Submenu = new LunaMenu("Spee_d",
                _speeds.Add(new LunaAction("_Normal", () => SetBaseSpeed(EmuSen.Common.SpeedController.NormalPercent))),
                _speeds.Add(new LunaAction("_Fast Forward", () => SetBaseSpeed(_speed.TurboPercent))),
                _speeds.Add(new LunaAction("_Slow Motion", () => SetBaseSpeed(_speed.SlowMotionPercent))),
                _speeds.Add(new LunaAction("_Unthrottled", () => SetBaseSpeed(EmuSen.Common.SpeedController.UnthrottledPercent))));

            BuildSaveSlotItems();

            MenuStrip.SetMenus(
                new LunaMenu("_File",
                    new LunaAction("_Open ROM...", () => _ = OpenRomAsync()),
                    new LunaAction("_Browse ROMs...", () => _ = BrowseRomsAsync()),
                    LunaAction.Separator(),
                    new LunaAction("Game _Library", ShowLibrary),
                    _newCollection,
                    LunaAction.Separator(),
                    new LunaAction("E_xit", () => Close())),
                new LunaMenu("_Emulation",
                    _pause, _reset, _closeGame,
                    LunaAction.Separator(),
                    _speedMenu,
                    LunaAction.Separator(),
                    _saveState, _loadState, _slotMenu),
                new LunaMenu("_View", _asGrid!, _asList!,
                    new LunaAction("_Larger Covers", () => TileScale.Value = Math.Min(MaximumTileScale, TileScale.Value + 0.25)),
                    new LunaAction("S_maller Covers", () => TileScale.Value = Math.Max(MinimumTileScale, TileScale.Value - 0.25)),
                    LunaAction.Separator(), _fullscreen),
                new LunaMenu("_Settings",
                    new LunaAction("_Controller Bindings...", ShowControllerBindings),
                    new LunaAction("_Graphics Settings...", ShowGraphicsSettings),
                    new LunaAction("_Debug Logging...", () => new DebugSettingsWindow().Show(this)),
                    new LunaAction("_Preferences...", ShowPreferences),
                    new LunaAction("Chea_t Database...", ShowCheatDatabase),
                    new LunaAction("_Active Cheats...", ShowActiveCheats),
                    LunaAction.Separator(),
                    _hardwareDashboard,
                    // Reports on the host VM, so unlike the one above it never needs a ROM - see `man vstop`.
                    new LunaAction("_Runtime Dashboard...", OpenVstopWindow),
                    new LunaAction("_DianaOS Console...", ShowShellConsole)));

            SyncMenuState();
        }

        // SetMenus draws a gesture without binding it, so a rebind moves the label only - see §4.19.
        private void ShowHotkeysOnTheMenu()
        {
            Gesture(_pause, HotkeyAction.TogglePause);
            Gesture(_saveState, HotkeyAction.SaveState);
            Gesture(_loadState, HotkeyAction.LoadState);
            Gesture(_fullscreen, HotkeyAction.ToggleFullscreen);
            Gesture(_closeGame, HotkeyAction.ExitToLibrary);
        }

        private void Gesture(LunaAction action, HotkeyAction bound)
        {
            action.Shortcut = _hotkeyBindings.ActionToKey.TryGetValue(bound, out Key key) ? new KeyGesture(key) : null;
            action.HelpText = HotkeyBindingMap.DisplayName(bound);
        }

        // Called wherever the state actually changes, not when a menu opens - see §4.12.
        private void SyncMenuState()
        {
            bool running = _session is { IsRomLoaded: true };
            _pause.IsEnabled = running;
            _reset.IsEnabled = running;
            _closeGame.IsEnabled = running;
            _speedMenu.IsEnabled = running;
            _saveState.IsEnabled = running;
            _loadState.IsEnabled = running;
            _slotMenu.IsEnabled = running;
            _pause.IsChecked = running && IsPaused;

            // Checking a member unchecks its siblings and runs no handler; ActionGroup.Checked is read-only.
            _speeds.Members[SpeedIndex(_baseSpeedPercent)].IsChecked = true;
            BuildSaveSlotItems();

            _saveState.Text = $"Save _State (slot {_stateSlot})";
            _loadState.Text = $"_Load State (slot {_stateSlot})";
            _hardwareDashboard.IsEnabled = _debugTarget is not null;

            ShowHotkeysOnTheMenu();
            SyncHud();
        }

        private int SpeedIndex(int percent)
        {
            if (percent == _speed.TurboPercent) return 1;
            if (percent == _speed.SlowMotionPercent) return 2;
            if (percent == EmuSen.Common.SpeedController.UnthrottledPercent) return 3;
            return 0;
        }

        private void SetBaseSpeed(int percent)
        {
            _baseSpeedPercent = percent;
            _audioPlayer.RateControl.Reset(); // the device's pacing target just moved - see EmuSen_Audio_Sync.md §3.2
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Speed: {DescribeSpeed(percent)}";
            SyncMenuState();
        }

        private static string DescribeSpeed(int percent) =>
            percent == EmuSen.Common.SpeedController.UnthrottledPercent ? "unthrottled" : $"{percent}%";

        // Rebuilt on every sync so a slot saved since last time shows its new stamp.
        private void BuildSaveSlotItems()
        {
            if (_slots.Members.Count == 0)
            {
                for (int slot = 1; slot <= StateSlots; slot++)
                {
                    int captured = slot;
                    _slots.Add(new LunaAction($"Slot {slot}", () => SelectStateSlot(captured)));
                }
                _slotMenu.Submenu = new LunaMenu("State Sl_ot", _slots.Members);
            }

            for (int slot = 1; slot <= StateSlots; slot++)
            {
                _slots.Members[slot - 1].Text = $"Slot {slot} - {DescribeSlot(slot)}";
            }
            _slots.Members[_stateSlot - 1].IsChecked = true;
        }

        private string DescribeSlot(int slot)
        {
            if (StatePathForSlot(slot) is not string path || !System.IO.File.Exists(path)) return "empty";
            return System.IO.File.GetLastWriteTime(path).ToString("yyyy-MM-dd HH:mm");
        }

        private void SelectStateSlot(int slot)
        {
            _stateSlot = slot;
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Save slot {slot} selected ({DescribeSlot(slot)})";
            SyncMenuState();
        }

        // Another game's addresses would quietly poke this one's RAM; a Reset keeps them.
        private int DropCheatsFromAnotherGame(string path)
        {
            bool sameGame = string.Equals(_cheatsRomPath, path, StringComparison.Ordinal);
            if (_cheatsRomPath is not null && !sameGame)
            {
                _cheats.Clear();
                _activeCheatsWindow.Current?.Refresh();
            }

            _cheatsRomPath = path;

            // Not when the list is already in hand: a Reset must not overwrite edits since - see §4.15.
            return !sameGame && _cheats.GetCheats().Count == 0 ? LoadSavedCheatsFor(path) : 0;
        }

        // A power cycle, not a soft reset - see EmuSen_Settings_Reference.md §4.12.
        private void ResetEmulation()
        {
            if (_session is not { IsRomLoaded: true }) return;
            if (_currentRomPath is not string path || _currentDisplayName is not string displayName) return;

            ResumeEmulation(); // a reset always comes back running, however it was paused
            LoadGame(path, displayName, resumeFrom: null, reset: true);
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Reset: {displayName}";
        }

        // Everything that must happen before _session can be replaced or dropped.
        private void ShutDownCurrentSession(bool leaving = true)
        {
            _timer?.Stop();
            StopEmulationThread(); // must fully stop before _session changes - see that method's own comment
            _session?.SaveSram(); // flush whatever was previously running before switching
            // A Reset starts the same game again, so it is not leaving it - see EmuSen_Settings_Reference.md §4.31.
            if (leaving)
            {
                WriteResumeState();
                RecordPlayTime();
            }
            // On the OLD session: StartLogging's own StopLogging would take the new one - see §4.22.
            _session?.FlushVerboseLogs();
        }

        private void LoadRom(string path, string displayName) => LoadGame(path, displayName, resumeFrom: null, reset: false);

        // resumeFrom is a state to load before the first frame runs - see EmuSen_Settings_Reference.md §4.31.
        private void LoadGame(string path, string displayName, string? resumeFrom, bool reset)
        {
            ShutDownCurrentSession(leaving: !reset);
            int restoredCheats = DropCheatsFromAnotherGame(path);

            try
            {
                _session = new EmulatorSession { Cheats = _cheats };
                // Off the path, not the session: no core exists yet to ask - see EmuSen_Multicore.md §12.
                StartLogging(EmuSen.Cores.CoreCatalog.ConsoleForRom(path) ?? "Unknown");
                _session.LoadRom(path);

                // The pad this ROM's console reads, not whatever the last one used.
                _activeConsole = _session.CoreName;

                // What the graphics window holds for this console, or each setting's own default - see EmuSen_Settings_Reference.md §4.26.
                ApplyConsoleSettings(_session, _activeConsole);
                ApplyScreenFilter(_activeConsole);
                // A half-loaded state is not a machine to run on, so the game starts again from nothing.
                if (resumeFrom is not null && TryResume(_session, resumeFrom, path) is string resumeFailure)
                {
                    _session = null;
                    LoadGame(path, displayName, resumeFrom: null, reset: false);
                    StatusText.Text = resumeFailure;
                    return;
                }
                _gamepad.Bindings = _gamepadBindings.For(_activeConsole);
                _gamepad.LeftStickIsAnalog = _session.SupportedAxes.Contains(PadAxis.LeftX);

                _rewind.Clear(); // a discontinuous jump - see §1.4
                _audioPlayer.RateControl.Reset();

                // Built by CoreFactory alongside the core, so this window names no concrete target.
                _debugTarget = _session.DebugTarget;
                _consoleWindow.Current?.UpdateTarget(_debugTarget, displayName,
                    _session.CheatAutoDetectCodec, _session.CheatExplicitCodec, _session.CpuTraceSwitch);
                _coretopWindow.Current?.UpdateTarget(_debugTarget);

                GameFrame.IsVisible = true;
                LibraryView.IsVisible = false;

                StatusText.Text = restoredCheats > 0
                    ? $"Running: {displayName}  ({restoredCheats} saved cheat(s) restored)"
                    : $"Running: {displayName}";
                _currentRomPath = path;
                _currentDisplayName = displayName;
                if (!reset) RecordStart(path);

                // Gamepad polling only; emulation is _emuThread's - see §4.21.
                _timer = new DispatcherTimer { Interval = FrameInterval };
                _timer.Tick += (_, _) => PollGamepad();
                _timer.Start();

                // The frame control stretches repeated rows on the GPU, so a core that can is asked to send each once - see EmuSen_Multicore.md §15.
                if (_session.Core is EmuSen.Cores.IRepeatedRows repeated) repeated.RepeatRows = false;

                StartEmulationThread();
                SyncMenuState();
            }
            catch (Exception ex)
            {
                _session = null;
                // Back to the list rather than a black viewport with only a status line.
                ShowLibrary();
                StatusText.Text = $"Failed to load {displayName}: {ex.Message}";
            }
        }

        // Steps out of a game without unloading it, unlike ShowLibrary - see EmuSen_Settings_Reference.md §4.18.
        private void ToggleLibrary()
        {
            if (_session is not { IsRomLoaded: true }) return;

            if (LibraryView.IsVisible)
            {
                LibraryView.IsVisible = false;
                GameFrame.IsVisible = true;
                ResumeEmulation();
                return;
            }

            PauseEmulation(); // before the screens swap, so no frame runs unwatched
            GameFrame.IsVisible = false;
            LibraryView.IsVisible = true;
            RefreshLibrary();
            StatusText.Text = $"Paused: {_currentDisplayName}";
        }

        // The no-game-running screen, so this is also the unload path - see §4.11.
        private void ShowLibrary()
        {
            ShutDownCurrentSession();
            StopLogging(); // this ROM's log files have no next writer - see §4.12
            _session = null;
            _currentRomPath = null;
            _currentDisplayName = null;
            _rewind.Clear();

            // Or an open console/dashboard keeps inspecting the core we just dropped.
            _debugTarget = null;
            _consoleWindow.Current?.UpdateTarget(null, null);
            _coretopWindow.Current?.UpdateTarget(null);

            GameFrame.IsVisible = false;
            LibraryView.IsVisible = true;
            RefreshLibrary();

            StatusText.Text = "No ROM loaded";
            FpsText.Text = "";
            SyncMenuState();
        }

        // The one console context the library and both cheat windows share - see EmuSen_Multicore.md §10.
        private string SelectedConsole => _appSettings.SelectedCore;

        // One event for both halves of the bar: a facet change rescans, a search change only re-filters what the scan found.
        private void OnLibraryFilterChanged()
        {
            if (LibraryFilter.Facet is not string chosen || chosen == SelectedConsole)
            {
                ShowLibraryEntries();
                return;
            }

            _appSettings.SelectedCore = chosen;
            _appSettings.Save();
            RefreshLibrary();

            // An open cheat window is showing the old console's systems.
            _cheatDatabaseWindow.Current?.SetConsole(chosen);
            _activeCheatsWindow.Current?.SetConsole(chosen, ConsoleCodecs(chosen));
        }

        // A running game wins over the filter - its codecs are the ones that can actually be applied.
        private (ICheatCodeCodec? AutoDetect, ICheatCodeCodec? Explicit) ConsoleCodecs(string console) =>
            _session is { CheatAutoDetectCodec: not null } live
                ? (live.CheatAutoDetectCodec, live.CheatExplicitCodec)
                : CoreFactory.CheatCodecsFor(console);

        // Everything the console filter matched, before the search box narrows it.
        private RomLibraryResult _libraryScan = new(RomLibraryStatus.NoDirectoryConfigured, null, Array.Empty<RomEntry>());

        // The filter bar is wired once, on the first library refresh.
        private bool _libraryFilterReady;

        private void RefreshLibrary()
        {
            if (!_libraryFilterReady)
            {
                _libraryFilterReady = true;
                LibraryList.Key = e => e.FullPath;
                LibraryFilter.SetFacets(EmuSen.Cores.CoreCatalog.FilterChoices,
                    EmuSen.Cores.CoreCatalog.FilterChoices.Contains(SelectedConsole)
                        ? SelectedConsole
                        : EmuSen.Cores.CoreCatalog.AllConsoles);

                // Safe to set before wiring: FilterBar stopped raising Changed for a code-set SearchText in LunaP 0.10.0 - see EmuSen_LunaP.md §14.2.
                LibraryFilter.SearchText = _appSettings.LibrarySearch;

                LibraryFilter.Changed += OnLibraryFilterChanged;
                LibraryFilter.Submitted += LaunchSelectedLibraryEntry;
            }

            // The disk walk happens here, once for every console; the sidebar counts them, the console choice narrows them, and typing only re-filters.
            _allScan = RomLibrary.Scan(_appSettings.RomDirectory);
            _libraryScan = RomLibrary.Narrow(_allScan, SelectedConsole);
            ShowLibraryEntries();
            ReclaimMovedGames();
        }

        private void ShowLibraryEntries()
        {
            string search = LibraryFilter.SearchText;
            _recordSnapshot = _records.All();
            _collections = _records.Collections();
            RomEntry? keptSelection = LibraryList.Selected;
            bool sameSearch = search == _lastLibrarySearch;
            _lastLibrarySearch = search;
            IReadOnlyList<RomEntry> pool = InCollection(_libraryScan.Entries);

            IReadOnlyList<RomEntry> shownEntries = string.IsNullOrWhiteSpace(search)
                ? pool
                : pool.Where(e => FilterBar.Matches(search, e.Title)).ToList();

            // Off the whole scan, not the search subset, so the tag cannot flicker while typing.
            bool mixed = _libraryScan.Entries.Select(e => e.CoreDisplayName).Distinct().Count() > 1;
            MixedConsoles = mixed;
            LibraryList.Label = e => mixed ? $"{e.Title}   —   {e.CoreDisplayName}" : e.Title;
            LibraryList.Refresh(shownEntries);
            _shownEntries = shownEntries;
            LibraryGrid.Refresh(shownEntries);
            ApplyLibraryView();
            FillSidebar();

            if (MediaShown)
            {
                ShowMedia(pool, search);
                ApplyLibraryView();
                LibraryHintText.IsVisible = _shownMedia.Count > 0;
                LibraryHintText.Text = _category == StatesCategory ? "Double-click a state to play from it." : "Double-click a screenshot to open it.";
                return;
            }

            // Otherwise a suspended game is unreachable from here - see EmuSen_Settings_Reference.md §4.18.
            bool suspended = _session is { IsRomLoaded: true };
            LibraryHintText.IsVisible = shownEntries.Count > 0 || suspended;
            LibraryHintText.Text = _gamepad.IsConnected
                ? PadLibraryHint()
                : suspended
                    ? $"Press {ExitToLibraryKeyName()} to return to {_currentDisplayName}."
                    : "Double-click a title, or press Enter, to start it.";

            if (shownEntries.Count > 0)
            {
                string shown = shownEntries.Count == pool.Count
                    ? $"{shownEntries.Count} game{(shownEntries.Count == 1 ? "" : "s")}"
                    : $"{shownEntries.Count} of {pool.Count} games";
                LibraryHeaderText.Text = $"{shown} under {_libraryScan.Directory}";
                // Deliberate: the top match is what FilterBar.Submitted launches - see §4.11a; the same search keeps the choice.
                RomEntry? kept = sameSearch ? keptSelection : null;
                LibraryList.Select(kept is not null && shownEntries.Contains(kept) ? kept : shownEntries[0]);
                LibraryGrid.Select(LibraryList.Selected);
            }
            else
            {
                LibraryHeaderText.Text = pool.Count > 0
                    ? $"No title matches \"{search}\" in {pool.Count} game(s)."
                    : _libraryScan.Entries.Count > 0 ? DescribeEmptyCollection()
                    : RomLibrary.DescribeEmpty(_libraryScan);
            }
        }

        // Named from the binding, so a rebind cannot make the hint lie.
        private string ExitToLibraryKeyName() =>
            _hotkeyBindings.ActionToKey.TryGetValue(HotkeyAction.ExitToLibrary, out Key key) ? key.ToString() : "Escape";

        

        private void OnLibraryItemActivated(object? sender, TappedEventArgs e) => LaunchSelectedLibraryEntry();

        // Wired from the list; the search box's Enter arrives as FilterBar.Submitted instead.
        private void OnLibraryKeyDown(object? sender, KeyEventArgs e)
        {
            if (e.Key != Key.Enter) return;
            e.Handled = true; // otherwise it also reaches the window's own game-input handler
            LaunchSelectedLibraryEntry();
        }

        private async void LaunchSelectedLibraryEntry()
        {
            if (LibraryList.Selected is not RomEntry entry) return;

            await StartGameAsync(entry.FullPath, entry.FileName);
        }

        // Hotkey counterpart to the console's pause/resume pair - see `man pause`.
        private void TogglePause()
        {
            if (_session is not { IsRomLoaded: true }) return;
            if (IsPaused) ResumeEmulation(); else PauseEmulation();
        }

        public void PauseEmulation()
        {
            _pauseSignal.Reset();
            _playClock.Stop();
            if (_session is { IsRomLoaded: true }) StatusText.Text = "Paused";
            SyncMenuState();
        }

        public void ResumeEmulation()
        {
            _pauseSignal.Set();
            if (_session is { IsRomLoaded: true })
            {
                _playClock.Start();
                StatusText.Text = $"Running: {_currentDisplayName}";
            }
            SyncMenuState();
        }

        private void StartEmulationThread()
        {
            // A request queued for the session this one replaced is not this session's to run.
            _coreRequests.Clear();
            _requestSignal.Reset();
            _parkedWakes ??= new[] { _pauseSignal.WaitHandle, _requestSignal.WaitHandle };
            _running = true;
            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();
        }

        // Synchronous by contract: callers replace _session right after - see §4.21.
        private void StopEmulationThread()
        {
            if (_emuThread is null) return;
            _running = false;
            _pauseSignal.Set(); // wake the thread if it's currently paused, so it can observe _running=false and exit rather than deadlocking Join() below
            _emuThread.Join();
            _emuThread = null;
        }

        // Runs off the UI thread, and the input race is accepted - see EmuSen_Settings_Reference.md §4.21.
        private void EmulationLoop()
        {
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            // Counts completed RunFrame calls, not presented frames - see §4.21.
            TimeSpan fpsWindowStart = clock.Elapsed;
            int framesInWindow = 0, offeredInWindow = 0;

            // The serial of the picture last offered; a core that says its picture is unchanged is not offered it again - see EmuSen_Multicore.md §14.
            long? offeredSerial = null;

            // RunFrame alone against the rest of this loop's per-frame work - see §4.21.
            TimeSpan runFrameTimeInWindow = TimeSpan.Zero;
            var frameStopwatch = new Stopwatch();

            // Whatever phases the core publishes, averaged like runFrameTimeInWindow - see EmuSen_Multicore.md §5.
            var profiler = _session?.Core as IFrameProfiler;
            string[] phaseNames = profiler?.LastFramePhases.Select(p => p.Name).ToArray() ?? Array.Empty<string>();
            var phaseMsInWindow = new double[phaseNames.Length];

            while (_running)
            {
                // Checked before waiting so the unpaused case stays a plain read - see §4.21.
                if (!_pauseSignal.IsSet)
                {
                    // Woken by a save or a load as well as by resume, so the hotkeys work while paused - see §4.21a.
                    while (!_pauseSignal.IsSet && _running)
                    {
                        WaitHandle.WaitAny(_parkedWakes!);
                        if (_session is { } parked) RunCoreRequests(parked);
                    }

                    if (!_running) break;

                    // Or the whole paused duration lands on one artificially slow frame - see §4.21.
                    nextTick = clock.Elapsed;
                }

                EmulatorSession? session = _session;
                if (session is null) break;

                // Held turbo wins over the menu's base speed - see EmuSen_Settings_Reference.md §4.13.
                _speed.SpeedPercent = _turboHeld ? _speed.TurboPercent : _baseSpeedPercent;
                nextTick += _speed.FrameInterval(session.FrameRateHz);

                // Takes over the frame entirely - see EmuSen_Rewind_And_FastForward.md §4.
                if (_rewindHeld && session.Core is not null)
                {
                    session.SkipRendering = false;
                    if (!_rewind.Rewind(session.Core)) nextTick = clock.Elapsed;
                    // No RunFrame() ran, so these would otherwise be stale - see §3.
                    _debugTarget?.RefreshProviders();
                    session.DequeueAudioSamples(int.MaxValue);
                    _audioPlayer.RateControl.Reset(); // skipped content - see EmuSen_Audio_Sync.md §3.2
                    SubmitFrame(session.GetFrameBufferRgba(), session.ScreenWidth, session.ScreenHeight, session.RowRepeat);
                    offeredSerial = null;
                    RunCoreRequests(session);
                    SleepUntil(nextTick, clock);
                    continue;
                }

                session.SkipRendering = !_speed.ShouldRender(session.TotalFrames);

                try
                {
                    frameStopwatch.Restart();
                    session.RunFrame();
                    runFrameTimeInWindow += frameStopwatch.Elapsed;

                    // Publishes this frame's snapshots; null-conditional because a swap can be in flight.
                    _debugTarget?.RefreshProviders();

                    // Runs what the console queued, on the thread that owns the core - see `man pause`.
                    _consoleWindow.Current?.DrainPendingFromEmulationThread();

                    // Same rule, same thread: the Apply Cheats button only sets the flag - see §4.15.
                    if (_applyCheatsPending)
                    {
                        _applyCheatsPending = false;
                        _debugTarget?.ApplyCheats();
                    }

                    // Between frames, and before the rewind snapshot, so a loaded state seeds the chain - see §4.21a.
                    RunCoreRequests(session);

                    // Right after RunFrame, which is what produces new samples to drain.
                    // Off for the N64 for now: its snapshot with several rasteriser workers froze a game - see EmuSen_Settings_Reference.md §4.21b.
                    if (session.Core is not null and not EmuSen.Cores.Nintendo.Mars.MarsCore) _rewind.OnFrameCompleted(session.Core);

                    // Drained every frame either way, so a muted stretch cannot back the buffer up - see EmuSen_Audio_Sync.md §4.
                    short[] samples = session.DequeueAudioSamples(int.MaxValue);
                    if (_speed.ShouldPlayAudio) _audioPlayer.Submit(samples, session.AudioSampleRate);
                    else _audioPlayer.RateControl.Reset(); // skipped content - see EmuSen_Audio_Sync.md §3.2

                    if (profiler is not null)
                    {
                        var current = profiler.LastFramePhases;
                        for (int p = 0; p < phaseMsInWindow.Length && p < current.Count; p++)
                        {
                            phaseMsInWindow[p] += current[p].Milliseconds;
                        }
                    }

                    // Nothing new was drawn on a skipped frame.
                    long? serial = session.FrameSerial;
                    if (!session.SkipRendering && (serial is null || serial != offeredSerial))
                    {
                        byte[] frame = session.GetFrameBufferRgba();
                        SubmitFrame(frame, session.ScreenWidth, session.ScreenHeight, session.RowRepeat);
                        offeredSerial = serial;
                        offeredInWindow++;
                    }

                    framesInWindow++;
                    TimeSpan windowElapsed = clock.Elapsed - fpsWindowStart;
                    if (windowElapsed >= TimeSpan.FromSeconds(1))
                    {
                        double fps = framesInWindow / windowElapsed.TotalSeconds;
                        double runFrameMs = runFrameTimeInWindow.TotalMilliseconds / framesInWindow;
                        double totalMs = windowElapsed.TotalMilliseconds / framesInWindow;
                        // A "parent/child" phase sits inside its parent, so it goes in the second group.
                        string top = string.Join(" / ", phaseNames.Index()
                            .Where(n => !n.Item.Contains('/'))
                            .Select(n => $"{n.Item} {phaseMsInWindow[n.Index] / framesInWindow:F2}ms"));
                        string nested = string.Join(" / ", phaseNames.Index()
                            .Where(n => n.Item.Contains('/'))
                            .Select(n => $"{n.Item} {phaseMsInWindow[n.Index] / framesInWindow:F2}ms"));

                        string breakdown = top.Length == 0 ? "" : $" [{top}]";
                        if (nested.Length > 0) breakdown += $" ({nested})";

                        // Frames handed over against frames the render thread drew, and what drawing them cost it - see EmuSen_Serenity.md §2.5.
                        var shown = GameFrame.TakeStatistics();
                        double seconds = windowElapsed.TotalSeconds;
                        string presentation = shown.Frames == 0
                            ? $" | offered {offeredInWindow / seconds:F1}, shown 0"
                            : $" | offered {offeredInWindow / seconds:F1}, shown {shown.Frames / seconds:F1} fps ({shown.Copies / seconds:F1} copied), copy {shown.CopyMilliseconds / Math.Max(shown.Copies, 1):F2} draw {shown.DrawMilliseconds / shown.Frames:F2}ms, {(shown.Gpu ? "GPU" : "software")} {shown.Width}x{shown.Height}";
                        string line = $"{fps:F1} fps (run {runFrameMs:F2}ms / total {totalMs:F2}ms){breakdown}{presentation}";
                        Console.WriteLine($"[fps] {line}");

                        Dispatcher.UIThread.Post(() => FpsText.Text = line);
                        framesInWindow = 0;
                        offeredInWindow = 0;
                        Array.Clear(phaseMsInWindow);
                        runFrameTimeInWindow = TimeSpan.Zero;
                        fpsWindowStart = clock.Elapsed;
                    }
                }
                catch (Exception ex)
                {
                    _running = false;
                    string message = ex.Message;
                    string settings = string.Join(", ", EmuSen.Cores.CoreCatalog.SettingsFor(_activeConsole).Select(k => $"{k.Key}={_graphics.Value(_activeConsole, k.Key) ?? k.Default}"));
                    string? report = CrashLog.Write("emulation halt", ex, $"{_activeConsole}, frame {session.Core?.TotalFrames}: {settings}");
                    // Halt and print rather than recover; StatusText needs the UI thread.
                    Dispatcher.UIThread.Post(() => StatusText.Text = $"[CPU HALT] {message}" + (report is null ? "" : $" - written to {report}"));
                    break;
                }

                TimeSpan remaining = nextTick - clock.Elapsed;
                if (remaining > TimeSpan.Zero)
                {
                    SleepUntil(nextTick, clock);
                }
                else
                {
                    // Owed rather than forgotten, up to a cap, or a game whose drawing frames overrun runs slow - see §4.28.
                    nextTick = FramePacer.Settle(nextTick, clock.Elapsed, _speed.FrameInterval(session.FrameRateHz));
                }
            }
        }

        // Spins the whole interval; the hybrid was measured and dropped - see §4.21.
        private static void SleepUntil(TimeSpan target, Stopwatch clock)
        {
            while (clock.Elapsed < target)
            {
                Thread.SpinWait(100);
            }
        }

        // Called from the emulation thread; newest wins - see EmuSen_Serenity.md §4.
        private void SubmitFrame(byte[] pixels, int width, int height, int rowRepeat)
        {
            _frames.Offer(new FrameData { Pixels = pixels, Width = width, Height = height, RowRepeat = rowRepeat });
        }

        // Presents whatever is newest when it runs; dropping stale frames is intended.
        private void PresentPendingFrame(FrameData frame)
        {
            GameFrame.UpdateFrame(frame.Pixels, frame.Width, frame.Height, frame.RowRepeat);
        }

        private void OnExitClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }

        // Redirected per ROM, and unconditionally - see EmuSen_Settings_Reference.md §4.22.
        private void StartLogging(string coreName)
        {
            StopLogging(); // close the previous session's files first - see CategorizedLogWriter.Dispose's own comment

            string logRoot = string.IsNullOrWhiteSpace(_appSettings.LogDirectory)
                ? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "EmuSen", "Logs")
                : _appSettings.LogDirectory;

            try
            {
                string logDir = Path.Combine(logRoot, coreName, $"gui_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                _activeLogWriter = new CategorizedLogWriter(_originalConsoleOut, logDir);
                Console.SetOut(_activeLogWriter);
            }
            catch (Exception ex)
            {
                // Best-effort: an unwritable directory must not block the load - see §4.22.
                StatusText.Text = $"Logging disabled: {ex.Message}";
            }
        }

        private void StopLogging()
        {
            if (_activeLogWriter is null) return;

            // Before Console.SetOut/Dispose - see EmuSen_Settings_Reference.md §4.22.
            _session?.FlushVerboseLogs();

            Console.SetOut(_originalConsoleOut);
            _activeLogWriter.Dispose();
            _activeLogWriter = null;
        }
    }
}
