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

        // Gamepad polling stays on the UI thread, on its own timer, separate
        // from emulation - see EmuSen_Settings_Reference.md §4.10.
        private DispatcherTimer? _timer;

        // Runs RunFrame() + frame-buffer readout off the UI thread entirely -
        // previously both lived inside the DispatcherTimer tick above, which
        // meant emulation work (and the WriteableBitmap copy after it)
        // blocked Avalonia's UI/input/paint pump every single frame. This is
        // the actual fix for Testing Studio running much slower than the
        // console build, which always ran its equivalent loop off of any UI
        // event pump to begin with (it doesn't have one). See
        // EmulationLoop()/StartEmulationThread()/StopEmulationThread() below.
        private Thread? _emuThread;
        private volatile bool _running;

        // Pause/resume for the emulation thread, added so a shell command
        // typed into _consoleWindow can safely read/write Cpu/Bus/Renderer
        // state without racing RunFrame() on _emuThread - see
        // EmulationControlCommands.cs's own comment for why this exists.
        // Signaled (Set) = running, unsignaled (Reset) = paused; starts
        // signaled so a freshly-started thread doesn't block before
        // anyone's had a chance to pause it. EmulationLoop blocks on this
        // at the top of every iteration rather than busy-polling a bool,
        // and StopEmulationThread always Sets it before Join()ing so a
        // paused thread can still wake up, notice _running is false, and
        // exit - otherwise stopping while paused would deadlock forever.
        private readonly ManualResetEventSlim _pauseSignal = new(initialState: true);

        public bool IsPaused => !_pauseSignal.IsSet;

        // Coalescing hand-off from _emuThread to the UI thread: the
        // emulation thread can produce frames faster than Avalonia can
        // present them, and posting one Dispatcher action per emulated frame
        // would just queue them up and make the UI thread fall further and
        // further behind. Instead, only ever have at most one Present
        // dispatched at a time - _emuThread always overwrites _pendingFrame
        // with the newest frame and only schedules a new Present if one
        // isn't already in flight, so a slow UI thread simply drops
        // intermediate frames rather than backing up a queue of them.
        private sealed class FrameData
        {
            public required byte[] Pixels;
            public required int Width;
            public required int Height;
        }
        // "Newest wins, at most one UI-thread callback outstanding" is LunaP's Latest<T> now. This
        // class, Hotaru's GameWindow and Serenity's FramePresenter each wrote it out identically,
        // which is what argued it into the toolkit - and all three carried the same defect: the
        // scheduled flag was cleared AFTER the hand-off, so a frame submitted while the UI thread
        // was inside UpdateFrame could neither schedule a callback nor be picked up by the running
        // one. It sat there until the next frame displaced it.
        //
        // Invisible at 60 fps, because the next frame arrives 16 ms later carrying the fix. Visible
        // the moment the stream stops - pause, and the frame at risk is the last one drawn, which
        // is the one somebody is about to sit and look at. LunaP.md §22.1.
        private readonly Latest<FrameData> _frames;

        // Rebuilt on every LoadRom() call (see LoadRom below) so a shell
        // command run in _consoleWindow always sees whatever core is
        // actually running now, never a Cpu/Bus/Renderer left over from a
        // ROM that's since been swapped out. Null before the first ROM
        // loads - the console window (and every shell command's own
        // RequireTarget guard) already treats that as a normal condition.
        private IDebugTarget? _debugTarget;

        // At most one console window at a time - Show()n non-modally (same
        // pattern DebugSettingsWindow/InputSettingsWindow already use), and
        // reused (brought to front) rather than duplicated if the menu item
        // is clicked again while one's still open. Cleared on Closed so a
        // later LoadRom() doesn't try to push a target update into a
        // disposed window.
        private readonly WindowSlot<DianaOSConsoleWindow> _consoleWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern as _consoleWindow
        // above - opened from inside the console window itself (typing
        // `coretop`, via CoretopWindowCommand), not from a menu item.
        private readonly WindowSlot<CoretopWindow> _coretopWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern, but with no
        // target to update - see `man vstop`.
        private readonly WindowSlot<VstopWindow> _vstopWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern, refreshed rather
        // than re-targeted - see EmuSen_Settings_Reference.md §4.14.
        private readonly WindowSlot<ActiveCheatsWindow> _activeCheatsWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern, retained so a console switch can retarget it.
        private readonly WindowSlot<CheatDatabaseWindow> _cheatDatabaseWindow = new();

        // Owned here, not by _debugTarget, so a cheat list outlives the core
        // a Reset rebuilds - see §4.14.
        private readonly CheatRegistry _cheats = new();

        // Which ROM the loaded cheats were meant for; null while they belong
        // to no game yet. Not _currentRomPath, which a close resets - see §4.14.
        private string? _cheatsRomPath;

        // Set by the Active Cheats window's Apply button, cleared by the
        // emulation thread - cheats must be poked from the thread that owns
        // the core, same rule DrainPendingFromEmulationThread follows. See §4.15.
        private volatile bool _applyCheatsPending;

        private string? _currentRomPath;
        private string? _currentDisplayName; // for restoring StatusText's "Running: ..." text exactly after a pause, without reformatting from _currentRomPath
        private readonly ControllerKeyBindings _keyBindings =
            ControllerKeyBindings.Load(EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));
        private readonly GamepadBindings _gamepadBindings =
            GamepadBindings.Load(EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console));

        // Which console's bindings are live. Follows the loaded ROM; the first
        // console in catalog order stands in before one is loaded - see EmuSen_Input.md §5.1.
        private string _activeConsole = EmuSen.Cores.CoreCatalog.ConsolesInReleaseOrder[0].Console;
        private readonly HotkeyBindingMap _hotkeyBindings = HotkeyBindingMap.Load();
        private readonly AppSettings _appSettings = AppSettings.Load();
        private readonly GamepadManager _gamepad;

        // Constructed once at startup and reused across every ROM load,
        // same lifetime as _gamepad above (and for the same reason -
        // Pump() below just no-ops via EmulatorSession's own null-session
        // fallback when nothing's loaded, so there's no need to
        // open/close the output device per load).
        private readonly AudioPlayer _audioPlayer;

        // Captured once at startup, before anything ever redirects
        // Console.Out - restoring this (rather than whatever
        // _activeLogWriter happened to be at the time) is what lets logging
        // be turned off again (or repointed) without leaving Console.Out
        // pointed at a disposed writer.
        private readonly TextWriter _originalConsoleOut = Console.Out;
        private CategorizedLogWriter? _activeLogWriter;

        // Keyboard and gamepad are tracked separately and combined with OR
        // logic - matches EmuSen.Hotaru's own GameWindow convention
        // ("either device works at any time, no need to pick
        // one"). Without this, a gamepad poll finding a button NOT pressed
        // would incorrectly release a button still being held on the
        // keyboard, and vice versa.
        private readonly bool[] _keyboardHeld = new bool[Enum.GetValues<PadButton>().Length];
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
            BuildMenus();
            // The tick follows the window, so a window-manager key cannot leave it lying - see §4.19.
            FullScreenChanged += on => _fullscreen.IsChecked = on;
            _gamepad = new GamepadManager(_gamepadBindings.For(_activeConsole));
            // Endymion is a leaf and reads no globals, so the settings come from here - see EmuSen_Audio_Sync.md §7.1.
            _audioPlayer = new AudioPlayer(
                AudioSettings.SampleRate, AudioSettings.OutputTargetLatencyMs, AudioSettings.RateControlMaxDeviation);
            // Attached to GameFrame rather than the window: "hidden over the video and
            // visible over the toolbar" is not something a window-level flag can say.
            _idleCursor = new IdleCursor(GameFrame);
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
                StopEmulationThread();
                _session?.SaveSram();
                _gamepad.Dispose();
                _audioPlayer.Dispose();
                StopLogging();
            };

            _gamepad.AnalogStickAsDpad = _appSettings.AnalogStickAsDpad;
            _gamepad.StickDeadzone = _appSettings.StickDeadzone;

            RefreshLibrary();
            BuildSaveSlotItems();

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
            if (!LibraryView.IsVisible && _keyBindings.For(_activeConsole).TryGetButton(key, out var button))
            {
                _keyboardHeld[(int)button] = pressed;
                ApplyButtonState(button);
                return;
            }

            if (!_hotkeyBindings.TryGetAction(key, out HotkeyAction action)) return;

            // Held vs one-shot - see EmuSen_Rewind_And_FastForward.md §4, EmuSen_Settings_Reference.md §4.3.
            if (HotkeyBindingMap.IsHeld(action))
            {
                if (action == HotkeyAction.FastForward) _turboHeld = pressed;
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
            bool held = _keyboardHeld[(int)button] || _gamepadHeld[(int)button];
            _session.SetButton(0, button, held);
            if (_appSettings.MirrorPlayer1ToPlayer2) _session.SetButton(1, button, held);
        }

        private void PollGamepad()
        {
            _gamepad.Poll();
            foreach (PadButton button in Enum.GetValues<PadButton>())
            {
                bool held = _gamepad.IsPressed(button);
                if (held != _gamepadHeld[(int)button])
                {
                    _gamepadHeld[(int)button] = held;
                    ApplyButtonState(button);
                }
            }
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

            await PromptForMissingFirmwareAsync(file.Path);
            LoadRom(file.Path, file.Name);
        }

        // Offers the OS picker for anything this ROM needs that the firmware
        // library doesn't have yet, and installs whatever comes back.
        // Declining is a perfectly good answer - the core then loads with
        // that chip absent, exactly as it does headless. Runs BEFORE LoadRom
        // because afterwards is too late. See EmuSen_Firmware.md §3.
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

        // The same sequence Open ROM... runs, so a dropped ROM cannot skip the
        // firmware prompt - see EmuSen_Firmware.md §3.
        private async Task OpenDroppedRomAsync(string path)
        {
            await PromptForMissingFirmwareAsync(path);
            LoadRom(path, System.IO.Path.GetFileName(path));
        }

        // Alternative to the OS file picker above - lists .smc/.sfc files
        // straight from AppSettings.RomDirectory, for anyone reloading
        // different ROMs from the same test folder repeatedly. See
        // RomBrowserWindow's own comment.
        private async Task BrowseRomsAsync()
        {
            string? selected = await new RomBrowserWindow(_appSettings.RomDirectory).ShowDialog<string?>(this);
            if (selected is null) return;

            await PromptForMissingFirmwareAsync(selected);
            LoadRom(selected, Path.GetFileName(selected));
        }

        private void ShowControllerBindings()
        {
            var window = new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad, _appSettings, _hotkeyBindings,
                _session is null ? null : _activeConsole);
            // A rebind has to reach the menu, or it advertises the old key - see §4.19.
            window.Closed += (_, _) => SyncMenuState();
            window.Show(this);
        }

        private void ShowPreferences()
        {
            // Non-modal, so the ROM directory can change while the library is
            // on screen behind it - re-scan on close rather than leaving a
            // stale list the user has to know to refresh.
            var window = new PreferencesWindow(_appSettings);
            window.Closed += (_, _) => { if (LibraryView.IsVisible) RefreshLibrary(); };
            window.Show(this);
        }

        private void ShowDebugLogging()
        {
            new DebugSettingsWindow().Show(this);
        }

        // Never needs a ROM - it manages the cheat folder and the cheat list,
        // neither of which is a session. See §4.14.
        private void ShowCheatDatabase()
        {
            _cheatDatabaseWindow.Show(this, () => new CheatDatabaseWindow(_appSettings, () => _cheats,
                ConsoleCodecs(SelectedConsole).AutoDetect,
                () => _activeCheatsWindow.Current?.Refresh(),
                ShowActiveCheats,
                () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems,
                SelectedConsole));
        }

        

        // Opens the one Active Cheats window, or brings it forward already
        // refreshed - the menu item and the database window's own button are
        // the same door. See §4.14.
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

        // Hands the poke to the emulation thread rather than doing it here -
        // see _applyCheatsPending. False means there was no core to hand it to.
        private bool RequestCheatApply()
        {
            if (_debugTarget is null) return false;

            // Paused parks the emulation thread in _pauseSignal.Wait() rather
            // than in the core, so nothing is racing and this can happen now -
            // which is the whole point of the button while paused. See §4.15.
            if (IsPaused) _debugTarget.ApplyCheats();
            else _applyCheatsPending = true;

            return true;
        }

        // The file name a ROM's cheat list is saved under and looked for at
        // load. Sanitized rather than validated, unlike the name `cheat save`
        // takes, because this one is derived rather than typed - see §4.15.
        private static string? CheatListName(string? romPath)
        {
            if (string.IsNullOrEmpty(romPath)) return null;

            var name = new string(System.IO.Path.GetFileNameWithoutExtension(romPath)
                .Select(c => System.IO.Path.GetInvalidFileNameChars().Contains(c) ? '_' : c).ToArray());
            return CheatFile.IsValidName(name) ? name : null;
        }

        // Whatever was saved for this ROM, brought back so a player doesn't
        // reload it from the database every session - see §4.15. Returns how
        // many arrived, for the caller to mention in its own status line.
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

        // Built fresh per DianaOSConsoleWindow construction (not cached) -
        // each PauseCommand/ResumeCommand instance only needs to close
        // over `this`, so there's no real cost to re-creating them, and
        // it avoids the two commands' delegates ever accidentally
        // outliving a MainWindow instance.
        private IEnumerable<IDianaOSCommand> MakeEmulationControlCommands() => new IDianaOSCommand[]
        {
            new PauseCommand(PauseEmulation, () => IsPaused),
            new ResumeCommand(ResumeEmulation, () => IsPaused),
            new CoretopWindowCommand(OpenCoretopWindow),
            new VstopWindowCommand(OpenVstopWindow),
            new FeedCommand(() => Activate()),
            new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.StateCommand(SaveStateFromConsole, LoadStateFromConsole, () => CurrentStatePath ?? ""),
        };

        // StateCommand's save/load delegates - can't just be
        // `_session.SaveState`/`LoadState` method groups the way
        // MakeEmulationControlCommands() reuses PauseEmulation/
        // ResumeEmulation directly, since `_session` can still be null the
        // first time this window's console is opened (before any ROM has
        // ever loaded) - a method-group conversion would dereference it
        // immediately, at command-construction time, not when the command
        // actually runs. These defer that check to call time instead, and
        // throw the same clean message CoretopWindowCommand's own
        // RequireTarget-backed "No ROM loaded" case already establishes,
        // rather than letting StateCommand's try/catch surface a bare
        // NullReferenceException's own unhelpful message.
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

        // Defaults to the same home/Saves/Save States/ tree the console
        // build and DianaOS shell's `state save`/`state load` write to -
        // overridable in Preferences (AppSettings.StateDirectory) for
        // anyone who wants states somewhere else.
        private string? CurrentStatePath => StatePathForSlot(_stateSlot);

        // Slot 1 is the plain <rom>.state every other frontend already
        // writes - see EmuSen_Settings_Reference.md §4.13.
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

            try
            {
                System.IO.Directory.CreateDirectory(System.IO.Path.GetDirectoryName(path)!);
                _session.SaveState(path);
                StatusText.Text = $"State saved to slot {_stateSlot}: {System.IO.Path.GetFileName(path)}";
                SyncMenuState(); // that slot has a timestamp now
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Save State failed: {ex.Message}";
            }
        }

        private void LoadState()
        {
            if (_session is not { IsRomLoaded: true } || CurrentStatePath is not string path)
            {
                StatusText.Text = "Load State: no ROM loaded";
                return;
            }

            if (!System.IO.File.Exists(path))
            {
                StatusText.Text = $"Load State: slot {_stateSlot} is empty";
                return;
            }

            try
            {
                _session.LoadState(path);
                _rewind.Clear(); // a discontinuous jump - see §1.4
                _audioPlayer.RateControl.Reset();
                StatusText.Text = $"State loaded from slot {_stateSlot}: {System.IO.Path.GetFileName(path)}";
            }
            catch (Exception ex)
            {
                StatusText.Text = $"Load State failed: {ex.Message}";
            }
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
                    LunaAction.Separator(),
                    new LunaAction("E_xit", () => Close())),
                new LunaMenu("_Emulation",
                    _pause, _reset, _closeGame,
                    LunaAction.Separator(),
                    _speedMenu,
                    LunaAction.Separator(),
                    _saveState, _loadState, _slotMenu),
                new LunaMenu("_View", _fullscreen),
                new LunaMenu("_Settings",
                    new LunaAction("_Controller Bindings...", ShowControllerBindings),
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

        // The menu advertises the keys HotkeyBindingMap holds, and nothing else binds
        // them - MenuBar.SetMenus draws a gesture without binding it, so a rebind moves
        // the label and cannot leave a second binding behind. See §4.19.
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

        // Another game's addresses are meaningless here, so they go rather
        // than quietly poking this one's RAM. A Reset keeps them - same ROM.
        private int DropCheatsFromAnotherGame(string path)
        {
            bool sameGame = string.Equals(_cheatsRomPath, path, StringComparison.Ordinal);
            if (_cheatsRomPath is not null && !sameGame)
            {
                _cheats.Clear();
                _activeCheatsWindow.Current?.Refresh();
            }

            _cheatsRomPath = path;

            // Only when this game's list isn't already in hand: a Reset or a
            // close-and-reopen must not overwrite edits made since - see §4.15.
            return !sameGame && _cheats.GetCheats().Count == 0 ? LoadSavedCheatsFor(path) : 0;
        }

        // A power cycle, not a soft reset - see EmuSen_Settings_Reference.md §4.12.
        private void ResetEmulation()
        {
            if (_session is not { IsRomLoaded: true }) return;
            if (_currentRomPath is not string path || _currentDisplayName is not string displayName) return;

            ResumeEmulation(); // a reset always comes back running, however it was paused
            LoadRom(path, displayName);
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Reset: {displayName}";
        }

        // Everything that has to happen to whatever is currently running
        // before _session can be replaced or dropped.
        private void ShutDownCurrentSession()
        {
            _timer?.Stop();
            StopEmulationThread(); // must fully stop before _session changes - see that method's own comment
            _session?.SaveSram(); // flush whatever was previously running before switching
            // Must happen on the OLD session: StartLogging()'s call to
            // StopLogging() runs against whatever _session currently is,
            // which would already be the new one.
            _session?.FlushVerboseLogs();
        }

        private void LoadRom(string path, string displayName)
        {
            ShutDownCurrentSession();
            int restoredCheats = DropCheatsFromAnotherGame(path);

            try
            {
                _session = new EmulatorSession { Cheats = _cheats };
                // Off the path, not the session: no core exists yet to ask - see EmuSen_Multicore.md §12.
                StartLogging(EmuSen.Cores.CoreCatalog.ConsoleForRom(path) ?? "Unknown");
                _session.LoadRom(path);

                // The pad this ROM's console reads, not whatever the last one used.
                _activeConsole = _session.CoreName;
                _gamepad.Bindings = _gamepadBindings.For(_activeConsole);

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

                // Gamepad polling only - see _timer's own field comment for
                // why this stays separate from emulation itself.
                _timer = new DispatcherTimer { Interval = FrameInterval };
                _timer.Tick += (_, _) => PollGamepad();
                _timer.Start();

                StartEmulationThread();
                SyncMenuState();
            }
            catch (Exception ex)
            {
                _session = null;
                // Back to the list rather than a black viewport with only a
                // status line to explain it.
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

        // The library is the no-game-running screen, so this is also the
        // unload path - see EmuSen_Settings_Reference.md §4.11.
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

                LibraryFilter.Changed += OnLibraryFilterChanged;
                LibraryFilter.Submitted += LaunchSelectedLibraryEntry;
            }

            // The disk walk happens here; typing in the search box only re-filters what it found.
            _libraryScan = RomLibrary.Scan(_appSettings.RomDirectory, SelectedConsole);
            ShowLibraryEntries();
        }

        private void ShowLibraryEntries()
        {
            string search = LibraryFilter.SearchText;

            IReadOnlyList<RomEntry> shownEntries = string.IsNullOrWhiteSpace(search)
                ? _libraryScan.Entries
                : _libraryScan.Entries.Where(e => FilterBar.Matches(search, e.Title)).ToList();

            // Off the whole scan, not the search subset, so the tag cannot flicker while typing.
            bool mixed = _libraryScan.Entries.Select(e => e.CoreDisplayName).Distinct().Count() > 1;
            LibraryList.Label = e => mixed ? $"{e.Title}   —   {e.CoreDisplayName}" : e.Title;
            LibraryList.Refresh(shownEntries);

            LibraryList.IsVisible = shownEntries.Count > 0;

            // Otherwise a suspended game is unreachable from here - see EmuSen_Settings_Reference.md §4.18.
            bool suspended = _session is { IsRomLoaded: true };
            LibraryHintText.IsVisible = shownEntries.Count > 0 || suspended;
            LibraryHintText.Text = suspended
                ? $"Press {ExitToLibraryKeyName()} to return to {_currentDisplayName}."
                : "Double-click a title, or press Enter, to start it.";

            if (shownEntries.Count > 0)
            {
                string shown = shownEntries.Count == _libraryScan.Entries.Count
                    ? $"{shownEntries.Count} game{(shownEntries.Count == 1 ? "" : "s")}"
                    : $"{shownEntries.Count} of {_libraryScan.Entries.Count} games";
                LibraryHeaderText.Text = $"{shown} under {_libraryScan.Directory}";
                // Deliberate: the top match is what FilterBar.Submitted launches - see §4.11a.
                LibraryList.Select(shownEntries[0]);
            }
            else
            {
                LibraryHeaderText.Text = _libraryScan.Entries.Count > 0
                    ? $"No title matches \"{search}\" in {_libraryScan.Entries.Count} game(s)."
                    : RomLibrary.DescribeEmpty(_libraryScan);
            }
        }

        // Named from the binding, so a rebind cannot make the hint lie.
        private string ExitToLibraryKeyName() =>
            _hotkeyBindings.ActionToKey.TryGetValue(HotkeyAction.ExitToLibrary, out Key key) ? key.ToString() : "Escape";

        

        private void OnLibraryItemActivated(object? sender, TappedEventArgs e) => LaunchSelectedLibraryEntry();

        // Shared by the list and the search box, so Enter starts a title from either.
        // Still wired from the list itself; the search box's own Enter arrives as FilterBar.Submitted instead.
        private void OnLibraryKeyDown(object? sender, KeyEventArgs e)
        {
            if (e.Key != Key.Enter) return;
            e.Handled = true; // otherwise it also reaches the window's own game-input handler
            LaunchSelectedLibraryEntry();
        }

        private async void LaunchSelectedLibraryEntry()
        {
            if (LibraryList.Selected is not RomEntry entry) return;

            await PromptForMissingFirmwareAsync(entry.FullPath);
            LoadRom(entry.FullPath, entry.FileName);
        }

        // Called from _consoleWindow's PauseCommand/ResumeCommand (both run
        // on the UI thread, same as this method) - ManualResetEventSlim's
        // Set/Reset are thread-safe regardless, so there's nothing else to
        // guard here. StatusText is only ever touched from the UI thread
        // in either case, so no Dispatcher.UIThread.Post is needed the way
        // EmulationLoop needs one for its own cross-thread updates.
        // Hotkey counterpart to the console's `pause`/`resume` pair.
        private void TogglePause()
        {
            if (_session is not { IsRomLoaded: true }) return;
            if (IsPaused) ResumeEmulation(); else PauseEmulation();
        }

        public void PauseEmulation()
        {
            _pauseSignal.Reset();
            if (_session is { IsRomLoaded: true }) StatusText.Text = "Paused";
            SyncMenuState();
        }

        public void ResumeEmulation()
        {
            _pauseSignal.Set();
            if (_session is { IsRomLoaded: true }) StatusText.Text = $"Running: {_currentDisplayName}";
            SyncMenuState();
        }

        private void StartEmulationThread()
        {
            _running = true;
            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();
        }

        // Signals the loop to stop and waits for it to actually exit before
        // returning - callers (LoadRom, window Closing) need this to be
        // synchronous since they go on to replace/dispose _session right
        // after. Safe to call from the UI thread: EmulationLoop checks
        // _running once per paced ~16ms frame interval at most, so Join()
        // here never blocks for long, and it never blocks indefinitely since
        // the loop has no other blocking wait.
        private void StopEmulationThread()
        {
            if (_emuThread is null) return;
            _running = false;
            _pauseSignal.Set(); // wake the thread if it's currently paused, so it can observe _running=false and exit rather than deadlocking Join() below
            _emuThread.Join();
            _emuThread = null;
        }

        // Runs entirely off the UI thread: RunFrame() (the actual CPU/PPU/
        // APU work) and the frame-buffer readout no longer compete with
        // Avalonia's input/paint pump the way they did inside the old
        // DispatcherTimer tick. Only the final present (PresentPendingFrame,
        // dispatched below) needs the UI thread - GameFrame (GameFrameControl)
        // is a regular Avalonia Control, and Control/Visual are UI-thread-
        // affine the same way WriteableBitmap used to be.
        //
        // Input note: keyboard/gamepad button state is applied straight to
        // _session.Bus.Input from the UI thread (SetButtonFromKey,
        // PollGamepad's ApplyButtonState calls) while this thread
        // concurrently calls RunFrame(), which reads that same state
        // (Input.cs's LatchAutoJoypad/live-state fields). Deliberately left
        // unsynchronized: each field involved is a single ushort/bool
        // read-modify-write with exactly one writer (the UI thread), so the
        // worst case is a button's state being observed one frame later
        // than it otherwise would - not a torn read or a crash - which is
        // an acceptable, already-inherent latency for a frame-latched input
        // model like this one, not a new correctness risk introduced by
        // moving emulation to its own thread.
        private void EmulationLoop()
        {
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            // Measured emulated FPS (TotalFrames delta / real elapsed time),
            // updated once a second - a diagnostic for telling "RunFrame()
            // itself is running behind real time" apart from "presentation
            // is just choppy but emulation is on schedule". Deliberately
            // counts completed RunFrame() calls, not presented frames -
            // SubmitFrame()/PresentPendingFrame's coalescing can legitimately
            // drop presented frames without that meaning emulation itself
            // is slow.
            TimeSpan fpsWindowStart = clock.Elapsed;
            int framesInWindow = 0;

            // Split timing: RunFrame() alone vs. the rest of this loop's own
            // per-frame work (GetFrameBufferRgba's copy + SubmitFrame's
            // hand-off). Boot/title screens hitting 60fps while gameplay
            // drops below it could mean either RunFrame() itself getting
            // more expensive under a heavier scene (shared core code -
            // would cost the console build the same) or this loop's own
            // wrapper overhead scaling up with scene complexity (frontend-
            // specific, e.g. more per-frame allocation/GC pressure) -
            // reporting both separately is how to tell which.
            TimeSpan runFrameTimeInWindow = TimeSpan.Zero;
            var frameStopwatch = new Stopwatch();

            // Whatever phases the core publishes, averaged like runFrameTimeInWindow - see EmuSen_Multicore.md §5.
            var profiler = _session?.Core as IFrameProfiler;
            string[] phaseNames = profiler?.LastFramePhases.Select(p => p.Name).ToArray() ?? Array.Empty<string>();
            var phaseMsInWindow = new double[phaseNames.Length];

            while (_running)
            {
                // Blocks here, not inside the try below, while paused -
                // see _pauseSignal's own field comment. StopEmulationThread
                // always Sets this before Join()ing, so this can never
                // block forever even if a console command pauses and the
                // window is then closed without resuming first. Checked
                // before waiting (rather than always calling Wait(), which
                // would also be correct but always costs a syscall even
                // when never paused) so the by-far-more-common unpaused
                // case stays a plain volatile-ish read.
                if (!_pauseSignal.IsSet)
                {
                    _pauseSignal.Wait();
                    if (!_running) break;

                    // Otherwise nextTick would still be wherever it was
                    // when the pause began, and the "fell behind" branch
                    // at the bottom of this loop would attribute the
                    // entire paused duration to a single artificially slow
                    // frame in the fps window above.
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
                    SubmitFrame(session.GetFrameBufferRgba(), session.ScreenWidth, session.ScreenHeight);
                    SleepUntil(nextTick, clock);
                    continue;
                }

                session.SkipRendering = !_speed.ShouldRender(session.TotalFrames);

                try
                {
                    frameStopwatch.Restart();
                    session.RunFrame();
                    runFrameTimeInWindow += frameStopwatch.Elapsed;

                    // Publishes this frame's register/sprite/palette/audio/
                    // hardware-load snapshots via IDebugTarget's provider
                    // properties - see EmuSen.Cauldron.IRealtimeProvider's
                    // own comment. Null-conditional since _debugTarget isn't
                    // atomically tied to _session (a ROM swap in flight
                    // could momentarily leave one set without the other).
                    _debugTarget?.RefreshProviders();

                    // Runs whatever _consoleWindow queued (a mutating
                    // command typed while not fast-path-eligible - see
                    // DianaOSConsoleWindow.Submit and
                    // EmuSen.DianaOS.DianaOS.Bin.DianaOSInterpreterScheduler's own
                    // comment) against the core we just finished a frame
                    // on - null-conditional since the console window is
                    // opened on demand and may not exist at all. Must run
                    // on this (the emulation) thread, same reasoning as
                    // RefreshProviders() just above.
                    _consoleWindow.Current?.DrainPendingFromEmulationThread();

                    // Same rule, same thread: the Apply Cheats button only
                    // sets the flag - see _applyCheatsPending.
                    if (_applyCheatsPending)
                    {
                        _applyCheatsPending = false;
                        _debugTarget?.ApplyCheats();
                    }

                    // Same call-site placement as PumpAudio() in
                    // EmuSen.Hotaru/Program.cs - right after
                    // RunFrame(), since that's what actually produces new
                    // samples to drain. Safe here on _emuThread rather than
                    // the UI thread - see AudioPlayer.Pump's own comment.
                    if (session.Core is not null) _rewind.OnFrameCompleted(session.Core);

                    // Drained every frame either way, so a muted stretch can't back the core buffer up - see §2.3.
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
                    if (!session.SkipRendering)
                    {
                        byte[] frame = session.GetFrameBufferRgba();
                        SubmitFrame(frame, session.ScreenWidth, session.ScreenHeight);
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

                        Dispatcher.UIThread.Post(() => FpsText.Text =
                            $"{fps:F1} fps (run {runFrameMs:F2}ms / total {totalMs:F2}ms){breakdown}");
                        framesInWindow = 0;
                        Array.Clear(phaseMsInWindow);
                        runFrameTimeInWindow = TimeSpan.Zero;
                        fpsWindowStart = clock.Elapsed;
                    }
                }
                catch (Exception ex)
                {
                    _running = false;
                    string message = ex.Message;
                    // Stop rather than spamming the same exception every
                    // tick - matches the console/Raylib build's behavior of
                    // halting and printing on a core exception rather than
                    // trying to recover. StatusText is a UI element, so the
                    // update has to go through the UI thread.
                    Dispatcher.UIThread.Post(() => StatusText.Text = $"[CPU HALT] {message}");
                    break;
                }

                TimeSpan remaining = nextTick - clock.Elapsed;
                if (remaining > TimeSpan.Zero)
                {
                    SleepUntil(nextTick, clock);
                }
                else
                {
                    // Fell behind - resync to "now" instead of trying to
                    // burst-catch-up, which would just run a pile of frames
                    // back-to-back with no pacing at all.
                    nextTick = clock.Elapsed;
                }
            }
        }

        // A hybrid Sleep-then-spin (sleep for most of the remaining time,
        // busy-spin only the last ~2ms) turned out not to be enough here -
        // measured CPU usage while stuck at ~56fps was only ~5%, meaning
        // the process is nowhere near compute-bound; the shortfall is
        // entirely from oversleeping, not slow work. That means
        // Thread.Sleep's wakeup latency in this environment is bigger than
        // the 2ms margin the hybrid approach budgeted for it - rather than
        // guess at a bigger margin, spin-wait the ENTIRE remaining time
        // instead of calling Thread.Sleep at all. With this much headroom
        // (a handful of percent of one core, going by that measurement),
        // pegging a single core for the ~14ms/frame this spins is a
        // perfectly reasonable trade for hitting the 60fps target
        // precisely - the same tradeoff real-time audio/emulation loops
        // routinely make, at the cost of that one core's power draw.
        private static void SleepUntil(TimeSpan target, Stopwatch clock)
        {
            while (clock.Elapsed < target)
            {
                Thread.SpinWait(100);
            }
        }

        // Called from the emulation thread. Publishes the newest frame and
        // schedules a UI-thread Present only if one isn't already pending -
        // see _pendingFrame's own field comment for why.
        private void SubmitFrame(byte[] pixels, int width, int height)
        {
            _frames.Offer(new FrameData { Pixels = pixels, Width = width, Height = height });
        }

        // Runs on the UI thread. Always presents whatever the newest frame
        // is at the moment it actually runs, which may not be the same
        // frame that triggered this dispatch if the emulation thread has
        // since produced newer ones - that's the intended drop-stale-frames
        // behavior, not a bug. GameFrameControl.UpdateFrame takes the raw
        // buffer + dimensions directly (see EmuSen.Serenity), so pseudo-
        // hi-res width changes (SETINI bit 3) just fall out for free -
        // unlike the old WriteableBitmap this replaced, there's no fixed-
        // size backing surface to resize.
        private void PresentPendingFrame(FrameData frame)
        {
            GameFrame.UpdateFrame(frame.Pixels, frame.Width, frame.Height);
        }

        private void OnExitClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }

        // Reuses the console/Raylib build's own CategorizedLogWriter
        // (Common/CategorizedLogWriter.cs) rather than building a second
        // logging mechanism - it works by redirecting Console.Out, and
        // the emulation core already writes every diagnostic message
        // (Cartridge load info, DebugSettings-gated traces, etc.) via
        // plain Console.WriteLine, so this frontend gets the exact same
        // categorized cpu/ppu/apu/memory/debug/general log files the
        // console build does, for free.
        //
        // Unconditional, same as the console/Raylib build
        // (Hotaru/Program.cs) - several DebugSettings.*Logging
        // flags default to true (Dma/CameraRam/RenderRead/AllScrollWrite/
        // MathUnit/BgModeChange/MosaicWrite), so the core writes a steady
        // stream of Console.WriteLine calls regardless of whether anyone
        // asked for logging. Leaving Console.Out un-redirected in that
        // case doesn't turn logging off - it just sends the same volume
        // of lines to the raw, synchronous console writer one syscall at
        // a time instead of CategorizedLogWriter's batched background
        // thread, which is exactly the gap that made this frontend so
        // much slower than the console build. Falling back to an
        // AppSettings.LogDirectory default under %AppData% (rather than
        // skipping the redirect) keeps the fast path always on; the user
        // only needs to set LogDirectory in Preferences if they want the
        // files somewhere specific.
        //
        // Called once per LoadRom() (not once at app startup) since,
        // unlike the console build (one ROM per process), this frontend
        // can load several ROMs across one running session - each gets
        // its own timestamped directory, mirroring the console build's
        // Logs/<CoreName>/console_<timestamp>/ convention but under the
        // user-configured (or default) root and with a "gui_" prefix
        // instead.
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
                // Best-effort - an unwritable log directory shouldn't block
                // loading the ROM itself, just leave logging off for this
                // session and say why in the status bar.
                StatusText.Text = $"Logging disabled: {ex.Message}";
            }
        }

        private void StopLogging()
        {
            if (_activeLogWriter is null) return;

            // Must happen before Console.SetOut/Dispose below - see
            // EmulatorSession.FlushVerboseLogs()'s own comment.
            _session?.FlushVerboseLogs();

            Console.SetOut(_originalConsoleOut);
            _activeLogWriter.Dispose();
            _activeLogWriter = null;
        }
    }
}
