using System;
using System.IO;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Automation;
using Avalonia.Controls.Primitives;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Commands;
using EmuSen.LunaP.Controls;

namespace EmuSen.Mistress.Views
{
    // OpenEmu's heads-up bar over the game, and its notices - see EmuSen_Settings_Reference.md §4.34.
    public partial class MainWindow
    {
        // Path data for the bar's glyphs, drawn rather than taken from a font so no fallback font decides them.
        private const string PowerGlyph = "M11,2H13V12H11Z M7.05,5.64L8.46,7.05A6,6 0 1,0 15.54,7.05L16.95,5.64A8,8 0 1,1 7.05,5.64Z";
        private const string PauseGlyph = "M6,4H10V20H6Z M14,4H18V20H14Z";
        private const string PlayGlyph = "M7,4L20,12L7,20Z";
        private const string RestartGlyph = "M12,4V1L7,5L12,9V6A6,6 0 1,1 6,12H4A8,8 0 1,0 12,4Z";
        private const string SaveGlyph = "M5,3H16L20,7V21H4V3Z M7,5V9H15V5Z M12,13A3,3 0 1,0 12,19A3,3 0 1,0 12,13Z";
        private const string OptionsGlyph = "M10,2H14L14.5,5A7,7 0 0,1 16.6,6.2L19.5,5.2L21.5,8.7L19.2,10.7A7,7 0 0,1 19.2,13.3L21.5,15.3L19.5,18.8L16.6,17.8A7,7 0 0,1 14.5,19L14,22H10L9.5,19A7,7 0 0,1 7.4,17.8L4.5,18.8L2.5,15.3L4.8,13.3A7,7 0 0,1 4.8,10.7L2.5,8.7L4.5,5.2L7.4,6.2A7,7 0 0,1 9.5,5Z M12,9A3,3 0 1,0 12,15A3,3 0 1,0 12,9Z";
        private const string QuietGlyph = "M3,9H7L12,4V20L7,15H3Z";
        private const string LoudGlyph = "M3,9H7L12,4V20L7,15H3Z M14,7A6,6 0 0,1 14,17V15A4,4 0 0,0 14,9Z M14,3A10,10 0 0,1 14,21V19A8,8 0 0,0 14,5Z";
        private const string FullScreenGlyph = "M3,3H10V5H5V10H3Z M14,3H21V10H19V5H14Z M3,14H5V19H10V21H3Z M19,14H21V21H14V19H19Z";

        private Button? _hudPause;
        private Slider? _hudVolume;
        private int _hudMenusOpen;

        private void SetUpHud()
        {
            GameHud.Watch = GameFrame;
            Button power = HudButton("HudPower", PowerGlyph, "Quit Game", ShowLibrary, pill: true);
            power.Classes.Add("hud-power");
            _hudPause = HudButton("HudPause", PauseGlyph, "Pause Game", TogglePause);
            Button restart = HudButton("HudRestart", RestartGlyph, "Restart Game", ResetEmulation);
            Button save = HudMenuButton("HudSave", SaveGlyph, "Save and Load", _saveState, _loadState, _slotMenu);
            Button options = HudMenuButton("HudOptions", OptionsGlyph, "Options", _speedMenu,
                new LunaAction("_Graphics Settings...", ShowGraphicsSettings),
                new LunaAction("S_haders...", ShowShaderSettings),
                new LunaAction("_Controller Bindings...", ShowControllerBindings),
                new LunaAction("Active _Cheats...", ShowActiveCheats),
                new LunaAction("Take a _Screenshot", TakeScreenshot));

            _hudVolume = new Slider { Name = "HudVolume", Minimum = 0, Maximum = 1, Width = 70, VerticalAlignment = VerticalAlignment.Center, Value = Math.Clamp(_appSettings.Volume, 0, 1) };
            AutomationProperties.SetName(_hudVolume, "Volume");
            _hudVolume.ValueChanged += (_, e) => SetVolume(e.NewValue);
            Button mute = HudButton("HudMute", QuietGlyph, "Mute", () => _hudVolume.Value = 0);
            Button loud = HudButton("HudLoud", LoudGlyph, "Full Volume", () => _hudVolume.Value = 1);
            mute.Width = loud.Width = 22;
            Button fullScreen = HudButton("HudFullScreen", FullScreenGlyph, "Toggle Full Screen", () => IsFullScreen = !IsFullScreen, pill: true);

            var left = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 2, VerticalAlignment = VerticalAlignment.Center, Children = { power, Gap(14), _hudPause, restart, Gap(15), save, Gap(8), options } };
            var right = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 3, VerticalAlignment = VerticalAlignment.Center, Children = { mute, _hudVolume, loud, Gap(14), fullScreen } };
            DockPanel.SetDock(left, Dock.Left);
            DockPanel.SetDock(right, Dock.Right);
            GameHud.Content = new DockPanel { LastChildFill = false, Children = { left, right } };
            _audioPlayer.Volume = (float)_hudVolume.Value;
        }

        private static Control Gap(double width) => new Border { Width = width };

        private static Button HudButton(string name, string glyph, string tip, Action click, bool pill = false)
        {
            var button = new Button { Name = name, Content = Glyph(glyph), Classes = { pill ? "hud-pill" : "hud" } };
            ToolTip.SetTip(button, tip);
            AutomationProperties.SetName(button, tip);
            button.Click += (_, _) => click();
            return button;
        }

        private static PathIcon Glyph(string data) => new() { Data = StreamGeometry.Parse(data), Width = 16, Height = 16 };

        // Its menu is LunaP's from the same actions the menu bar holds, so neither can show the other a stale label.
        private Button HudMenuButton(string name, string glyph, string tip, params LunaAction[] actions)
        {
            var flyout = new MenuFlyout { ItemsSource = Menus.Items(actions), Placement = PlacementMode.Top };
            flyout.Opened += (_, _) => { _hudMenusOpen++; GameHud.KeepOpen = true; };
            flyout.Closed += (_, _) => GameHud.KeepOpen = --_hudMenusOpen > 0;
            Button button = HudButton(name, glyph, tip, () => { });
            button.Flyout = flyout;
            return button;
        }

        // Set only when this window did the pausing, so a pause the player chose survives the window losing focus.
        private bool _pausedInBackground;

        private void SetUpBackgroundPause()
        {
            Deactivated += (_, _) =>
            {
                if (!_appSettings.PauseInBackground || !GameFrame.IsVisible || _session is not { IsRomLoaded: true } || IsPaused) return;
                PauseEmulation();
                _pausedInBackground = true;
            };
            Activated += (_, _) =>
            {
                if (!_pausedInBackground) return;
                _pausedInBackground = false;
                if (GameFrame.IsVisible && IsPaused && !PadMenuPanel.IsVisible && !Sheets.IsPresenting) ResumeEmulation();
            };
        }

        private void SetVolume(double volume)
        {
            _audioPlayer.Volume = (float)volume;
            _appSettings.Volume = volume;
            _appSettings.Save();
        }

        // The bar belongs to the game screen; the pause glyph says what pressing it will do.
        private void SyncHud()
        {
            if (_hudPause is null) return;
            bool paused = _session is { IsRomLoaded: true } && IsPaused;
            _hudPause.Content = Glyph(paused ? PlayGlyph : PauseGlyph);
            ToolTip.SetTip(_hudPause, paused ? "Resume Game" : "Pause Game");
            AutomationProperties.SetName(_hudPause, paused ? "Resume Game" : "Pause Game");
            if (!GameFrame.IsVisible) GameHud.Conceal();
        }

        private void Notify(string text)
        {
            if (GameFrame.IsVisible) GameNotice.Show(text);
        }

        // Written on the emulation thread with the frame the machine is on, like a state's picture - see EmuSen_Settings_Reference.md §4.34.
        private void TakeScreenshot()
        {
            if (_session is not { IsRomLoaded: true } || _currentRomPath is not string rom) return;
            string path = Path.Combine(DataStore.Screenshots, $"{Path.GetFileNameWithoutExtension(rom)} {DateTime.Now:yyyy-MM-dd HH.mm.ss.fff}.png");
            RequestOnEmulationThread(session =>
            {
                string status;
                try
                {
                    Directory.CreateDirectory(DataStore.Screenshots);
                    WritePicture(session, path);
                    status = $"Screenshot saved: {Path.GetFileName(path)}";
                }
                catch (Exception ex)
                {
                    status = $"Screenshot failed: {ex.Message}";
                }
                Dispatcher.UIThread.Post(() =>
                {
                    StatusText.Text = status;
                    Notify(status.StartsWith("Screenshot saved", StringComparison.Ordinal) ? "Screenshot saved" : status);
                });
            });
        }
    }
}
