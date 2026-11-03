using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Styling;
using EmuSen.Crystal.Client;
using EmuSen.Crystal.Client.Contract;
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
            var model = new SettingsScreen(this, section);
            if (path.Length > 0) model.OpenPath(section, path);
            _settings = new SettingsView(model, layout, SettingsFrame);
            _gamepad.Polled -= OnSettingsPadsPolled;
            _gamepad.Polled += OnSettingsPadsPolled;
            if (!_bigScreen)
            {
                // The desktop's one Settings window, 960 by 640, a sheet over the library (EmuSen_Interface.md §17.7).
                var window = _settingsWindow = new SettingsWindow(layout, button => { SettingsPadCommand(button); return true; });
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
            _settings = null;
            _gamepad.Polled -= OnSettingsPadsPolled;
            if (_settingsPaused && GameOnScreen && IsPaused) ResumeEmulation();
            _settingsPaused = false;
        }

        // The pad on Settings: the directions move and step, A chooses, B closes a page and then Settings, Start goes Home; the shoulders and the rest as the layout declares.
        private void SettingsPadCommand(UiButton button)
        {
            if (_settings is not { } view) return;
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

        private void OnSettingsKey(object? sender, KeyEventArgs e)
        {
            if (e.Handled || KeyAsPad(e.Key) is not { } b) return;
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

        IReadOnlyList<SystemFacts> ISettingsHost.Systems => RuntimeCatalog.Systems;

        IReadOnlyList<SettingRow> ISettingsHost.Schema(string console)
        {
            try { return RuntimeCatalog.SettingsFor(console); }
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

        // The runtime's firmware_status for a system, null where it cannot say (EmuSen_Cores.md §3.11).
        IReadOnlyList<FirmwareFile>? ISettingsHost.Firmware(string system)
        {
            try
            {
                using var answer = System.Text.Json.JsonDocument.Parse(Runtime.FirmwareStatus(system));
                static string Text(System.Text.Json.JsonElement e, string name) => e.TryGetProperty(name, out var v) && v.ValueKind == System.Text.Json.JsonValueKind.String ? v.GetString() ?? "" : "";
                return answer.RootElement.GetProperty("firmware").EnumerateArray()
                    .Select(f => new FirmwareFile(Text(f, "label"), Text(f, "file"), f.TryGetProperty("required", out var r) && r.ValueKind == System.Text.Json.JsonValueKind.True, Text(f, "status"))).ToList();
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

        IReadOnlyList<string> ISettingsHost.Pads => _gamepad.Pads.Select(p => p.Name).ToList();

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

        // A button held on a pad marks its row on the Players page, drawn again only when what is held changes.
        private void OnSettingsPadsPolled()
        {
            if (_settings is not { } view || view.Model.Trail.LastOrDefault() != "Players") return;
            int held = 0;
            for (int i = 0; i < _gamepad.Pads.Count && i < 30; i++) if (_gamepad.Pads[i].AnyPressed()) held |= 1 << i;
            if (held == _padsHeld) return;
            _padsHeld = held;
            RefreshSettings();
        }

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
                case "statusbar" or "messages" or "fps": ApplyStatusBar(); break;
                case "volume": SetVolume(_appSettings.Volume); break;
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
                case SettingsScreen.DoorBindings: ShowControllerBindings(); break;
                case SettingsScreen.DoorThemes: ShowThemeSettings(); break;
                case SettingsScreen.DoorScraper: ShowScraper(); break;
                case SettingsScreen.DoorSignIn: _ = SignInFromSettingsAsync(); break;
                case SettingsScreen.DoorCheckAccount: _ = SaidInSettings(((IScrapeHost)this).CheckMemberAsync()); break;
                case SettingsScreen.DoorSignOut: StatusText.Text = ((IScrapeHost)this).SignOut(); RefreshSettings(); break;
                case SettingsScreen.DoorOpenVgdb: _ = OpenVgdbFromSettingsAsync(); break;
                case SettingsScreen.DoorCleanUp: _ = CleanUpFromSettingsAsync(); break;
                case SettingsScreen.DoorCollections: ShowCollections(); break;
                case SettingsScreen.DoorShell: ShowShellConsole(); break;
                case SettingsScreen.DoorDiscs: ShowCompressedDiscs(); break;
                case SettingsScreen.DoorNewCollection: _ = NewCollectionAsync(); break;
                case SettingsScreen.DoorFinishCollection: FinishEditingCollection(); RefreshSettings(); break;
                case var d when d.StartsWith(SettingsScreen.DoorDeleteCollection, StringComparison.Ordinal) && long.TryParse(d[SettingsScreen.DoorDeleteCollection.Length..], out long id) && _collections.FirstOrDefault(c => c.Id == id) is { } doomed:
                    _ = CollectionThenRefresh(DeleteCollectionAsync(this, doomed)); break;
                case var d when d.StartsWith(SettingsScreen.DoorCardSaves, StringComparison.Ordinal):
                    _ = SheetLayer.Show(new MemoryCardsWindow(_graphics, d[SettingsScreen.DoorCardSaves.Length..]), this); break;
                case SettingsScreen.DoorRomFolder: _ = AskFolderAsync("ROM folder", _appSettings.RomDirectory, p => { _appSettings.RomDirectory = p; RefreshLibrary(); }); break;
                case SettingsScreen.DoorArtFolder: _ = AskFolderAsync("Artwork folder", _appSettings.ArtworkDirectory, p => { _appSettings.ArtworkDirectory = p; ScanArtwork(); }); break;
                case SettingsScreen.DoorStateFolder: _ = AskFolderAsync("State folder", _appSettings.StateDirectory, p => _appSettings.StateDirectory = p); break;
                case SettingsScreen.DoorLogFolder: _ = AskFolderAsync("Log folder", _appSettings.LogDirectory, p => _appSettings.LogDirectory = p); break;
                case SettingsScreen.DoorFirmwareFolder: _ = AskFolderAsync("Firmware folder", _appSettings.FirmwareDirectory, p => _appSettings.FirmwareDirectory = p); break;
                case SettingsScreen.DoorMediaFolder: _ = AskFolderAsync("ES-DE media folder", _appSettings.EsdeMediaDirectory, p => _appSettings.EsdeMediaDirectory = p); break;
                case SettingsScreen.DoorThemeFolder: _ = AskFolderAsync("ES-DE theme folder", _appSettings.BigPictureTheme, p => { _appSettings.BigPictureTheme = p; ThemeChanged(false); }); break;
            }
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
                return _accountSaid is { Length: > 0 } said ? $"{line}. {said}" : line;
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

        private async Task CollectionThenRefresh(Task done)
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

        private async Task SaidInSettings(Task<SignInAnswer> asked)
        {
            _accountSaid = "Asking ScreenScraper...";
            RefreshSettings();
            _accountSaid = (await asked).Message;
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
        private async Task AskFolderAsync(string title, string? current, Action<string> apply)
        {
            if (await Dialogs.PromptAsync(this, title, $"The {title.ToLowerInvariant()}'s path", current ?? "", "Use") is not string typed) return;
            apply(typed.Trim());
            _appSettings.Save();
            RefreshSettings();
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
