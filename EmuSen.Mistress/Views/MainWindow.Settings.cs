using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Styling;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
using EmuSen.Endymion;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Scraping;

namespace EmuSen.Mistress.Views
{
    // Settings over the whole screen on the big screen, every door to it, and what it reads and asks of the window - see EmuSen_BigPicture.md §52.
    public partial class MainWindow : ISettingsHost
    {
        private SettingsView? _settings;
        private SettingsWindow? _settingsWindow;
        private bool _settingsPaused;
        private double _settingsScale = 1;

        public SettingsScreen? SettingsShown => _settings?.Model;

        public SettingsLayout? SettingsLayoutShown => _settings?.Layout;

        // Every door: Home's place, the card's Settings, the pad menu's and the desktop's menu open Settings at a section, and a page under it where asked.
        public void ShowSettings(string section = SettingsScreen.Controllers, params string[] path)
        {
            // Over a game the Pause Card stays beneath what it opened (EmuSen_Interface.md §18.5); over the library the card gives way.
            if (_padMenuOpen && !_padMenuInGame) ClosePadMenu();
            CloseSettings(restoreFocus: false);
            // A game under Settings stays paused until it closes, as under the card it was opened from.
            _settingsPaused = GameOnScreen && !IsPaused;
            if (_settingsPaused) PauseEmulation();
            var layout = new SettingsLayout { Name = "SettingsScreen", Form = _bigScreen ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop, PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily };
            // Keys reach the bindings pages before anything else in the window that holds Settings, whatever has the focus.
            if (!_settingsKeysWired)
            {
                AddHandler(KeyDownEvent, OnSettingsKeyFirst, Avalonia.Interactivity.RoutingStrategies.Tunnel);
                AddHandler(KeyUpEvent, OnSettingsKeyUp, Avalonia.Interactivity.RoutingStrategies.Tunnel);
                _settingsKeysWired = true;
            }
            _settingsScale = EmuSen.LunaP.Theme.LunaTokens.TextScale;
            var model = new SettingsScreen(this, section);
            if (path.Length > 0) model.OpenPath(section, path);
            _settings = new SettingsView(model, layout, SettingsFrame);
            _gamepad.Polled -= OnSettingsPadsPolled;
            _gamepad.Polled += OnSettingsPadsPolled;
            if (!_bigScreen)
            {
                // The desktop's one Settings window, 960 by 640, a sheet over the library (EmuSen_Interface.md §17.7).
                var window = _settingsWindow = new SettingsWindow(layout, button => { SettingsPadCommand(button); return true; });
                window.AddHandler(KeyDownEvent, OnSettingsKeyFirst, Avalonia.Interactivity.RoutingStrategies.Tunnel);
                window.AddHandler(KeyUpEvent, OnSettingsKeyUp, Avalonia.Interactivity.RoutingStrategies.Tunnel);
                window.Closed += (_, _) => { if (_settingsWindow == window) { _settingsWindow = null; SettingsClosed(); } };
                _ = SheetLayer.Show(window, this);
                return;
            }
            layout.KeyDown += OnSettingsKey;
            ScreenContent.Children.Add(layout);
            StatusBar.IsVisible = false;
            layout.Focus();
        }

        public void CloseSettings(bool restoreFocus = true)
        {
            if (_settingsWindow is { } window)
            {
                _settingsWindow = null;
                window.Close();
                SettingsClosed();
                return;
            }
            if (_settings is not { } shown) return;
            shown.Layout.KeyDown -= OnSettingsKey;
            ScreenContent.Children.Remove(shown.Layout);
            SettingsClosed();
            ApplyStatusBar();
            if (!restoreFocus) return;
            if (OwnScreenShown) ShowOwnLook(false);
            if (HomeLayout is { } home) home.Focus();
            else if (ThemedLibraryShown) ThemedLibraryHost.Focus();
        }

        private void SettingsClosed()
        {
            // A text size changed in Settings reaches the screen under it as it closes: Home or the grid, a theme's view, or the card (EmuSen_BigPicture.md §52.8).
            if (_settingsScale != EmuSen.LunaP.Theme.LunaTokens.TextScale)
            {
                if (OwnScreenShown) ShowOwnLook(false);
                else if (LibraryView.IsVisible && ThemedLibraryShown) ShowThemedLibrary();
                if (_padMenuOpen) RefreshPadMenuText(PadMenuList.SelectedIndex);
            }
            _settings = null;
            _gamepad.Polled -= OnSettingsPadsPolled;
            if (_settingsPaused && GameOnScreen && IsPaused) ResumeEmulation();
            _settingsPaused = false;
        }

        // The pad on Settings: the directions move and step, A chooses, B closes a page and then Settings, Start goes Home; the shoulders and the rest as the layout declares.
        private void SettingsPadCommand(UiButton button)
        {
            if (_settings is not { } view) return;
            // While a binding listens, and until the button it heard is let go, the pad is the binding's, not the screen's.
            if (_bindHeld || view.Model.Binding is { Testing: true }) return;
            // A key cannot come from the pad: while a key is listened for, B backs out and nothing else does.
            if (view.Model.Binding is { Keys: true, Capturing: true } || view.Model.HotkeyListening is not null)
            {
                if (button != UiButton.Back) return;
                view.Model.CancelCapture();
                view.Model.CancelHotkey();
                view.Fill();
                return;
            }
            if (view.Model.Binding is { } binding && button is UiButton.PageUp or UiButton.PageDown)
            {
                view.Model.StepBindingConsole(button == UiButton.PageUp ? -1 : 1);
                view.Fill();
                return;
            }
            switch (button)
            {
                case UiButton.Up: view.Layout.Move(0, -1); return;
                case UiButton.Down: view.Layout.Move(0, 1); return;
                case UiButton.Left: view.Layout.Move(-1, 0); return;
                case UiButton.Right: view.Layout.Move(1, 0); return;
                case UiButton.Back:
                    if (!view.Back()) CloseSettings();
                    return;
                case UiButton.Menu:
                    CloseSettings(restoreFocus: false);
                    if (OwnScreenShown || !ThemedLibraryShown) ShowHome();
                    else OpenPadMenu();
                    return;
            }
            if (ToolPaneWindow.Glyph(button) is not { } glyph || view.Layout.Press(glyph)) return;
            if (button is UiButton.PageUp or UiButton.PageDown && view.Model.StepConsole(button == UiButton.PageUp ? -1 : 1)) view.Fill();
        }

        // Keys before the screen sees them, on both forms: a binding or a hotkey listening takes the key, the tester sees what is held and Escape stops it.
        private void OnSettingsKeyFirst(object? sender, KeyEventArgs e)
        {
            if (_settings is not { } view) return;
            _keysHeld.Add(e.Key);
            SettingsScreen model = view.Model;
            if (model.Binding is { Keys: false, Capturing: true })
            {
                if (e.Key == Key.Escape) EndBindCapture();
                e.Handled = true;
                return;
            }
            if (model.RebindKey(e.Key) || model.RebindHotkey(e.Key))
            {
                e.Handled = true;
                view.Fill();
                return;
            }
            if (model.Binding is { Testing: true })
            {
                if (e.Key == Key.Escape) model.StopTesting();
                e.Handled = true;
                LightBindings(view, force: true);
            }
        }

        private void OnSettingsKeyUp(object? sender, KeyEventArgs e)
        {
            if (!_keysHeld.Remove(e.Key) || _settings is not { } view) return;
            if (view.Model.Binding is not null) LightBindings(view);
            if (view.Model.Binding is { Testing: true }) e.Handled = true;
        }

        private readonly HashSet<Key> _keysHeld = new();
        private bool _settingsKeysWired;

        private void OnSettingsKey(object? sender, KeyEventArgs e)
        {
            if (e.Handled) return;
            if (KeyAsPad(e.Key) is not { } b) return;
            SettingsPadCommand(b);
            e.Handled = true;
        }

        // A change made under a door Settings opened shows when it closes.
        private void RefreshSettings()
        {
            if (_settings is not { } view) return;
            view.Model.Read();
            view.Fill();
        }

        // The game the previews show: the one running, else the one played last.
        private string? LastPlayed => ((IHomeHost)this).Recent.FirstOrDefault();

        private IImage? SettingsFrame()
        {
            if (PausedFrame() is { } frame) return frame;
            if (LastPlayed is not { } path) return null;
            GameArt art = ((IGamePageHost)this).Art(path);
            return Load(art.Screenshot) ?? Load(art.Hero) ?? Load(art.Cover);
        }

        // --- ISettingsHost: what Settings reads ---

        AppSettings ISettingsHost.App => _appSettings;

        GraphicsConfig ISettingsHost.Graphics => _graphics;

        bool ISettingsHost.Desktop => !_bigScreen;

        string ISettingsHost.Version => BuildName.Split('+')[0];

        string? ISettingsHost.RunningTitle => _session is not null ? RunningGameTitle() : LastPlayed is { } path ? ((IGamePageHost)this).Facts(path).Title : null;

        string? ISettingsHost.RunningConsole => _session is null ? null : _activeConsole;

        string? ISettingsHost.RunningCropNote(string console) => RunningGameCropNote(console);

        private ShaderShelf? _shaderShelf;

        ShaderShelf ISettingsHost.Shaders => _shaderShelf ??= new ShaderShelf();

        (bool Running, int Waiting) ISettingsHost.ScrapeRun => (((IScrapeHost)this).ScrapeRunning, ((IScrapeHost)this).Interrupted);

        void ISettingsHost.DownloadShaders() => _ = ((ISettingsHost)this).Shaders.Download(HttpFactory, RefreshSettings);

        IReadOnlyList<SystemFacts> ISettingsHost.Systems => RuntimeCatalog.Systems;

        IReadOnlyList<SettingRow> ISettingsHost.Schema(string console)
        {
            // The running game's own rows for its console, as its engine declares them, the console's schema for every other - see EmuSen_Backend.md §18.10.11.
            try { return _session is { } running && console == _activeConsole ? RuntimeCatalog.SettingsOf(running.Crystal) : RuntimeCatalog.SettingsFor(console); }
            catch (CrystalException) { return []; }
        }

        string? ISettingsHost.CoreValue(string console, string key) => CoreSettings.Value(console, key);

        void ISettingsHost.SetCoreValue(string console, string key, string? value)
        {
            try
            {
                CoreSettings.Set(console, key, value);
                if (key == RuntimeCatalog.EngineKey) RuntimeCatalog.Forget();
            }
            catch (CrystalException e) { StatusText.Text = $"The setting was not changed: {e.Message}"; }
        }

        // The runtime's firmware_overview by file: its size, where it was found and what its open version costs; empty where the runtime cannot say.
        IReadOnlyDictionary<string, FirmwareWords> ISettingsHost.FirmwareOverview()
        {
            var words = new Dictionary<string, FirmwareWords>(StringComparer.Ordinal);
            try
            {
                using var overview = System.Text.Json.JsonDocument.Parse(Runtime.FirmwareOverview());
                static string? Text(System.Text.Json.JsonElement e, string name) => e.TryGetProperty(name, out var v) && v.ValueKind == System.Text.Json.JsonValueKind.String ? v.GetString() : null;
                foreach (var system in overview.RootElement.GetProperty("systems").EnumerateArray())
                    foreach (var i in system.GetProperty("items").EnumerateArray())
                        words.TryAdd(Text(i, "file") ?? "", new FirmwareWords(Text(i, "file") ?? "", i.GetProperty("size").GetInt32(), i.GetProperty("wrong_size").GetBoolean(), Text(i, "effect"), Text(i, "cost"), Text(i, "found_in"), Text(i, "wrong_size_in")));
            }
            catch (Exception e) when (e is CrystalException or System.Text.Json.JsonException or InvalidOperationException or KeyNotFoundException) { }
            return words;
        }

        // The runtime's firmware_status for a system, null where it cannot say (EmuSen_Cores.md §3.11).
        IReadOnlyList<FirmwareFile>? ISettingsHost.Firmware(string system)
        {
            try
            {
                using var answer = System.Text.Json.JsonDocument.Parse(Runtime.FirmwareStatus(system));
                static string Text(System.Text.Json.JsonElement e, string name) => e.TryGetProperty(name, out var v) && v.ValueKind == System.Text.Json.JsonValueKind.String ? v.GetString() ?? "" : "";
                return answer.RootElement.GetProperty("firmware").EnumerateArray()
                    .Select(f => new FirmwareFile(Text(f, "label"), Text(f, "file"), f.TryGetProperty("required", out var r) && r.ValueKind == System.Text.Json.JsonValueKind.True, Text(f, "status"),
                        Text(f, "need") is { Length: > 0 } need ? need : null)).ToList();
            }
            catch (Exception e) when (e is CrystalException or System.Text.Json.JsonException or InvalidOperationException or KeyNotFoundException) { return null; }
        }

        IReadOnlyList<CardMedium> ISettingsHost.Cards(string console)
        {
            try { return CardMedium.Of(Runtime, console); }
            catch (Exception e) when (e is CrystalException or System.Text.Json.JsonException or KeyNotFoundException) { throw new InvalidOperationException(e.Message, e); }
        }

        string ISettingsHost.NewCard(string console)
        {
            CardMedium? medium = ((ISettingsHost)this).Cards(console).FirstOrDefault();
            if (medium is null) return "";
            CardOutcome made = CardOutcome.Of(Runtime, CrystalAbi.CardsNew, new { path = medium.PathOf(medium.NextName) });
            StatusText.Text = made.Words;
            RefreshSettings();
            return made.Words;
        }

        int ISettingsHost.Games(string console) => _allScan.Entries.Count(e => Listed(e) && RuntimeCatalog.ByDisplayName(e.CoreDisplayName)?.Console == console);

        int ISettingsHost.AllGames => _allScan.Entries.Count(Listed);

        int ISettingsHost.Hidden => _allScan.Entries.Count(e => IsHiddenGame(e.FullPath));

        IReadOnlyList<GameCollection> ISettingsHost.CustomCollections => _collections;

        IReadOnlyList<PlayerSeat> ISettingsHost.Players =>
            Enumerable.Range(1, 4).Select(p => new PlayerSeat(p, _gamepad.Players.SeatOf(p)?.Name, _gamepad.Players.SeatOf(p) is { } pad && !_gamepad.Pads.Contains(pad))).ToList();

        // Two pads of one name numbered in front, from the order Endymion lists them in, so a long name trimmed at its end keeps the number.
        IReadOnlyList<string> ISettingsHost.Pads => _gamepad.Pads.Select(p => _gamepad.Pads.Where(o => o.Name == p.Name).ToList() is { Count: > 1 } same ? $"{same.IndexOf(p) + 1} · {p.Name}" : p.Name).ToList();

        void ISettingsHost.ForgetSeat(int player) => _gamepad.Players.Forget(player);

        int ISettingsHost.PlayerOf(int pad) => pad >= 0 && pad < _gamepad.Pads.Count ? _gamepad.Players.PlayerOf(_gamepad.Pads[pad]) : 0;

        // The big picture's looks as Preferences named them: EmuSen's own first, a theme of the same name told apart by its folder.
        private List<EmuSen.Mistress.BigPicture.BigPictureLook> LookList => EmuSen.Mistress.BigPicture.BigPictureLooks.All(_appSettings).ToList();

        IReadOnlyList<string> ISettingsHost.Looks
        {
            get
            {
                List<EmuSen.Mistress.BigPicture.BigPictureLook> looks = LookList;
                return looks.Select(l => l.BuiltIn ? $"{l.Name} (built in)" : looks.Count(o => o.Name == l.Name) > 1 ? $"{l.Name} ({System.IO.Path.GetFileName(l.Theme!.Directory)})" : l.Name).ToList();
            }
        }

        int ISettingsHost.LookAt => Math.Max(0, LookList.FindIndex(l => EmuSen.Mistress.BigPicture.BigPictureLooks.IsCurrent(_appSettings, l)));

        void ISettingsHost.ChooseLook(int at)
        {
            List<EmuSen.Mistress.BigPicture.BigPictureLook> looks = LookList;
            if (at < 0 || at >= looks.Count) return;
            EmuSen.Mistress.BigPicture.BigPictureLooks.Choose(_appSettings, looks[at]);
            ThemeChanged(false);
        }

        bool ISettingsHost.PadPressed(int pad) => pad >= 0 && pad < _gamepad.Pads.Count && _gamepad.Pads[pad].AnyPressed();

        private int _padsHeld;

        // A button held on a pad marks its row on the Players page, and a pad plugged in or pulled out adds or takes its row, drawn again only when either changes.
        private void OnSettingsPadsPolled()
        {
            if (_settings is { } bound && (bound.Model.Binding is { Keys: false, Capturing: true } || _bindHeld))
            {
                PollBindCapture(bound);
                return;
            }
            if (_settings is { Model.Binding: not null } shown)
            {
                PollTester(shown);
                return;
            }
            if (_settings is not { } view || view.Model.Trail.LastOrDefault() != "Players") return;
            int held = 0;
            for (int i = 0; i < _gamepad.Pads.Count && i < 30; i++) if (_gamepad.Pads[i].AnyPressed()) held |= 1 << i;
            if (held == _padsHeld && _gamepad.Pads.Count == _padsShown) return;
            _padsHeld = held;
            _padsShown = _gamepad.Pads.Count;
            RefreshSettings();
        }

        // --- The bindings page's capture: the press that chose it let go first, then the first button pressed, or a trigger pulled, within the timeout (EmuSen_Input.md §10.5). ---

        private bool _bindHeld, _bindArmed;
        private readonly System.Diagnostics.Stopwatch _bindListening = new();

        // How long a binding listens before it gives up, since every pad button that could cancel it is one it could bind.
        public TimeSpan BindCaptureTimeout { get; set; } = TimeSpan.FromSeconds(5);

        private void PollBindCapture(SettingsView view)
        {
            SDL3.SDL.GamepadButton? pressed = _gamepad.GetAnyPressedButton();
            if (view.Model.Binding is not { Capturing: true } binding)
            {
                if (pressed is null) _bindHeld = false;
                return;
            }
            if (!_bindHeld)
            {
                _bindHeld = true;
                _bindArmed = false;
                _bindListening.Restart();
            }
            if (!_bindArmed)
            {
                _bindArmed = pressed is null;
                return;
            }
            if (pressed is null && binding.Focus is EmuSen.Galaxia.Input.PadControl.L2 or EmuSen.Galaxia.Input.PadControl.R2
                && _gamepad.RawAxis(binding.Focus == EmuSen.Galaxia.Input.PadControl.L2 ? SDL3.SDL.GamepadAxis.LeftTrigger : SDL3.SDL.GamepadAxis.RightTrigger) >= 0.5)
                view.Model.RebindTrigger();
            else if (pressed is { } pad) view.Model.Rebind(pad);
            else if (_bindListening.Elapsed >= BindCaptureTimeout) view.Model.CancelCapture();
            else return;
            view.Fill();
        }

        private static readonly System.Diagnostics.Stopwatch Uptime = System.Diagnostics.Stopwatch.StartNew();

        // How long B is held to leave the tester, since a short press of B is a button being tried; the clock a test sets.
        public static readonly TimeSpan TestExitHold = TimeSpan.FromSeconds(1);
        public Func<TimeSpan> TesterClock { get; set; } = () => Uptime.Elapsed;
        private TimeSpan? _backHeldSince;

        // The tester: B held for a second stops it; what is held lights the drawing, drawn again only when it changes.
        private void PollTester(SettingsView view)
        {
            if (view.Model.Binding is { Testing: true } && _gamepad.Primary is { } pad && pad.IsRawPressed(_appSettings.SwapPadButtons ? SDL3.SDL.GamepadButton.South : SDL3.SDL.GamepadButton.East))
            {
                _backHeldSince ??= TesterClock();
                if (TesterClock() - _backHeldSince.Value >= TestExitHold)
                {
                    view.Model.StopTesting();
                    _backHeldSince = null;
                    LightBindings(view, force: true);
                    return;
                }
            }
            else _backHeldSince = null;
            LightBindings(view);
        }

        private void LightBindings(SettingsView view, bool force = false)
        {
            if (view.Model.Binding is not { } page) return;
            List<EmuSen.Galaxia.Input.PadControl> held = page.Capturing ? [] : view.Model.ControlsOf(page.Console).Where(c => BindingHeld(page, c)).ToList();
            if (view.Model.SetHeld(held) || force) view.Fill();
            view.SetSticks(axis => BindingAxis(page, axis));
        }

        // An axis as the game would read it: the player's stick, pushed all the way by a key held for its direction.
        private double BindingAxis(BindingsPage page, EmuSen.Galaxia.Input.PadAxis axis)
        {
            double raw = !page.Keys && (_gamepad.Players.PadFor(page.Player) ?? (page.Player == 1 ? _gamepad.Primary : null)) is { } pad ? pad.RawAxis(axis switch
            {
                EmuSen.Galaxia.Input.PadAxis.LeftX => SDL3.SDL.GamepadAxis.LeftX, EmuSen.Galaxia.Input.PadAxis.LeftY => SDL3.SDL.GamepadAxis.LeftY,
                EmuSen.Galaxia.Input.PadAxis.RightX => SDL3.SDL.GamepadAxis.RightX, _ => SDL3.SDL.GamepadAxis.RightY,
            }) : 0;
            return EmuSen.Galaxia.Input.PadControls.Resolve(axis, raw, c => _keyBindings.For(page.Console).ButtonToKey.TryGetValue(c, out Key k) && _keysHeld.Contains(k));
        }

        // A control as the game would hear it: its key held, or the player's pad pressing its button, its trigger or its stick's direction (EmuSen_Input.md §7.3).
        private bool BindingHeld(BindingsPage page, EmuSen.Galaxia.Input.PadControl control)
        {
            if (_keyBindings.For(page.Console).ButtonToKey.TryGetValue(control, out Key key) && _keysHeld.Contains(key)) return true;
            if (page.Keys || (_gamepad.Players.PadFor(page.Player) ?? (page.Player == 1 ? _gamepad.Primary : null)) is not { } pad) return false;
            if (!EmuSen.Galaxia.Input.PadControls.IsButton(control, out EmuSen.Galaxia.Input.PadButton button))
            {
                bool left = control is EmuSen.Galaxia.Input.PadControl.LeftStickUp or EmuSen.Galaxia.Input.PadControl.LeftStickDown or EmuSen.Galaxia.Input.PadControl.LeftStickLeft or EmuSen.Galaxia.Input.PadControl.LeftStickRight;
                bool vertical = control is EmuSen.Galaxia.Input.PadControl.LeftStickUp or EmuSen.Galaxia.Input.PadControl.LeftStickDown or EmuSen.Galaxia.Input.PadControl.RightStickUp or EmuSen.Galaxia.Input.PadControl.RightStickDown;
                bool negative = control is EmuSen.Galaxia.Input.PadControl.LeftStickUp or EmuSen.Galaxia.Input.PadControl.LeftStickLeft or EmuSen.Galaxia.Input.PadControl.RightStickUp or EmuSen.Galaxia.Input.PadControl.RightStickLeft;
                double value = pad.RawAxis(left ? (vertical ? SDL3.SDL.GamepadAxis.LeftY : SDL3.SDL.GamepadAxis.LeftX) : (vertical ? SDL3.SDL.GamepadAxis.RightY : SDL3.SDL.GamepadAxis.RightX));
                return negative ? value <= -ControllerDiagrams.DirectionThreshold : value >= ControllerDiagrams.DirectionThreshold;
            }
            if (button is EmuSen.Galaxia.Input.PadButton.L2 or EmuSen.Galaxia.Input.PadButton.R2 && pad.RawAxis(button == EmuSen.Galaxia.Input.PadButton.L2 ? SDL3.SDL.GamepadAxis.LeftTrigger : SDL3.SDL.GamepadAxis.RightTrigger) >= 0.5) return true;
            return _gamepadBindings.For(page.Console, page.Player, page.Profile).ButtonToPad.TryGetValue(button, out SDL3.SDL.GamepadButton bound) && pad.IsRawPressed(bound);
        }

        private void EndBindCapture()
        {
            if (_settings is not { } view) return;
            view.Model.CancelCapture();
            _bindHeld = false;
            view.Fill();
        }

        private int _padsShown;

        void ISettingsHost.Seat(int pad, int player)
        {
            if (pad >= 0 && pad < _gamepad.Pads.Count) _gamepad.Assign(_gamepad.Pads[pad], player);
        }

        bool ISettingsHost.Logging
        {
            get { try { return Runtime.LogSwitch; } catch (CrystalException) { return false; } }
            set { try { Runtime.LogSwitch = value; } catch (CrystalException e) { StatusText.Text = $"Logging was not changed: {e.Message}"; } }
        }

        // Saved at once, and what the change touches applied now: the picture, the bar, the sounds, the library.
        void ISettingsHost.SaveApp(string? what)
        {
            _appSettings.Save();
            switch (what)
            {
                case "variant": ApplyAppearance(); break;
                case "textsize" or "motion": ApplyAccessibility(); RefreshSettings(); break;
                case "statusbar" or "messages" or "fps": ApplyStatusBar(); break;
                case "volume": SetVolume(_appSettings.Volume); break;
                case "deadzone": _gamepad.StickDeadzone = _appSettings.StickDeadzone; break;
                case "stickdpad": _gamepad.AnalogStickAsDpad = _appSettings.AnalogStickAsDpad; break;
                case "mirror": ApplyKeys(); break;
                case "output":
                    _audioPlayer.SetOutputDevice(_appSettings.SoundOutputDevice);
                    _uiSounds?.SetOutputDevice(_appSettings.SoundOutputDevice);
                    break;
                case "rumble":
                    try { _session?.Crystal.RumbleSwitch(_appSettings.Rumble); }
                    catch (CrystalException e) { StatusText.Text = $"Rumble was not changed in the game: {e.Message}"; }
                    break;
                case "hidden": RefreshLibrary(); break;
                case "screenscraper" or "openemu": ApplyScraping(); ApplyOnlineCovers(); break;
                default:
                    if (what is not null && (what.StartsWith("show", StringComparison.Ordinal) || what.StartsWith("auto", StringComparison.Ordinal) || what.StartsWith("flatten", StringComparison.Ordinal)
                        || what is "collections" or "favfirst" or "favfirstcustom" or "stars" or "folderstop") && LibraryView.IsVisible && ThemedLibraryShown) ShowThemedLibrary();
                    break;
            }
        }

        void ISettingsHost.SaveGraphics(string? console)
        {
            _graphics.Save();
            GameFrame.SyncToDisplay = _graphics.SyncToDisplay;
            if (_session is not null && (console is null || console == _activeConsole)) ApplyScreenFilter(_activeConsole);
        }

        void ISettingsHost.Open(string door)
        {
            switch (door)
            {
                case SettingsScreen.DoorCheats: ShowActiveCheats(); break;
                case SettingsScreen.DoorThemes: ShowThemeSettings(); break;
                case SettingsScreen.DoorScraper: ShowScraper(); break;
                case SettingsScreen.DoorSignIn: _ = SignInFromSettingsAsync(); break;
                case SettingsScreen.DoorCheckAccount: _ = SaidInSettings(((IScrapeHost)this).CheckMemberAsync(), check: true); break;
                case SettingsScreen.DoorSignOut: StatusText.Text = ((IScrapeHost)this).SignOut(); RefreshSettings(); break;
                case SettingsScreen.DoorOpenVgdb: _ = OpenVgdbFromSettingsAsync(); break;
                case SettingsScreen.DoorCleanUp: _ = CleanUpFromSettingsAsync(); break;
                case SettingsScreen.DoorScrapeResume: ((IScrapeHost)this).ResumeScrape(); RefreshSettings(); break;
                case SettingsScreen.DoorScrapeCancel: _ = ThenRefreshSettings(((IScrapeHost)this).ConfirmCancelAsync((Avalonia.Controls.Window?)_settingsWindow ?? this)); break;
                case SettingsScreen.DoorScrapeStatus: ((IScrapeHost)this).ShowScrapeStatus(); break;
                case SettingsScreen.DoorCollections: ShowCollections(); break;
                case SettingsScreen.DoorShell: ShowShellConsole(); break;
                case SettingsScreen.DoorDiscs: ShowCompressedDiscs(); break;
                case SettingsScreen.DoorNewCollection: _ = NewCollectionAsync(); break;
                case SettingsScreen.DoorFinishCollection: FinishEditingCollection(); RefreshSettings(); break;
                case var d when d.StartsWith(SettingsScreen.DoorDeleteCollection, StringComparison.Ordinal) && long.TryParse(d[SettingsScreen.DoorDeleteCollection.Length..], out long id) && _collections.FirstOrDefault(c => c.Id == id) is { } doomed:
                    _ = ThenRefreshSettings(DeleteCustomCollectionAsync(this, doomed)); break;
                case var d when d.StartsWith(SettingsScreen.DoorCardSaves, StringComparison.Ordinal):
                    _ = SheetLayer.Show(new MemoryCardsWindow(_graphics, d[SettingsScreen.DoorCardSaves.Length..]), this); break;
                case SettingsScreen.DoorRomFolder: _ = AskFolderAsync("ROM folder", _appSettings.RomDirectory, p => { _appSettings.RomDirectory = p; RefreshLibrary(); }); break;
                case SettingsScreen.DoorArtFolder: _ = AskFolderAsync("Artwork folder", _appSettings.ArtworkDirectory, p => { _appSettings.ArtworkDirectory = p; ScanArtwork(); }); break;
                case SettingsScreen.DoorStateFolder: _ = AskFolderAsync("State folder", _appSettings.StateDirectory, p => _appSettings.StateDirectory = p); break;
                case SettingsScreen.DoorLogFolder: _ = AskFolderAsync("Log folder", _appSettings.LogDirectory, p => _appSettings.LogDirectory = p); break;
                case SettingsScreen.DoorOpenFirmware: OpenFirmwareFolder(); break;
                case SettingsScreen.DoorClearFirmware: _appSettings.FirmwareDirectory = null; _appSettings.Save(); RefreshSettings(); break;
                case SettingsScreen.DoorFirmwareFolder: _ = AskFolderAsync("Firmware folder", _appSettings.FirmwareDirectory, p => _appSettings.FirmwareDirectory = p); break;
                case SettingsScreen.DoorMediaFolder: _ = AskFolderAsync("ES-DE media folder", _appSettings.EsdeMediaDirectory, p => _appSettings.EsdeMediaDirectory = p); break;
                case SettingsScreen.DoorThemeFolder: _ = AskFolderAsync("ES-DE theme folder", _appSettings.BigPictureTheme, p => { _appSettings.BigPictureTheme = p; ThemeChanged(false); }); break;
            }
        }

        GamepadBindings ISettingsHost.PadBindings => _gamepadBindings;

        ControllerKeyBindings ISettingsHost.KeyBindings => _keyBindings;

        HotkeyBindingMap ISettingsHost.Hotkeys => _hotkeyBindings;

        // Both files saved, the game hearing the new keys at once, and the menus advertising them.
        void ISettingsHost.SaveKeys()
        {
            _keyBindings.Save();
            _hotkeyBindings.Save();
            ApplyKeys();
            SyncMenuState();
        }

        // The console's pad as its settings stand, the running game's ports where it is that console's.
        PadFacts ISettingsHost.PadNow(string console)
        {
            PadFacts facts = RuntimeCatalog.PadNow(console);
            return _session is { } running && console == _activeConsole ? facts with { Players = running.PortControllers } : facts;
        }

        void ISettingsHost.SavePadBindings() => _gamepadBindings.Save();

        IReadOnlyList<string> ISettingsHost.SoundOutputs => AudioPlayer.OutputDevices();

        void ISettingsHost.Ask(string title, string prompt, string current, Action<string> then) => _ = AskInSettingsAsync(title, prompt, current, then);

        // A name typed in; what was kept is applied and Settings drawn again.
        private async Task AskInSettingsAsync(string title, string prompt, string current, Action<string> then)
        {
            if (await Dialogs.PromptAsync((Avalonia.Controls.Window?)_settingsWindow ?? this, title, prompt, current, "Use") is not string typed || typed.Trim().Length == 0) return;
            then(typed.Trim());
            RefreshSettings();
        }

        string? ISettingsHost.ScrapeUser => ((IScrapeHost)this).Member is { IsSet: true } m ? m.User : null;

        long? ISettingsHost.OpenVgdbBytes => ((IScrapeHost)this).OpenVgdbBytes;

        // The account in one line, as the old Scraping tab said it, with the last answer ScreenScraper gave.
        string ISettingsHost.AccountLine
        {
            get
            {
                MemberAccount member = ((IScrapeHost)this).Member;
                string line = !member.IsSet ? "Not signed in: runs use EmuSen's developer credentials alone." : $"Signed in as {member.User}";
                if (member.IsSet && ((IScrapeHost)this).MemberChecked is { Level: { Length: > 0 } level }) line += $" · level {level}";
                return _accountSaid is { Length: > 0 } said ? $"{line.TrimEnd('.')}. {said}" : line;
            }
        }

        private string? _accountSaid;

        string? ISettingsHost.EditingCollection => _editedCollection?.Name;

        // A collection made from Settings starts its editing at once, so Settings closes onto the library where its games are added with Y.
        private async Task NewCollectionAsync()
        {
            await CreateCollectionAsync(this);
            if (_editedCollection is not null) CloseSettings();
            else RefreshSettings();
        }

        private async Task ThenRefreshSettings(Task done)
        {
            await done;
            RefreshSettings();
        }

        private async Task SignInFromSettingsAsync()
        {
            if (await Dialogs.PromptAsync(this, "ScreenScraper", "Your ScreenScraper user name", "", "Next") is not string user) return;
            if (await Dialogs.PromptAsync(this, "ScreenScraper", $"The password for {user}", "", "Log In", secret: true) is not string password) return;
            await SaidInSettings(((IScrapeHost)this).SignInAsync(user, password));
        }

        // A check that fails keeps the account it asked about, and says so.
        private async Task SaidInSettings(Task<SignInAnswer> asked, bool check = false)
        {
            _accountSaid = "Asking ScreenScraper...";
            RefreshSettings();
            SignInAnswer answer = await asked;
            _accountSaid = answer.Message + (check && !answer.SignedIn && ((IScrapeHost)this).Member.IsSet ? " The account is kept until you Log Out." : "");
            RefreshSettings();
        }

        private async Task OpenVgdbFromSettingsAsync()
        {
            var host = (IScrapeHost)this;
            _accountSaid = host.OpenVgdbBytes is null ? await host.DownloadOpenVgdbAsync() : await host.RemoveOpenVgdbAsync();
            RefreshSettings();
        }

        private async Task CleanUpFromSettingsAsync()
        {
            _accountSaid = await ((IScrapeHost)this).CleanUpOrphansAsync();
            RefreshSettings();
        }

        // A folder typed in, as Game Mode never shows the platform's picker; what was kept is saved and Settings drawn again.
        // A folder typed by the setting's rules: a relative path or a file refused and said, a missing folder kept and said, blank back to the default - see EmuSen_Config_Reference.md §7.7.
        private async Task AskFolderAsync(string title, string? current, Action<string?> apply)
        {
            if (await Dialogs.PromptAsync((Avalonia.Controls.Window?)_settingsWindow ?? this, title, $"The {title.ToLowerInvariant()}'s path", current ?? "", "Use") is not string typed) return;
            FolderValue folder = SettingFolders.Checked(typed.Trim());
            StatusText.Text = folder.Problem ?? "";
            if (folder.State is FolderState.Relative or FolderState.File) return;
            apply(folder.State == FolderState.Unset ? null : folder.Written);
            _appSettings.Save();
            RefreshSettings();
        }

        // EmuSen's firmware folder in the file manager, made first since a file manager opens nothing for a path that does not exist; the player's own is never made.
        private static void OpenFirmwareFolder()
        {
            string folder = EmuSen.Galaxia.Library.DataStore.Firmware;
            try { System.IO.Directory.CreateDirectory(folder); } catch (System.IO.IOException) { } catch (UnauthorizedAccessException) { }
            Input.DeviceKeyboard.Launcher.Open(folder);
        }

        bool ISettingsHost.SystemReducesMotion => SystemReducesMotion();

        // Text size and reduced motion through LunaP's own switches, so every layout and movement of the kit follows at once (EmuSen_BigPicture.md §52.8).
        private void ApplyAccessibility()
        {
            EmuSen.LunaP.Theme.LunaTokens.TextScale = SettingsScreen.ScaleOf(_appSettings.TextSize);
            EmuSen.LunaP.Theme.LunaMotion.Reduced = _appSettings.ReducedMotion switch
            {
                AppSettings.ReducedMotionOn => true,
                AppSettings.ReducedMotionOff => false,
                _ => SystemReducesMotion(),
            };
        }

        private static bool? _systemReducesMotion;

        // The desktop's own preference, read once: KDE's AnimationDurationFactor of 0, or GNOME's enable-animations off in its settings file.
        private static bool SystemReducesMotion() => _systemReducesMotion ??= DesktopReducesMotion();

        private static bool DesktopReducesMotion()
        {
            try
            {
                string config = Environment.GetEnvironmentVariable("XDG_CONFIG_HOME") is { Length: > 0 } x ? x : System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".config");
                string kde = System.IO.Path.Combine(config, "kdeglobals");
                if (System.IO.File.Exists(kde) && System.IO.File.ReadLines(kde).Any(l => l.Replace(" ", "") is "AnimationDurationFactor=0" or "AnimationDurationFactor=0.0")) return true;
                string gtk = System.IO.Path.Combine(config, "gtk-3.0", "settings.ini");
                return System.IO.File.Exists(gtk) && System.IO.File.ReadLines(gtk).Any(l => l.Replace(" ", "") is "gtk-enable-animations=0" or "gtk-enable-animations=false");
            }
            catch (Exception e) when (e is System.IO.IOException or UnauthorizedAccessException) { return false; }
        }

        // Light, dark, or the desktop's own choice, through the palette's variant.
        private void ApplyAppearance()
        {
            if (Avalonia.Application.Current is not { } app) return;
            app.RequestedThemeVariant = _appSettings.Appearance switch
            {
                AppSettings.AppearanceLight => ThemeVariant.Light,
                AppSettings.AppearanceSystem => ThemeVariant.Default,
                _ => ThemeVariant.Dark,
            };
        }
    }
}
