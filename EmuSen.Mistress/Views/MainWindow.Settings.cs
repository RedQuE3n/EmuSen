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

namespace EmuSen.Mistress.Views
{
    // Settings over the whole screen on the big screen, every door to it, and what it reads and asks of the window - see EmuSen_BigPicture.md §52.
    public partial class MainWindow : ISettingsHost
    {
        private SettingsView? _settings;

        public SettingsScreen? SettingsShown => _settings?.Model;

        public SettingsLayout? SettingsLayoutShown => _settings?.Layout;

        // Every door: Home's place, the card's Settings, the pad menu's and the desktop's menu open Settings at a section, and a page under it where asked.
        public void ShowSettings(string section = SettingsScreen.Controllers, params string[] path)
        {
            if (!_bigScreen)
            {
                ShowPreferences();
                return;
            }
            if (_padMenuOpen) ClosePadMenu();
            CloseSettings(restoreFocus: false);
            var layout = new SettingsLayout { Name = "SettingsScreen", PadFamily = HelpFamily == PadFamily.Generic ? PadFamily.Xbox : HelpFamily };
            var model = new SettingsScreen(this, section);
            if (path.Length > 0) model.OpenPath(section, path);
            _settings = new SettingsView(model, layout, SettingsFrame);
            layout.KeyDown += OnSettingsKey;
            ScreenContent.Children.Add(layout);
            StatusBar.IsVisible = false;
            layout.Focus();
        }

        public void CloseSettings(bool restoreFocus = true)
        {
            if (_settings is not { } shown) return;
            _settings = null;
            shown.Layout.KeyDown -= OnSettingsKey;
            ScreenContent.Children.Remove(shown.Layout);
            ApplyStatusBar();
            if (!restoreFocus) return;
            if (OwnScreenShown) ShowOwnLook(false);
            if (HomeLayout is { } home) home.Focus();
            else if (ThemedLibraryShown) ThemedLibraryHost.Focus();
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
            if (ToolPaneWindow.Glyph(button) is { } glyph) view.Layout.Press(glyph);
        }

        private void OnSettingsKey(object? sender, KeyEventArgs e)
        {
            if (e.Handled) return;
            UiButton? button = e.Key switch
            {
                Key.Up => UiButton.Up, Key.Down => UiButton.Down, Key.Left => UiButton.Left, Key.Right => UiButton.Right,
                Key.Enter => UiButton.Accept, Key.Escape or Key.Back => UiButton.Back, Key.PageUp => UiButton.PageUp, Key.PageDown => UiButton.PageDown,
                _ => null,
            };
            if (button is not { } b) return;
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
                case SettingsScreen.DoorScraper: ShowPreferencesAt(PreferencesWindow.ScrapingTab); break;
                case SettingsScreen.DoorCollections: ShowCollections(); break;
                case SettingsScreen.DoorShell: ShowShellConsole(); break;
                case SettingsScreen.DoorDiscs: ShowCompressedDiscs(); break;
                case SettingsScreen.DoorRomFolder: _ = AskFolderAsync("ROM folder", _appSettings.RomDirectory, p => { _appSettings.RomDirectory = p; RefreshLibrary(); }); break;
                case SettingsScreen.DoorArtFolder: _ = AskFolderAsync("Artwork folder", _appSettings.ArtworkDirectory, p => { _appSettings.ArtworkDirectory = p; ScanArtwork(); }); break;
                case SettingsScreen.DoorStateFolder: _ = AskFolderAsync("State folder", _appSettings.StateDirectory, p => _appSettings.StateDirectory = p); break;
                case SettingsScreen.DoorLogFolder: _ = AskFolderAsync("Log folder", _appSettings.LogDirectory, p => _appSettings.LogDirectory = p); break;
                case SettingsScreen.DoorFirmwareFolder: _ = AskFolderAsync("Firmware folder", _appSettings.FirmwareDirectory, p => _appSettings.FirmwareDirectory = p); break;
                case SettingsScreen.DoorMediaFolder: _ = AskFolderAsync("ES-DE media folder", _appSettings.EsdeMediaDirectory, p => _appSettings.EsdeMediaDirectory = p); break;
                case SettingsScreen.DoorThemeFolder: _ = AskFolderAsync("ES-DE theme folder", _appSettings.BigPictureTheme, p => { _appSettings.BigPictureTheme = p; ThemeChanged(false); }); break;
            }
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
