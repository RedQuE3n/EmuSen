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
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Serenity;
using EmuSen.Serenity.Dashboards;
using EmuSen.Mistress.Input;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.DianaSh;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Galaxia.Input;

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
        private readonly LunaAction _rewindReel;
        private readonly ActionGroup _speeds = new();
        private readonly ActionGroup _slots = new();

        // The game or empty console open on the frontend API; replaced only with the emulation thread stopped - see EmuSen_Backend.md §14.13.
        private GameSession? _session;

        private static CrystalRuntime Runtime => CoreSettings.Runtime;

        // Over the frame only, so the pointer stays visible on the menu - see §4.20.
        private IdleCursor? _idleCursor;
        private FileDrop? _fileDrop;

        // Held, not edge-triggered - see EmuSen_Rewind_And_FastForward.md §4.
        private volatile bool _turboHeld;
        private volatile bool _rewindHeld;

        // Two snapshots a second of console time, and a held rewind still plays back at about four times speed - see EmuSen_Settings_Reference.md §4.112.
        internal const double RewindSnapshotsPerSecond = 2, HeldRewindSpeedup = 4;
        private int _heldRewindTicks;

        internal static int RewindIntervalFor(double frameRateHz) => Math.Max(1, (int)Math.Round(frameRateHz / RewindSnapshotsPerSecond));

        // The runtime keeps the history; the reel's pictures ride along with each snapshot this many pixels wide - see EmuSen_Settings_Reference.md §4.49.
        internal const int ReelPictureWidth = 160;
        private int _rewindInterval = 30;

        // The Speed menu's choices, which the window sets through the API: the runtime owns the speed itself - see EmuSen_Settings_Reference.md §4.13.
        internal const int NormalPercent = 100, TurboPercent = 300, SlowMotionPercent = 25, UnthrottledPercent = 0;

        // What the Speed menu picked - see EmuSen_Settings_Reference.md §4.13.
        private volatile int _baseSpeedPercent = NormalPercent;

        // Which save slot Save/Load State act on, 1-8 - see §4.13.
        private int _stateSlot = 1;
        private const int StateSlots = 8;

        // Gamepad polling only, on the UI thread - see EmuSen_Settings_Reference.md §4.10.
        private DispatcherTimer? _timer;

        // RunFrame off the UI thread entirely - see EmuSen_Settings_Reference.md §4.21.
        private Thread? _emuThread;
        private volatile bool _running;

        // The session's run control holds a pause; this mirrors it for the menus, set by the window and by the session's run events - see `man pause`.
        private volatile bool _paused;

        public bool IsPaused => _paused;

        // A breakpoint holds the session until run control resumes it; the window says so and a resume lets it go (C2).
        private volatile bool _halted;

        // The last frame's facts, the frame thread's; and the session events it drains between frames - see EmuSen_Backend.md §14.13.
        private CrystalFrameInfo _info;
        private CrystalSubscription? _events;

        // Coalescing hand-off to the UI thread over LunaP's Latest (LunaP.md §22.1), giving a lent array back once - see EmuSen_Serenity.md §2.8.
        private readonly FrameHandOff _frames;

        // At most one, reused and brought forward rather than duplicated; cleared on Closed.
        private readonly WindowSlot<DianaShConsoleWindow> _consoleWindow = new();

        // Same rule, opened by typing `coretop` rather than from a menu item.
        private readonly WindowSlot<CoretopWindow> _coretopWindow = new();

        // Same rule, with no target to update - see `man vstop`.
        private readonly WindowSlot<VstopWindow> _vstopWindow = new();

        // Same rule, refreshed rather than re-targeted - see EmuSen_Settings_Reference.md §4.14.
        private readonly WindowSlot<ActiveCheatsWindow> _activeCheatsWindow = new();

        // Same at-most-one/reuse/clear-on-Closed pattern, retained so a console switch can retarget it.
        private readonly WindowSlot<CheatDatabaseWindow> _cheatDatabaseWindow = new();

        // The running session's cheats, or with no game a list held for the next, so a list outlives a session a restart replaces - see §4.14.
        private readonly CheatList _cheats = new();

        // Which ROM the cheats are for; not _currentRomPath, which a close resets - see §4.14.
        private string? _cheatsRomPath;

        private string? _currentRomPath;
        private string? _currentDisplayName; // for restoring StatusText's "Running: ..." text exactly after a pause, without reformatting from _currentRomPath
        private readonly ControllerKeyBindings _keyBindings =
            ControllerKeyBindings.Load(RuntimeCatalog.ConsoleNames);
        private readonly GamepadBindings _gamepadBindings =
            GamepadBindings.Load(RuntimeCatalog.ConsoleNames);

        // Follows the loaded ROM; catalog order stands in before one loads - see EmuSen_Input.md §5.1.
        private string _activeConsole = RuntimeCatalog.Consoles.FirstOrDefault()?.Console ?? "";
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
        private PortRouter _ports = null!;

        public MainWindow()
        {
            _frames = new FrameHandOff((pixels, width, height, rowRepeat, release, sequence, aspect, lines, raw, lend) => GameFrame.UpdateFrame(pixels, width, height, rowRepeat, release, sequence, aspect, lines, raw, lend));
            EarlyLoader = OpenSession;
            _pause = new LunaAction("_Pause", _ => TogglePause()) { IsCheckable = true };
            _reset = new LunaAction("_Reset", ResetEmulation);
            _closeGame = new LunaAction("_Close Game", ShowLibrary);
            _saveState = new LunaAction("Save _State", SaveState);
            _loadState = new LunaAction("_Load State", LoadState);
            _speedMenu = new LunaAction("Spee_d", () => { });
            _slotMenu = new LunaAction("State Sl_ot", () => { });
            _fullscreen = new LunaAction("_Fullscreen", a => IsFullScreen = a.IsChecked) { IsCheckable = true };
            _bigPictureMenu = new LunaAction("_Big Picture", () => SetBigPicture(!_bigScreen)) { HelpText = "Full screen, in big picture mode" };
            _hardwareDashboard = new LunaAction("_Hardware Dashboard...", OpenCoretopWindow);
            _rewindReel = new LunaAction("Re_wind...", () => OpenRewindReel(resumeAfter: false));
            InitializeComponent();
            // Why the game's picture is read back rather than handed to the compositor on the device, logged once a window - see EmuSen_DevicePicture.md §4.4.
            GameFrame.DevicePictureReport += problem => ErrorLog.Warning("picture", problem);
            SetUpStatusLine();
            ApplyStatusBar();
            SetUpLibraryScreen();
            BuildMenus();
            // The tick follows the window, so a window-manager key cannot leave it lying - see §4.19.
            FullScreenChanged += on => _fullscreen.IsChecked = on;
            // SDL's device scan takes ~150 ms, so it waits for the window's first frame - see EmuSen_Settings_Reference.md §4.42.
            _gamepad = new GamepadManager(_gamepadBindings.For(_activeConsole), start: false);
            _gamepad.PadChanged += OnPadChanged;
            _gamepad.Claimed += OnPadClaimed;
            ApplyPlayerRules();
            _ports = new PortRouter(_gamepad, (p, b, held) => Send(s => s.Button(p, b, held)), (p, a, v) => Send(s => s.Axis(p, a, v)), (p, on) => Send(s => s.Connected(p, on)))
            {
                KeyboardHeld = control => _keyboardHeld[(int)control],
            };
            Opened += (_, _) => RequestAnimationFrame(_ => Dispatcher.UIThread.Post(_gamepad.Start, DispatcherPriority.Background));
            // `--play <rom>` starts a game straight away, which the cadence measurement of §4.87.1 launches with.
            Opened += (_, _) => { if (PlayArgument(Environment.GetCommandLineArgs()) is { } rom) Dispatcher.UIThread.Post(() => LoadRom(rom, Path.GetFileNameWithoutExtension(rom)), DispatcherPriority.Background); };
            // Endymion is a leaf and reads no globals, so audio.json's settings come from here, the cores' with them - see EmuSen_Audio_Sync.md §7.1.
            AudioConfig audio = AudioConfig.Load().Bounded();
            _audioPlayer = new AudioPlayer(audio.SampleRate, audio.OutputTargetLatencyMs, audio.RateControlMaxDeviation);
            OpenOptions = new CrystalOpen { AudioBufferMaxSamples = audio.AudioBufferMaxSamples };
            // graphics.json's filtering and display sync, which the picture and the pacer read - see EmuSen_Settings_Reference.md §3, §4.87.
            GameFrame.BilinearFiltering = _graphics.BilinearFiltering;
            GameFrame.SyncToDisplay = _graphics.SyncToDisplay;
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
                CloseThemedLibrary();
                StopEmulationThread();
                WriteResumeState();
                RecordPlayTime();
                KeepCheatList();
                CloseSession();
                _events?.Dispose();
                StopScraping();
                StopOnlineCovers();
                CloseRecords();
                _gamepad.Dispose();
                PadHints.Swapped = false;
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
            AddHandler(PointerMovedEvent, (_, _) => ScreensaverPointer(pressed: false), RoutingStrategies.Tunnel, handledEventsToo: true);
            AddHandler(PointerPressedEvent, (_, e) => { if (ScreensaverOpen) { ScreensaverPointer(pressed: true); e.Handled = true; } else ScreensaverPointer(pressed: false); }, RoutingStrategies.Tunnel, handledEventsToo: true);
        }

        // A focused text field owns the whole keyboard - see EmuSen_Settings_Reference.md §4.17.
        private static bool TypingIntoATextField(RoutedEventArgs e) =>
            e.Source is Visual source && source.FindAncestorOfType<TextBox>(includeSelf: true) is not null;

        private void SetButtonFromKey(Key key, bool pressed, KeyEventArgs e)
        {
            if (ScreensaverKey(key, pressed))
            {
                e.Handled = true;
                return;
            }
            // In a big-screen menu a text field is a row, which ES-DE's keys move past rather than type into; the pad router's own keys are never the keyboard's (Q102).
            bool routed = PadWindowRouter.Raising;
            // A key let go that was steering is released as a button whatever now has the focus, as the text popup takes it on Enter's press (§4.79).
            if (!pressed && !routed && ThemedKeyButton(key) is { } held && _themedKeys.Remove(held))
            {
                e.Handled = true;
                return;
            }
            if (TypingIntoATextField(e) && (routed || !(BigMenuOnScreen && !TextEntryOpen) || FramedTextBoxKeepsKey(key))) return;
            if (!routed && ThemedKey(key, e.KeyModifiers, pressed))
            {
                e.Handled = true;
                return;
            }

            // A suspended game must not collect the keys used to browse the library - see EmuSen_Settings_Reference.md §4.18.
            // A sheet over the game takes its keys; they are not the game's - see EmuSen_Settings_Reference.md §4.45.2.
            if (!LibraryView.IsVisible && !Sheets.IsPresenting && _keyBindings.For(_activeConsole).TryGetControl(key, out PadControl control))
            {
                _keyboardHeld[(int)control] = pressed;
                ApplyKeys();
                // Or the menu bar, the only focusable control here, also gets the key - see EmuSen_Settings_Reference.md §4.24.
                e.Handled = true;
                return;
            }

            if (pressed && EditMetadataFromTheKeyboard(e))
            {
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
                case HotkeyAction.ExitToLibrary: if (EscapeLeavesBigPicture) SetBigPicture(false); else ToggleLibrary(); break;
                case HotkeyAction.Screenshot: TakeScreenshot(); break;
                // Nothing to pause while the library is up: it is already suspended - see §4.18.
                case HotkeyAction.TogglePause: if (!LibraryView.IsVisible) TogglePause(); break;
                // Through ToolWindow, so leaving goes back to the state full screen was entered from, a maximised window included.
                case HotkeyAction.ToggleFullscreen: ToggleFullScreen(); break;
                case HotkeyAction.ToggleBigPicture: SetBigPicture(!_bigScreen); break;
            }
            e.Handled = true;
        }

        // Through ICore, not the bus; each player's pad to its own port and the keyboard to its player's - see EmuSen_Input.md §8.5.
        private void ApplyKeys()
        {
            if (_session is null) return;
            RoutePlayers();
            _ports.KeysChanged();
        }

        private void RoutePlayers()
        {
            _ports.KeyboardPlayer = Math.Clamp(_appSettings.KeyboardPlayer, 1, PlayerSlots.MaxPlayers);
            _ports.MirrorPlayer1ToPlayer2 = _appSettings.MirrorPlayer1ToPlayer2;
            _gamepad.FirstControllerOnly = _appSettings.FirstControllerOnly;
            ApplyPlayerRules();
        }

        // Both before any pad is seated and on every poll, so a change in Preferences applies at once - see EmuSen_Input.md §8.13.
        private void ApplyPlayerRules()
        {
            _gamepad.ClaimPlayersByPress = _appSettings.ClaimPlayersByPress;
            _gamepad.ExternalControllerFirst = _appSettings.ExternalControllerFirst;
        }

        private void PollGamepad()
        {
            _gamepad.Poll();

            // A menu over the game, or the button that closed one, is not the game's - see §4.29.
            if (PadBelongsToTheInterface()) return;
            if (_session is null) return;

            // A core whose ports changed while the game runs, as by an adapter set on a port, is heard on its new ports - see EmuSen_Input.md §8.11.
            if (_session.ControllerPorts != _ports.Ports) _ports.Resize(_session.ControllerPorts);

            // Buttons go when they change; a stick moves without crossing any threshold, so the axes go every poll.
            RoutePlayers();
            _ports.PollPads();
        }

        private async Task OpenRomAsync()
        {
            // One entry per core in this build, so a new core needs no edit here.
            FilePickerFileType[] types = RuntimeCatalog.Consoles
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

        // Runs before the game opens because afterwards is too late, asking the runtime what the engine that will run it needs; declining is fine - see EmuSen_Firmware.md §3.
        private async Task PromptForMissingFirmwareAsync(string romPath)
        {
            string needed;
            try { needed = Runtime.FirmwareNeeded(romPath); }
            catch (CrystalException) { return; }
            using System.Text.Json.JsonDocument requests = System.Text.Json.JsonDocument.Parse(needed);
            foreach (System.Text.Json.JsonElement request in requests.RootElement.EnumerateArray())
            {
                string chip = request.GetProperty("chip").GetString() ?? "", file = request.GetProperty("file").GetString() ?? "";
                long size = request.GetProperty("size").GetInt64();
                var types = new[]
                {
                    new FilePickerFileType($"{chip} firmware") { Patterns = new[] { file, "*.rom", "*.bin" } },
                    new FilePickerFileType("All files") { Patterns = new[] { "*" } },
                };

                string title = $"{chip} firmware needed - pick {file} ({size:N0} bytes), or cancel to play without it";
                if (await Dialogs.PickFileAsync(this, title, types) is not { } chosen)
                {
                    StatusText.Text = $"No {chip} firmware selected - that chip will not be emulated.";
                    continue;
                }

                try
                {
                    Runtime.FirmwareInstall(request.GetRawText(), chosen.Path);
                    StatusText.Text = $"Installed {chip} firmware.";
                }
                catch (CrystalException)
                {
                    StatusText.Text = $"{chosen.Name} is not a {chip} dump ({size:N0} bytes expected) - that chip will not be emulated.";
                }
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
            string? selected = await SheetLayer.ShowDialog<string>(new RomBrowserWindow(_appSettings.RomDirectory), this);
            if (selected is null) return;

            await StartGameAsync(selected, Path.GetFileName(selected));
        }

        private void ShowControllerBindings()
        {
            var window = new InputSettingsWindow(_keyBindings, _gamepadBindings, _gamepad, _appSettings, _hotkeyBindings,
                _session is null ? null : _activeConsole, _session?.PortControllers);
            // A rebind has to reach the menu, or it advertises the old key - see §4.19.
            window.Closed += (_, _) => SyncMenuState();
            _ = SheetLayer.Show(window, this);
        }

        private void ShowGraphicsSettings()
        {
            // A core's setting reaches a running session through the runtime's settings_set; the picture's are the window's own - see §4.26.
            var window = new GraphicsSettingsWindow(_graphics, console =>
            {
                if (_session is not null && console == _activeConsole) ApplyScreenFilter(console);
            }, _session is null ? null : _activeConsole, HttpFactory, overscanNote: RunningGameCropNote) { ShadersChanged = ShaderChanged };
            _ = SheetLayer.Show(window, this);
        }

        // Every console's memory cards and the like, on the running game's console, whose held cards it shows and does not change - see EmuSen_SaveMedia.md §6.5.
        private void ShowMemoryCards() => _ = SheetLayer.Show(new MemoryCardsWindow(_graphics, _session is null ? null : _activeConsole), this);

        // Its own window, on the running game's console - see EmuSen_Settings_Reference.md §4.48.
        private void ShowShaderSettings() =>
            _ = SheetLayer.Show(new ShaderSettingsWindow(_graphics, ShaderChanged, _session is null ? null : _activeConsole, HttpFactory), this);

        private void ShaderChanged(string console)
        {
            if (_session is not null && console == _activeConsole) ApplyScreenFilter(console);
        }

        // On the UI thread: the filter belongs to the control that draws, not to the core - see EmuSen_Settings_Reference.md §4.40.
        private void ApplyScreenFilter(string console)
        {
            string? stored = _graphics.Value(console, GraphicsSettingsWindow.ScreenFilterKey);
            EmuSen.Serenity.Shaders.ScreenFilterChoice choice = EmuSen.Serenity.Shaders.ScreenFilters.Find(stored);
            // Before the shader, so a chain or runner built for it starts from the player's values - see EmuSen_Settings_Reference.md §4.48.3.
            GameFrame.ShaderParameters = RunningStandard(ShaderParameterValues(console, stored ?? choice.Name));
            GameFrame.ActiveEffect = choice.Effect;
            // Before the filter, whose chain starts from this console's defaults and constants - see EmuSen_Serenity.md §3.9.
            GameFrame.FilterConsole = RuntimeCatalog.HostOf(console);
            GameFrame.SquarePixels = _graphics.Value(console, GraphicsSettingsWindow.PictureShapeKey) == GraphicsSettingsWindow.SquarePixels;
            GameFrame.Crop = RunningCrop(console);
            GameFrame.ActiveFilter = choice.Filter;
            GameFrame.ActiveSlangPreset = SlangPresetPath(stored);
            GameFrame.InvalidateVisual();
        }

        private Dictionary<string, float> ShaderParameterValues(string console, string shader)
        {
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach ((string id, string text) in _graphics.ParametersFor(console, shader))
                if (float.TryParse(text, System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out float value)) values[id] = value;
            return values;
        }

        private bool _slangFailuresShown;

        // A stored "slang:" value as a file in the downloaded pack, or null; a preset since deleted is said once and drawn plain.
        private string? SlangPresetPath(string? stored)
        {
            if (stored is null || !stored.StartsWith(GraphicsSettingsWindow.SlangPrefix, StringComparison.Ordinal)) return null;
            if (!_slangFailuresShown)
            {
                _slangFailuresShown = true;
                GameFrame.SlangFailed += problem => Dispatcher.UIThread.Post(() => StatusText.Text = $"RetroArch shader not drawn: {problem}");
            }
            string relative = stored[GraphicsSettingsWindow.SlangPrefix.Length..];
            string path = System.IO.Path.GetFullPath(System.IO.Path.Combine(EmuSen.Mistress.Library.SlangPackDownload.DefaultDirectory, relative));
            if (System.IO.File.Exists(path)) return path;
            StatusText.Text = $"RetroArch shader not drawn: {relative} is not in the downloaded pack.";
            return null;
        }

        private void ShowPreferences() => ShowPreferencesAt(null);

        private void ShowPreferencesAt(string? tab)
        {
            // Non-modal, so re-scan on close rather than leaving a stale library behind it.
            var window = new PreferencesWindow(_appSettings, this, _bigScreen ? HelpFamily : null, _gamepad, () => _session?.ControllerPorts ?? 0) { OpenThemeSettings = ShowThemeSettings, Graphics = _graphics, SyncToDisplayChanged = on => GameFrame.SyncToDisplay = on };
            WatchPreferences(window);
            if (tab is not null) window.ShowTab(tab);
            window.StatusBarChanged += ApplyStatusBar;
            // Not when it closes because this window is closing, after the records it reads have gone - see EmuSen_Settings_Reference.md §4.88.
            window.Closed += (_, _) => { if (_recordsClosed) return; ScanArtwork(); ApplyOnlineCovers(); ApplyScraping(); if (LibraryView.IsVisible) RefreshLibrary(); };
            _ = SheetLayer.Show(window, this);
        }

        // The bar goes when it is switched off or when both its parts are; a big screen's counters are its HUD's, not the bar's - see EmuSen_Settings_Reference.md §4.51 and §4.83.6.
        private void ApplyStatusBar()
        {
            bool counters = _appSettings.ShowFpsBar && !_bigScreen;
            StatusText.IsVisible = _appSettings.ShowStatusText;
            FpsText.IsVisible = counters;
            StatusBar.IsVisible = _appSettings.ShowStatusBar && (_appSettings.ShowStatusText || counters) && !(ThemedLibraryShown && LibraryView.IsVisible);
        }

        private void ShowDebugLogging()
        {
            _ = SheetLayer.Show(new DebugSettingsWindow(Runtime), this);
        }

        // The running game's console, else the library's filter: cheats are for the game being played - see EmuSen_Settings_Reference.md §4.45.5.
        private string CheatConsole => _session is not null ? _activeConsole : CoreOfShelf(SelectedConsole);

        // The cheat windows know cores, not shelves: the Game Boy Color shelf is Mercury's - see EmuSen_Settings_Reference.md §4.46.
        private static string CoreOfShelf(string shelf) => RuntimeCatalog.ShelfByName(shelf)?.Core.DisplayName ?? shelf;

        // Never needs a ROM: it manages a folder and a list, not a session; the database's ops need no game. See §4.14.
        private void ShowCheatDatabase()
        {
            _cheatDatabaseWindow.Show(this, () => new CheatDatabaseWindow(_appSettings, _cheats, () => _session?.Crystal,
                () => _activeCheatsWindow.Current?.Refresh(),
                ShowActiveCheats,
                CheatConsole));
        }

        // The menu item and the database window's button are the same door - see §4.14.
        private void ShowActiveCheats()
        {
            _activeCheatsWindow.Show(this, () => new ActiveCheatsWindow(_cheats,
                () => CheatsFor(_cheatsRomPath),
                CheatConsole,
                ShowCheatDatabase),
                refresh: w => w.Refresh());
        }

        // In the console's folder, so two consoles' games of one name keep separate lists - see EmuSen_Config_Reference.md §3.4b.
        private static GameCheats? CheatsFor(string? romPath) =>
            GameCheats.For(romPath is null ? null : RuntimeCatalog.ConsoleForRom(romPath), romPath);

        // Whatever was saved for this ROM, brought back; returns how many - see §4.15.
        private int LoadSavedCheatsFor(string path)
        {
            CheatFile? saved = CheatsFor(path)?.Load();
            if (saved is null) return 0;

            (int loaded, _) = _cheats.Import(saved);
            _activeCheatsWindow.Current?.Refresh();
            return loaded;
        }

        private void ShowShellConsole()
        {
            _consoleWindow.Show(this, () => new DianaShConsoleWindow(_session?.Crystal, MakeEmulationControlCommands()));
        }

        // Built fresh per console window, so the delegates never outlive a MainWindow.
        private IEnumerable<IDianaShCommand> MakeEmulationControlCommands() => new IDianaShCommand[]
        {
            new PauseCommand(() => Dispatcher.UIThread.Post(PauseEmulation), () => IsPaused),
            new ResumeCommand(() => Dispatcher.UIThread.Post(ResumeEmulation), () => IsPaused || _halted),
            new CoretopWindowCommand(() => Dispatcher.UIThread.Post(OpenCoretopWindow), () => _session is not null),
            new VstopWindowCommand(() => Dispatcher.UIThread.Post(OpenVstopWindow)),
            new FeedCommand(() => Dispatcher.UIThread.Post(() => Activate())),
            new EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.StateCommand(SaveStateFromConsole, LoadStateFromConsole, () => CurrentStatePath ?? ""),
        };

        // Not method groups: _session can still be null at construction time; the runtime clears the history and raises the load's event - see `man state`.
        private void SaveStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.Crystal.SaveState(path);
        }

        private void LoadStateFromConsole(string path)
        {
            if (_session is null) throw new InvalidOperationException("No ROM loaded.");
            _session.Crystal.LoadState(path);
        }

        // Over the window's session as the API answers it, read at each poll, so a new game needs no update - see EmuSen_Backend.md §14.13.
        private void OpenCoretopWindow() => _coretopWindow.Show(this, () => CoretopWindow.Over(new SessionReadout(() => _session?.Crystal)));

        // No target to hand over or refresh, unlike OpenCoretopWindow above.
        private void OpenVstopWindow() => _vstopWindow.Show(this, () => new VstopWindow());

        // The same tree `state save` writes to, overridable in Preferences.
        private string? CurrentStatePath => StatePathForSlot(_stateSlot);

        // A slot's file as the running session's states name it, slot 1 the plain <rom>.state every other frontend writes; null with no game - see §4.13.
        private string? StatePathForSlot(int slot) => SlotPaths() is { } paths && slot < paths.Length ? paths[slot] : null;

        // The session's slots, the resume state's first, read once a menu sync - see EmuSen_Backend.md §14.13.
        private string[]? _slotPaths;
        private GameSession? _slotPathsOf;

        private string[]? SlotPaths()
        {
            if (_session is not { } session) return null;
            if (ReferenceEquals(_slotPathsOf, session) && _slotPaths is { } known) return known;
            try
            {
                using System.Text.Json.JsonDocument slots = System.Text.Json.JsonDocument.Parse(session.Crystal.StateSlots());
                _slotPaths = slots.RootElement.EnumerateArray().OrderBy(e => e.GetProperty("slot").GetInt32()).Select(e => e.GetProperty("path").GetString() ?? "").ToArray();
                _slotPathsOf = session;
                return _slotPaths;
            }
            catch (CrystalException) { return null; }
        }

        private void SaveState()
        {
            if (_session is not { } session || CurrentStatePath is not string path)
            {
                StatusText.Text = "Save State: no ROM loaded";
                return;
            }

            int slot = _stateSlot;
            // Off the UI thread: the runtime applies it at the next frame boundary, the caller waiting - see EmuSen_Backend.md §14.3.
            Task.Run(() =>
            {
                string status;
                try
                {
                    session.Crystal.SaveSlot((uint)slot);
                    status = $"State saved to slot {slot}: {System.IO.Path.GetFileName(path)}";
                }
                catch (Exception ex) when (ex is CrystalException or IOException or InvalidOperationException)
                {
                    ErrorLog.Error("states", $"Save State to slot {slot} failed", ex, path);
                    status = $"Save State failed: {ex.Message}";
                }

                // SyncMenuState because that slot has a timestamp now.
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; SyncMenuState(); Notify(status.StartsWith("State saved", StringComparison.Ordinal) ? $"State saved to slot {slot}" : status); });
            });
        }

        private void LoadState() => _ = LoadStateAsync();

        // Runs straight through to the load unless the state's record names another copy of the game; what may load is the runtime's to say - see EmuSen_Settings_Reference.md §4.37 and EmuSen_Backend.md §14.12.
        private async Task LoadStateAsync()
        {
            if (_session is not { } session || CurrentStatePath is not string path)
            {
                StatusText.Text = "Load State: no ROM loaded";
                return;
            }
            string rom = _currentRomPath ?? "";

            if (!System.IO.File.Exists(path))
            {
                StatusText.Text = $"Load State: slot {_stateSlot} is empty";
                return;
            }

            StateRecord? record = StateRecordsOnApi.Read(Runtime, path);
            if (!await SameGameOrConfirmedAsync(record, rom)) return;

            int slot = _stateSlot;
            await Task.Run(() =>
            {
                string status, notice;
                try
                {
                    session.Crystal.LoadSlot((uint)slot);
                    status = $"State loaded from slot {slot}: {System.IO.Path.GetFileName(path)}";
                    notice = $"State loaded from slot {slot}";
                }
                catch (CrystalException ex) when (ex.Status == CrystalAbi.StateRefused)
                {
                    status = $"Load State: {ex.Message}";
                    notice = "State not loaded";
                }
                catch (Exception ex) when (ex is CrystalException or IOException or InvalidOperationException)
                {
                    ErrorLog.Error("states", $"Load State from slot {slot} failed", ex, path);
                    status = $"Load State failed: {ex.Message}{Provenance(record)}";
                    notice = status;
                }

                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(notice); });
            });
        }

        // A call into the running session that may find it closed under it.
        private void Send(Action<CrystalSession> call)
        {
            if (_session is not { } session) return;
            try { call(session.Crystal); }
            catch (CrystalException) { }
        }

        // The four menus - shortcuts stay with HotkeyBindingMap, see EmuSen_Settings_Reference.md §4.19.
        private void BuildMenus()
        {
            _speedMenu.Submenu = new LunaMenu("Spee_d",
                _speeds.Add(new LunaAction("_Normal", () => SetBaseSpeed(NormalPercent))),
                _speeds.Add(new LunaAction("_Fast Forward", () => SetBaseSpeed(TurboPercent))),
                _speeds.Add(new LunaAction("_Slow Motion", () => SetBaseSpeed(SlowMotionPercent))),
                _speeds.Add(new LunaAction("_Unthrottled", () => SetBaseSpeed(UnthrottledPercent))));

            BuildSaveSlotItems();

            MenuStrip.SetMenus(
                new LunaMenu("_File",
                    new LunaAction("_Open ROM...", () => _ = OpenRomAsync()),
                    new LunaAction("_Browse ROMs...", () => _ = BrowseRomsAsync()),
                    StartConsoleMenuItems(),
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
                    _saveState, _loadState, _slotMenu, _rewindReel,
                    LunaAction.Separator(),
                    DriveMenuItems()[0], DriveMenuItems()[1]),
                new LunaMenu("_View", _asGrid!, _asList!,
                    new LunaAction("_Larger Covers", () => TileScale.Value = Math.Min(MaximumTileScale, TileScale.Value + 0.25)),
                    new LunaAction("S_maller Covers", () => TileScale.Value = Math.Max(MinimumTileScale, TileScale.Value - 0.25)),
                    LunaAction.Separator(), _fullscreen, _bigPictureMenu),
                new LunaMenu("_Settings",
                    new LunaAction("_Controller Bindings...", ShowControllerBindings),
                    new LunaAction("_Graphics Settings...", ShowGraphicsSettings),
                    new LunaAction("_Shaders...", ShowShaderSettings),
                    new LunaAction("_Debug Logging...", ShowDebugLogging),
                    new LunaAction("_Preferences...", ShowPreferences),
                    new LunaAction("Chea_t Database...", ShowCheatDatabase),
                    new LunaAction("_Active Cheats...", ShowActiveCheats),
                    new LunaAction("_Memory Cards...", ShowMemoryCards),
                    LunaAction.Separator(),
                    _hardwareDashboard,
                    // Reports on the host VM, so unlike the one above it never needs a ROM - see `man vstop`.
                    new LunaAction("_Runtime Dashboard...", OpenVstopWindow),
                    new LunaAction("_DianaSh Console...", ShowShellConsole)));

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
            Gesture(_bigPictureMenu, HotkeyAction.ToggleBigPicture);
        }

        private void Gesture(LunaAction action, HotkeyAction bound)
        {
            action.Shortcut = _hotkeyBindings.ActionToKey.TryGetValue(bound, out Key key) ? new KeyGesture(key) : null;
            action.HelpText = HotkeyBindingMap.DisplayName(bound);
        }

        // Called wherever the state actually changes, not when a menu opens - see §4.12.
        private void SyncMenuState()
        {
            bool running = _session is not null;
            _pause.IsEnabled = running;
            _reset.IsEnabled = running;
            _closeGame.IsEnabled = running;
            _speedMenu.IsEnabled = running;
            _saveState.IsEnabled = running;
            _loadState.IsEnabled = running;
            _slotMenu.IsEnabled = running;
            _rewindReel.IsEnabled = running;
            _pause.IsChecked = running && IsPaused;
            SyncDriveMenu();

            // Checking a member unchecks its siblings and runs no handler; ActionGroup.Checked is read-only.
            _speeds.Members[SpeedIndex(_baseSpeedPercent)].IsChecked = true;
            BuildSaveSlotItems();

            _saveState.Text = $"Save _State (slot {_stateSlot})";
            _loadState.Text = $"_Load State (slot {_stateSlot})";
            _hardwareDashboard.IsEnabled = _session is not null;

            ShowHotkeysOnTheMenu();
            SyncHud();
        }

        private int SpeedIndex(int percent)
        {
            if (percent == TurboPercent) return 1;
            if (percent == SlowMotionPercent) return 2;
            if (percent == UnthrottledPercent) return 3;
            return 0;
        }

        private void SetBaseSpeed(int percent)
        {
            _baseSpeedPercent = percent;
            Send(ApplySpeed);
            _audioPlayer.RateControl.Reset(); // the device's pacing target just moved - see EmuSen_Audio_Sync.md §3.2
            if (_session is not null) StatusText.Text = $"Speed: {DescribeSpeed(percent)}";
            SyncMenuState();
        }

        private static string DescribeSpeed(int percent) =>
            percent == UnthrottledPercent ? "unthrottled" : $"{percent}%";

        // The menu's speed as the session's normal one, with the turbo and slow motion the hotkeys and the menu name.
        private void ApplySpeed(CrystalSession session) => session.Speed((uint)_baseSpeedPercent, TurboPercent, SlowMotionPercent);

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
            if (_session is not null) StatusText.Text = $"Save slot {slot} selected ({DescribeSlot(slot)})";
            SyncMenuState();
        }

        // A list changed since it was saved is written when its game is left, so a change survives without Save - see §4.15b.
        private void KeepCheatList()
        {
            if (CheatsFor(_cheatsRomPath) is not { } target) return;
            CheatFile list;
            try { list = _cheats.ToFile(); }
            catch (CrystalException) { return; }
            bool? kept = target.SaveIfChanged(list);
            if (kept == true) Console.WriteLine($"[cheats] Kept {list.Cheats.Count} cheat(s) for {target.Name} in {target.File.Path}");
            else if (kept == false) ErrorLog.Error("cheats", $"Could not keep the cheat list for {target.Name}", new IOException(target.File.Path));
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
            return !sameGame && _cheats.Count == 0 ? LoadSavedCheatsFor(path) : 0;
        }

        // The console's own reset button where its core claims one, else the game started again - decided 2026-10-18 by the tester; see EmuSen_Settings_Reference.md §4.12 and EmuSen_Backend.md §14.13.
        private void ResetEmulation()
        {
            if (_session is not { } session) return;
            ResumeEmulation(); // a reset always comes back running, however it was paused
            try
            {
                session.Crystal.Reset();
                _audioPlayer.RateControl.Reset();
                StatusText.Text = $"Reset: {_currentDisplayName}";
                return;
            }
            catch (CrystalException ex) when (ex.Status == CrystalSessionAbi.NotSupported)
            {
                Console.WriteLine($"[core] {ex.Message}; the game is started again instead");
            }
            catch (CrystalException ex)
            {
                StatusText.Text = $"Reset failed: {ex.Message}";
                return;
            }

            if (_consoleSession is string console)
            {
                StartConsole(console, reset: true);
                if (_session is not null) StatusText.Text = $"Reset: {_currentDisplayName}";
                return;
            }
            if (_currentRomPath is not string path || _currentDisplayName is not string displayName) return;

            LoadGame(path, displayName, resumeFrom: null, reset: true);
            if (_session is not null) StatusText.Text = $"Reset: {displayName}";
        }

        // Everything that must happen before _session can be replaced or dropped; the session is closed, which writes its battery save.
        private void ShutDownCurrentSession(bool leaving = true)
        {
            _timer?.Stop();
            StopEmulationThread(); // must fully stop before _session changes - see that method's own comment
            _frames.EndSession(); // a picture the screen never took goes back - see EmuSen_Serenity.md §2.8
            // A Reset starts the same game again, so it is not leaving it - see EmuSen_Settings_Reference.md §4.31.
            if (leaving)
            {
                WriteResumeState();
                RecordPlayTime();
                KeepCheatList();
            }
            CloseSession();
        }

        // The list kept for the next game, then the session closed: its battery save written and its media let go.
        private void CloseSession()
        {
            _cheats.Detach();
            GameSession? closing = _session;
            _session = null;
            _slotPaths = null;
            closing?.Dispose();
        }

        internal static string? PlayArgument(string[] args)
        {
            int i = Array.IndexOf(args, "--play");
            return i >= 0 && i + 1 < args.Length && File.Exists(args[i + 1]) ? args[i + 1] : null;
        }

        private void LoadRom(string path, string displayName) => LoadGame(path, displayName, resumeFrom: null, reset: false);

        // resumeFrom is a state to load before the first frame runs, and loading a game already opened off this thread - see EmuSen_Settings_Reference.md §4.31.
        private void LoadGame(string path, string displayName, string? resumeFrom, bool reset, Task<GameSession>? loading = null)
        {
            ShutDownCurrentSession(leaving: !reset);
            int restoredCheats = DropCheatsFromAnotherGame(path);

            try
            {
                // Off the path, not the session: nothing is open yet to ask - see EmuSen_Multicore.md §12.
                StartLogging(RuntimeCatalog.ConsoleForRom(path) ?? "Unknown");
                GameSession session = loading?.GetAwaiter().GetResult() ?? OpenSession(new CrystalOpen { Path = path });
                _session = session;
                if (session.EngineNotice is { } engineNotice) Console.WriteLine("[core] " + engineNotice);

                // The runtime's console, not the core's name: a Game Boy core says GBC while it is a Color, and the GB tab is its settings - see §4.47.
                _activeConsole = session.Console;

                // Recorded before the picture's settings, which take the running game's own crop and standard - see EmuSen_CRT.md §26.6.
                _currentRomPath = path;
                ApplyScreenFilter(_activeConsole);
                // A half-loaded state is not a machine to run on, so the game starts again from nothing.
                if (resumeFrom is not null && TryResume(session, resumeFrom) is string resumeFailure)
                {
                    CloseSession();
                    LoadGame(path, displayName, resumeFrom: null, reset: false);
                    StatusText.Text = resumeFailure;
                    return;
                }
                WireSession(session, displayName, restoredCheats, newGame: !reset);
                _consoleSession = null;
                _currentDisplayName = displayName;
                if (!reset) RecordStart(path);
                RunSession(session);
            }
            catch (Exception ex) when (ex is CrystalException or IOException or InvalidOperationException)
            {
                CloseSession();
                // Back to the list rather than a black viewport with only a status line.
                ShowLibrary();
                ErrorLog.Error("launch", $"Failed to load {displayName}", ex);
                StatusText.Text = $"Failed to load {displayName}: {ex.Message}";
            }
        }

        // A session opened on the runtime with the presenter's device to lend and the window's options; any thread.
        private GameSession OpenSession(CrystalOpen what)
        {
            EmuSen.Serenity.Slang.SlangVulkan? device = PresenterDevice;
            _lentDevice = device;
            return GameSession.Open(Runtime, OpenOptions with { Path = what.Path, Console = what.Console, Engine = what.Engine, Device = device?.DeviceHandle ?? 0, SingleRows = true });
        }

        // The presenter's device, read on the UI thread before a session opens on any thread; the one the open session was lent, for borrowing its lend.
        private EmuSen.Serenity.Slang.SlangVulkan? PresenterDevice => GameFrame.LendableDevice;
        private volatile EmuSen.Serenity.Slang.SlangVulkan? _lentDevice;

        // A loaded session's pad, ports, rewind, speed, cheats, shell and screen, and the status line naming it.
        private void WireSession(GameSession session, string displayName, int restoredCheats, bool newGame)
        {
            _gamepad.Bindings = _gamepadBindings.For(_activeConsole);
            string bindingsConsole = _activeConsole;
            _gamepad.PlayerBindings = player => _gamepadBindings.For(bindingsConsole, player);
            _gamepad.LeftStickIsAnalog = session.Axes.Contains(PadAxis.LeftX);
            _ports.Reset(session.ControllerPorts, session.Axes);
            // A new game is claimed again by the first press; a restart keeps its players - see EmuSen_Input.md §8.13.
            if (newGame) _gamepad.BeginClaims();

            // The history the runtime keeps, a snapshot every half second of console time with its reel picture - see §4.49.
            _rewindInterval = RewindIntervalFor(session.FrameRateHz);
            session.Crystal.Rewind(true, (uint)_rewindInterval, 0, ReelPictureWidth);
            ApplySpeed(session.Crystal);
            _audioPlayer.RateControl.Reset();
            _info = default;
            _paused = false;
            _halted = false;
            _cheats.Attach(session.Crystal);
            _events ??= Runtime.Subscribe(0, CrystalAbi.EventState | CrystalAbi.EventRun | CrystalAbi.EventSettings | CrystalAbi.EventDisc);

            _consoleWindow.Current?.UpdateTarget(session.Crystal, displayName);
            _activeCheatsWindow.Current?.Refresh();

            GameFrame.IsVisible = true;
            LibraryView.IsVisible = false;

            StatusText.Text = restoredCheats > 0
                ? $"Running: {displayName}  ({restoredCheats} saved cheat(s) restored)"
                : $"Running: {displayName}";
            if (session.EngineNotice is { } notice) StatusText.Text += $"  ({notice})";
            if (session.FirmwareNotice is { } firmware) StatusText.Text += $"  ({firmware})";
            if (session.MediaNotice is { } media) StatusText.Text += $"  ({media})";
        }

        // The wired session started: the pad polled and the emulation thread running.
        private void RunSession(GameSession session)
        {
            // The press that launched the game is not the game's until every button is let go, as after a menu - see EmuSen_Input.md §8.13.9.
            _padQuiet = true;
            // Gamepad polling only; emulation is _emuThread's - see §4.21.
            _timer = new DispatcherTimer { Interval = FrameInterval };
            _timer.Tick += (_, _) => PollGamepad();
            _timer.Start();

            StartEmulationThread();
            SyncMenuState();
        }

        // Steps out of a game without unloading it, unlike ShowLibrary - see EmuSen_Settings_Reference.md §4.18.
        private void ToggleLibrary()
        {
            if (_session is null) return;

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
            _currentRomPath = null;
            _consoleSession = null;
            _currentDisplayName = null;

            // Or an open console keeps a shell over the session just closed; the dashboard reads whatever session is open.
            _consoleWindow.Current?.UpdateTarget(null, null);

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
            _cheatDatabaseWindow.Current?.SetConsole(CoreOfShelf(chosen));
            _activeCheatsWindow.Current?.SetConsole(CoreOfShelf(chosen));
        }

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
                LibraryFilter.SetFacets(RuntimeCatalog.FilterChoices,
                    RuntimeCatalog.FilterChoices.Contains(SelectedConsole)
                        ? SelectedConsole
                        : RuntimeCatalog.AllConsoles);

                // Safe to set before wiring: FilterBar stopped raising Changed for a code-set SearchText in LunaP 0.10.0 - see EmuSen_LunaP.md §14.2.
                LibraryFilter.SearchText = _appSettings.LibrarySearch;

                LibraryFilter.Changed += OnLibraryFilterChanged;
                LibraryFilter.Submitted += LaunchSelectedLibraryEntry;
            }

            // The disk walk happens here, once for every console, with each file's route asked again - see EmuSen_Backend.md §14.13.
            RuntimeCatalog.Forget();
            _allScan = RomLibrary.Scan(_appSettings.RomDirectory);
            _libraryScan = RomLibrary.Narrow(_allScan, SelectedConsole);
            ShowLibraryEntries();
            ReclaimMovedGames();
        }

        private void ShowLibraryEntries()
        {
            UpdateStartConsoleButton();
            string search = LibraryFilter.SearchText;
            _recordSnapshot = _records.All();
            _recentSnapshot = Recent(Listed(_allScan.Entries));
            _editSnapshot = _records.AllEdits();
            _collections = _records.Collections();
            ShowThemedLibrary();
            RomEntry? keptSelection = LibraryList.Selected;
            bool sameSearch = search == _lastLibrarySearch;
            _lastLibrarySearch = search;
            IReadOnlyList<RomEntry> pool = InCollection(Listed(_libraryScan.Entries));

            IReadOnlyList<RomEntry> shownEntries = string.IsNullOrWhiteSpace(search)
                ? pool
                : pool.Where(e => FilterBar.Matches(search, DisplayTitle(e))).ToList();

            // Off the whole scan, not the search subset, so the tag cannot flicker while typing.
            bool mixed = _libraryScan.Entries.Select(e => e.Shelf).Distinct().Count() > 1;
            MixedConsoles = mixed;
            LibraryList.Label = e => mixed ? $"{DisplayTitle(e)}   —   {e.Shelf}" : DisplayTitle(e);
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
            bool suspended = _session is not null;
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
            if (_session is null) return;
            if (IsPaused) ResumeEmulation(); else PauseEmulation();
        }

        // Held by the session's run control; the emulation thread parks in the runtime's wait - see `man pause` and EmuSen_Backend.md §14.13.
        public void PauseEmulation()
        {
            _paused = true;
            Send(s => s.Run(CrystalSessionAbi.RunPause));
            _playClock.Stop();
            if (_session is not null) StatusText.Text = "Paused";
            SyncMenuState();
        }

        public void ResumeEmulation()
        {
            _paused = false;
            _halted = false;
            Send(s => s.Run(CrystalSessionAbi.RunResume));
            if (_session is not null)
            {
                _playClock.Start();
                StatusText.Text = $"Running: {_currentDisplayName}";
            }
            SyncMenuState();
        }

        // The session paused, resumed or halted by anyone, the shell's run control among them: the window's mirror follows the session's run state as it is now, so an event of the window's own that arrives late undoes nothing.
        private void RunChanged(GameSession session, string? op, long address)
        {
            if (!ReferenceEquals(session, _session)) return;
            string? run;
            try
            {
                using System.Text.Json.JsonDocument info = System.Text.Json.JsonDocument.Parse(session.Crystal.Info());
                run = info.RootElement.GetProperty("run").GetString();
            }
            catch (CrystalException) { return; }
            bool paused = run == "paused", halted = run == "halted";
            if (paused != _paused || halted != _halted)
            {
                _paused = paused;
                _halted = halted;
                if (paused || halted) _playClock.Stop(); else _playClock.Start();
                StatusText.Text = halted ? $"[BREAKPOINT] Halted at ${address:X6} - type resume in the DianaSh console to go on"
                    : paused ? "Paused" : $"Running: {_currentDisplayName}";
            }
            SyncMenuState();
        }

        private void StartEmulationThread()
        {
            _running = true;
            _emuThread = new Thread(EmulationLoop) { IsBackground = true, Name = "EmuSen-Emulation" };
            _emuThread.Start();
        }

        // Synchronous by contract: callers replace _session right after - see §4.21.
        private void StopEmulationThread()
        {
            if (_emuThread is null) return;
            _running = false;
            _emuThread.Join(); // a paused thread waits in the runtime at most a moment between looks at _running
            _emuThread = null;
        }

        // Frames until the ports are read again after a setting changed; the frame thread's.
        private int _portsDue;

        // The events a frame thread acts on between frames: a load resets the rate control (C7), a setting can move the ports, the rest are the window's.
        private void DrainEvents(GameSession session)
        {
            if (_events is not { } events || !events.Wait(0)) return;
            using System.Text.Json.JsonDocument drained = System.Text.Json.JsonDocument.Parse(events.Drain());
            foreach (System.Text.Json.JsonElement e in drained.RootElement.EnumerateArray())
            {
                string? kind = e.GetProperty("kind").GetString(), op = e.TryGetProperty("op", out System.Text.Json.JsonElement o) ? o.GetString() : null;
                uint id = e.TryGetProperty("session", out System.Text.Json.JsonElement s) ? s.GetUInt32() : 0;
                // A console's setting is raised on no session; it is this one's where it names this session's console.
                bool ours = id == session.Id || (id == 0 && kind == "settings" && e.TryGetProperty("console", out System.Text.Json.JsonElement c) && string.Equals(c.GetString(), session.Console, StringComparison.OrdinalIgnoreCase));
                if (!ours) continue;
                switch (kind)
                {
                    case "state" when op == "loaded":
                        _audioPlayer.RateControl.Reset();
                        break;
                    case "run":
                        long address = e.TryGetProperty("address", out System.Text.Json.JsonElement a) && a.ValueKind == System.Text.Json.JsonValueKind.Number ? a.GetInt64() : 0;
                        Dispatcher.UIThread.Post(() => RunChanged(session, op, address));
                        break;
                    case "settings":
                        _portsDue = 2;
                        break;
                    case "disc":
                        Dispatcher.UIThread.Post(SyncMenuState);
                        break;
                }
            }
        }

        // The sound waiting, played or dropped as the speed says; a dropped stretch resets the rate control - see EmuSen_Audio_Sync.md §3.2.
        private short[] _sound = new short[8192];

        private void DrainSound(CrystalSession session, bool play)
        {
            int frames = session.AudioFramesReady;
            if (_sound.Length < frames * 2) _sound = new short[frames * 2];
            int got = session.DrainAudio(_sound, out uint rate);
            if (!play)
            {
                _audioPlayer.RateControl.Reset();
                return;
            }
            if (got > 0) _audioPlayer.Submit(_sound.AsSpan(0, got * 2).ToArray(), (int)rate);
        }

        // The picture to the frame control: the lent device's when the core drew there and the screen presents there, else copied into a buffer the hand-off owns - see EmuSen_DevicePicture.md §17.
        private void OfferPicture(GameSession session)
        {
            int width = _info.width, height = _info.height;
            if (width <= 0 || height <= 0) return;
            if (_info.picture_on_device != 0 && GameFrame.PresentsOnDevice && BorrowedLend(session) is { } lend)
            {
                _frames.OfferOnDevice(lend, width, height, (long)_info.total_frames, _info.display_aspect, _info.display_lines);
                return;
            }
            if (_frames.Rent(width * height * 4) is not { } pixels) return;
            session.Crystal.CopyPicture(pixels);
            _frames.Offer(pixels, width, height, Math.Max(1, _info.row_repeat), (long)_info.total_frames, _info.display_aspect, _info.display_lines, RawFor(session.Crystal));
        }

        // The machine's lend borrowed for the presenter, kept while the runtime hands the same one - see EmuSen_Backend.md §14.12.
        private (nint Handle, EmuSen.Serenity.Slang.SlangLend Lend)? _borrowed;

        private EmuSen.Serenity.Slang.SlangLend? BorrowedLend(GameSession session)
        {
            nint handle = session.Crystal.PictureLend();
            if (handle == 0 || _lentDevice is not { } device) return null;
            if (_borrowed is { } kept && kept.Handle == handle) return kept.Lend;
            var lend = EmuSen.Serenity.Slang.SlangLend.Borrow(device, handle);
            _borrowed = (handle, lend);
            return lend;
        }

        // The picture before the palette in the encoding the screen's filter reads, where it reads one - see EmuSen_CRT.md §22.
        private string? _rawSelected;

        private EmuSen.Serenity.RawPicture? RawFor(CrystalSession session)
        {
            string? wanted = GameFrame.RawEncoding;
            if (wanted != _rawSelected)
            {
                try { session.RawSelect(wanted); } catch (CrystalException) { wanted = null; }
                _rawSelected = wanted;
                return null;
            }
            if (wanted is null) return null;
            long bytes = session.RawCopy(Span<byte>.Empty, out _);
            if (bytes <= 0 || _frames.RentRaw((int)bytes) is not { } values) return null;
            session.RawCopy(values, out CrystalRawInfo shape);
            return new EmuSen.Serenity.RawPicture(wanted, shape.width, shape.height, (int)shape.element, shape.stride, values);
        }

        // Which way the picture reaches the screen, said once a session as its first picture is shown - see EmuSen_Backend.md §14.13.
        private void NamePicturePath(GameSession session)
        {
            string path = _info.picture_on_device != 0 && GameFrame.PresentsOnDevice ? "on the device: the core draws on the presenter's device and the chain reads it there"
                : !session.DeviceOffered ? "read back: the picture is not presented on the device, so no device was lent"
                : _info.picture_on_device == 0 ? "read back: the engine took no lent device"
                : "read back: the screen stopped presenting on the device";
            Console.WriteLine($"[picture] {session.Engine}: {path}");
        }

        // Runs off the UI thread; the frame-end order is the runtime's, the pacing against the display the window's - see EmuSen_Settings_Reference.md §4.21 and EmuSen_Backend.md §14.13.
        private void EmulationLoop()
        {
            if (_session is not { } session) return;
            CrystalSession crystal = session.Crystal;
            Stopwatch clock = Stopwatch.StartNew();
            TimeSpan nextTick = clock.Elapsed;

            // One frame per refresh when the display is close to the core's rate, the core's own rate otherwise - see EmuSen_Settings_Reference.md §4.87.
            var scheduler = new FrameScheduler(GameFrame.Display.Current, Stopwatch.GetTimestamp());
            if (double.TryParse(Environment.GetEnvironmentVariable("EMUSEN_SYNC_MARGIN_MS"), System.Globalization.NumberStyles.Float, System.Globalization.CultureInfo.InvariantCulture, out double marginMs)) scheduler.Margin = TimeSpan.FromMilliseconds(marginMs);

            // Counts completed frames, not presented ones - see §4.21.
            TimeSpan fpsWindowStart = clock.Elapsed;
            int framesInWindow = 0, offeredInWindow = 0;

            // The serial of the picture last offered; a core that says its picture is unchanged is not offered it again - see EmuSen_Multicore.md §14.
            ulong? offeredSerial = null;
            bool pictureNamed = false, turboApplied = false;

            // The frame call alone against the rest of this loop's per-frame work - see §4.21.
            TimeSpan runFrameTimeInWindow = TimeSpan.Zero;
            var frameStopwatch = new Stopwatch();
            long eventsTicks = 0, audioTicks = 0, handOffTicks = 0, restTicks = 0, loopMark = 0;

            // Whatever phases the core publishes, averaged like the frame's time - see EmuSen_Multicore.md §5.
            string[] phaseNames = PhaseNamesOf(crystal);
            var phases = new double[phaseNames.Length];
            var phaseMsInWindow = new double[phaseNames.Length];

            while (_running)
            {
                try
                {
                    // Held turbo wins over the menu's base speed - see EmuSen_Settings_Reference.md §4.13.
                    if (_turboHeld != turboApplied)
                    {
                        turboApplied = _turboHeld;
                        crystal.SpeedMode(turboApplied ? CrystalAbi.SpeedTurbo : CrystalAbi.SpeedNormal);
                    }
                    double frameRate = _info.frame_rate_hz > 0 ? _info.frame_rate_hz : session.FrameRateHz;
                    bool known = _info.size != 0;
                    TimeSpan interval = known ? TimeSpan.FromTicks((long)(_info.frame_interval_ns / 100)) : TimeSpan.FromSeconds(1.0 / frameRate);
                    scheduler.SyncToDisplay = GameFrame.SyncToDisplay;
                    nextTick = scheduler.Next(nextTick, frameRate, !known || _info.speed_percent == NormalPercent, interval);

                    // Takes over the frame entirely: one step back every interval/speedup ticks, so a sparse history does not rewind in a blur - see EmuSen_Rewind_And_FastForward.md §4.
                    if (_rewindHeld)
                    {
                        if (++_heldRewindTicks >= Math.Max(1, (int)Math.Round(_rewindInterval / HeldRewindSpeedup)))
                        {
                            _heldRewindTicks = 0;
                            if (crystal.RewindStep(1, ref _info) == 0) nextTick = clock.Elapsed;
                            else OfferPicture(session);
                        }
                        DrainSound(crystal, play: false);
                        DrainEvents(session);
                        _consoleWindow.Current?.DrainPendingFromEmulationThread();
                        offeredSerial = null;
                        SleepUntil(nextTick, clock);
                        continue;
                    }

                    long frameStart = Stopwatch.GetTimestamp();
                    if (loopMark != 0) restTicks += frameStart - loopMark;
                    frameStopwatch.Restart();
                    crystal.Frame(ref _info);

                    // Paused or halted: nothing ran, and the thread parks in the runtime until a change or run control wakes it (C1, C2).
                    if (_info.ran == 0)
                    {
                        DrainEvents(session);
                        _consoleWindow.Current?.DrainPendingFromEmulationThread();
                        crystal.Wait(20);
                        nextTick = clock.Elapsed;
                        loopMark = 0;
                        continue;
                    }
                    runFrameTimeInWindow += frameStopwatch.Elapsed;
                    long afterRun = Stopwatch.GetTimestamp();

                    // Runs what the console queued, between frames - see `man pause`.
                    _consoleWindow.Current?.DrainPendingFromEmulationThread();
                    DrainEvents(session);
                    // A setting can move the ports, as an adapter set on one does; machine info says so once a frame has run under it.
                    if (_portsDue > 0 && --_portsDue == 0) session.RefreshPorts();
                    long afterEvents = Stopwatch.GetTimestamp();
                    eventsTicks += afterEvents - afterRun;

                    // Drained every frame either way, so a muted stretch cannot back the buffer up - see EmuSen_Audio_Sync.md §4.
                    _audioPlayer.RateControl.NominalRatio = scheduler.Decision.AudioRatio; // the locked rate's drift, centred - see EmuSen_Audio_Sync.md §3.4
                    if (PresentationTrace.Enabled) PresentationTrace.Audio(Stopwatch.GetTimestamp(), _audioPlayer.QueuedFrames, _audioPlayer.RateControl.LastRatio, _audioPlayer.RateControl.NominalRatio, _audioPlayer.RateControl.SheddingEvents);
                    DrainSound(crystal, play: _info.play_audio != 0);
                    long afterAudio = Stopwatch.GetTimestamp();
                    audioTicks += afterAudio - afterEvents;

                    if (phases.Length > 0)
                    {
                        int count = crystal.Phases(phases);
                        for (int p = 0; p < phaseMsInWindow.Length && p < count; p++) phaseMsInWindow[p] += phases[p];
                    }

                    // A picture for a later refresh is not handed over a refresh early - see EmuSen_Settings_Reference.md §4.87.6.
                    long held = 0;
                    if (scheduler.HandOverNotBefore is { } notBefore && clock.Elapsed < notBefore)
                    {
                        long holdStart = Stopwatch.GetTimestamp();
                        SleepUntil(notBefore, clock);
                        held = Stopwatch.GetTimestamp() - holdStart;
                    }

                    // Nothing new was drawn on a frame the speed left undrawn, nor when the core says its picture is the last one.
                    if (_info.rendered != 0 && _info.picture_serial != offeredSerial)
                    {
                        OfferPicture(session);
                        if (!pictureNamed)
                        {
                            pictureNamed = true;
                            NamePicturePath(session);
                        }
                        offeredSerial = _info.picture_serial;
                        offeredInWindow++;
                        PresentationTrace.Frame((long)_info.total_frames, frameStart, Stopwatch.GetTimestamp(), true);
                    }
                    else
                    {
                        PresentationTrace.Frame((long)_info.total_frames, frameStart, Stopwatch.GetTimestamp(), false);
                    }

                    loopMark = Stopwatch.GetTimestamp();
                    handOffTicks += loopMark - afterAudio;
                    scheduler.Completed(TimeSpan.FromTicks((loopMark - frameStart - held) * TimeSpan.TicksPerSecond / Stopwatch.Frequency));

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
                        double Ms(long ticks) => ticks * 1000.0 / Stopwatch.Frequency / framesInWindow;
                        string outside = $" | outside: events {Ms(eventsTicks):F2} audio {Ms(audioTicks):F2} hand-off {Ms(handOffTicks):F2} sleep+rest {Ms(restTicks):F2}ms";
                        string counters = $"{fps:F1} fps (run {runFrameMs:F2}ms / total {totalMs:F2}ms)";
                        string line = $"{counters}{breakdown}{presentation}{outside} | {scheduler.Decision.Describe()}";
                        PresentationTrace.Note($"{scheduler.Decision.Describe()}; render timer {RenderTimerRate.Current} fps");
                        Console.WriteLine($"[fps] {line}");

                        // The bar shows the frame rate, the frame's cost and its top phases; the whole line is its tooltip (§4.83.6).
                        string barLine = top.Length == 0 ? counters : $"{counters} [{top}]";
                        if (_appSettings.ShowFpsBar) Dispatcher.UIThread.Post(() => ShowCounters(barLine, line));
                        framesInWindow = 0;
                        offeredInWindow = 0;
                        Array.Clear(phaseMsInWindow);
                        runFrameTimeInWindow = TimeSpan.Zero;
                        eventsTicks = audioTicks = handOffTicks = restTicks = 0;
                        fpsWindowStart = clock.Elapsed;
                    }
                }
                catch (Exception ex)
                {
                    _running = false;
                    string message = ex.Message;
                    string settings = string.Join(", ", RuntimeCatalog.SettingsFor(_activeConsole).Select(k => $"{k.Key}={SettingOrDefault(_activeConsole, k)}"));
                    string? report = CrashLog.Write("emulation halt", ex, $"{_activeConsole}, frame {_info.total_frames}: {settings}");
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
                    TimeSpan step = _info.frame_interval_ns > 0 ? TimeSpan.FromTicks((long)(_info.frame_interval_ns / 100)) : TimeSpan.Zero;
                    nextTick = FramePacer.Settle(nextTick, clock.Elapsed, scheduler.Interval(step));
                }
            }
        }

        private static string[] PhaseNamesOf(CrystalSession session)
        {
            try
            {
                using System.Text.Json.JsonDocument names = System.Text.Json.JsonDocument.Parse(session.PhaseNames());
                return names.RootElement.EnumerateArray().Select(n => n.GetString() ?? "").ToArray();
            }
            catch (CrystalException) { return Array.Empty<string>(); }
        }

        private static string SettingOrDefault(string console, SettingRow row)
        {
            try { return Runtime.Setting(console, row.Key) ?? row.Default; }
            catch (CrystalException) { return row.Default; }
        }

        // Spins the whole interval; the hybrid was measured and dropped - see §4.21.
        private static void SleepUntil(TimeSpan target, Stopwatch clock)
        {
            while (clock.Elapsed < target)
            {
                Thread.SpinWait(100);
            }
        }

        // What each session is opened with: audio.json's limit, and for a test no battery - see EmuSen_Multicore.md §6.
        internal CrystalOpen OpenOptions { get; set; } = new();

        private void OnExitClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }

        // Redirected per ROM, and unconditionally - see EmuSen_Settings_Reference.md §4.22.
        private void StartLogging(string coreName)
        {
            StopLogging(); // close the previous session's files first - see CategorizedLogWriter.Dispose's own comment

            string logRoot = ErrorLog.Usable(_appSettings.LogDirectory) ? _appSettings.LogDirectory! : ErrorLog.DefaultRoot;

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
                ErrorLog.Error("logging", "Per-game logging disabled", ex, logRoot);
                StatusText.Text = $"Logging disabled: {ex.Message}";
            }
        }

        private void StopLogging()
        {
            if (_activeLogWriter is null) return;

            Console.SetOut(_originalConsoleOut);
            _activeLogWriter.Dispose();
            _activeLogWriter = null;
        }
    }
}
