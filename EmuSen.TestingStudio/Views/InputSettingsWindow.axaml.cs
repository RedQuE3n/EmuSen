using System;
using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using Silk.NET.SDL;
using EmuSen.Cores.Nintendo.Venus.Controllers;
using EmuSen.TestingStudio.Input;
using EmuSen.TestingStudio.Settings;

namespace EmuSen.TestingStudio.Views
{
    public partial class InputSettingsWindow : Avalonia.Controls.Window
    {
        private readonly ControllerKeyMap _keyBindings;
        private readonly GamepadBindingMap _gamepadBindings;
        private readonly GamepadManager _gamepad;
        private readonly AppSettings _appSettings;

        // At most one row listens for a key, and independently at most one
        // row listens for a pad button - a key rebind and a pad rebind could
        // in principle be "in progress" at the same time since they're
        // captured completely differently (an event vs a poll), though the
        // UI doesn't really invite doing both at once.
        private SnesButton? _listeningForKey;
        private SnesButton? _listeningForPad;
        private DispatcherTimer? _padPollTimer;

        private readonly Dictionary<SnesButton, TextBlock> _keyLabels = new();
        private readonly Dictionary<SnesButton, TextBlock> _padLabels = new();
        private readonly Dictionary<SnesButton, Button> _rebindKeyButtons = new();
        private readonly Dictionary<SnesButton, Button> _rebindPadButtons = new();

        // Parameterless constructor exists only so Avalonia's XAML tooling
        // (previewer, generated InitializeComponent) is happy - always use
        // the full constructor in real code (see MainWindow's menu handler).
        public InputSettingsWindow() : this(new ControllerKeyMap(), new GamepadBindingMap(), null!, new AppSettings()) { }

        public InputSettingsWindow(ControllerKeyMap keyBindings, GamepadBindingMap gamepadBindings, GamepadManager gamepad, AppSettings appSettings)
        {
            InitializeComponent();
            _keyBindings = keyBindings;
            _gamepadBindings = gamepadBindings;
            _gamepad = gamepad;
            _appSettings = appSettings;
            BuildRows();
            MirrorPlayer1ToPlayer2CheckBox.IsChecked = _appSettings.MirrorPlayer1ToPlayer2;
            KeyDown += OnWindowKeyDown;
            Closing += (_, _) => _padPollTimer?.Stop();
        }

        private void OnMirrorPlayer1ToPlayer2Changed(object? sender, RoutedEventArgs e)
        {
            _appSettings.MirrorPlayer1ToPlayer2 = MirrorPlayer1ToPlayer2CheckBox.IsChecked == true;
            _appSettings.Save();
        }

        private void BuildRows()
        {
            BindingsPanel.Children.Clear();
            _keyLabels.Clear();
            _padLabels.Clear();
            _rebindKeyButtons.Clear();
            _rebindPadButtons.Clear();

            foreach (SnesButton button in Enum.GetValues<SnesButton>())
            {
                // Key labels are Avalonia Key.ToString() ("RightBracket",
                // "LeftShift", ...) and pad labels are SDL
                // GameControllerButton.ToString() ("Leftshoulder",
                // "Rightshoulder", ...) - both routinely longer than the
                // original 90px columns, which let them visually overflow
                // underneath the next column's button (added later in
                // Children, so it painted on top) instead of wrapping or
                // clipping. Widened columns plus TextTrimming below are
                // the fix; the window itself was widened to match.
                var row = new Grid { ColumnDefinitions = new ColumnDefinitions("80,110,Auto,150,Auto") };

                var nameText = new TextBlock { Text = button.ToString(), VerticalAlignment = VerticalAlignment.Center };
                Grid.SetColumn(nameText, 0);

                var keyText = new TextBlock
                {
                    Text = CurrentKeyLabel(button),
                    VerticalAlignment = VerticalAlignment.Center,
                    TextTrimming = TextTrimming.CharacterEllipsis,
                };
                Grid.SetColumn(keyText, 1);
                _keyLabels[button] = keyText;

                var rebindKeyButton = new Button { Content = "Rebind Key" };
                Grid.SetColumn(rebindKeyButton, 2);
                rebindKeyButton.Click += (_, _) => StartListeningForKey(button);
                _rebindKeyButtons[button] = rebindKeyButton;

                var padText = new TextBlock
                {
                    Text = CurrentPadLabel(button),
                    VerticalAlignment = VerticalAlignment.Center,
                    TextTrimming = TextTrimming.CharacterEllipsis,
                };
                Grid.SetColumn(padText, 3);
                _padLabels[button] = padText;

                var rebindPadButton = new Button { Content = "Rebind Pad" };
                Grid.SetColumn(rebindPadButton, 4);
                rebindPadButton.Click += (_, _) => StartListeningForPad(button);
                _rebindPadButtons[button] = rebindPadButton;

                row.Children.Add(nameText);
                row.Children.Add(keyText);
                row.Children.Add(rebindKeyButton);
                row.Children.Add(padText);
                row.Children.Add(rebindPadButton);
                BindingsPanel.Children.Add(row);
            }
        }

        private string CurrentKeyLabel(SnesButton button) =>
            _keyBindings.ButtonToKey.TryGetValue(button, out Key k) ? k.ToString() : "(unbound)";

        private string CurrentPadLabel(SnesButton button) =>
            _gamepadBindings.ButtonToPad.TryGetValue(button, out GameControllerButton p) ? p.ToString() : "(unbound)";

        // --- Keyboard rebind: capture via the window's KeyDown event ---

        private void StartListeningForKey(SnesButton button)
        {
            if (_listeningForKey is SnesButton previous) _rebindKeyButtons[previous].Content = "Rebind Key";

            _listeningForKey = button;
            _rebindKeyButtons[button].Content = "Press a key...";
        }

        private void OnWindowKeyDown(object? sender, KeyEventArgs e)
        {
            if (_listeningForKey is not SnesButton button) return;

            if (e.Key != Key.Escape)
            {
                _keyBindings.Rebind(button, e.Key);
                _keyBindings.Save();
            }

            _listeningForKey = null;
            RefreshKeyLabels();
            e.Handled = true;
        }

        private void RefreshKeyLabels()
        {
            foreach (SnesButton button in Enum.GetValues<SnesButton>())
            {
                _keyLabels[button].Text = CurrentKeyLabel(button);
                _rebindKeyButtons[button].Content = "Rebind Key";
            }
        }

        // --- Gamepad rebind: no button-press event exists in Avalonia, so
        // this polls GamepadManager on a short timer until something is held
        // down, same GamepadManager instance MainWindow uses for actual
        // gameplay input (passed in via the constructor) rather than opening
        // a second SDL controller handle. ---

        private void StartListeningForPad(SnesButton button)
        {
            if (_gamepad is null) return; // parameterless-ctor / previewer case

            if (_listeningForPad is SnesButton previous) _rebindPadButtons[previous].Content = "Rebind Pad";

            _listeningForPad = button;
            _rebindPadButtons[button].Content = "Press a button...";

            _padPollTimer?.Stop();
            _padPollTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(50) };
            _padPollTimer.Tick += (_, _) => PollForPadButton();
            _padPollTimer.Start();
        }

        private void PollForPadButton()
        {
            if (_listeningForPad is not SnesButton button)
            {
                _padPollTimer?.Stop();
                return;
            }

            GameControllerButton? pressed = _gamepad.GetAnyPressedButton();
            if (pressed is not GameControllerButton padButton) return;

            _gamepadBindings.Rebind(button, padButton);
            _gamepadBindings.Save();

            _padPollTimer?.Stop();
            _listeningForPad = null;
            RefreshPadLabels();
        }

        private void RefreshPadLabels()
        {
            foreach (SnesButton button in Enum.GetValues<SnesButton>())
            {
                _padLabels[button].Text = CurrentPadLabel(button);
                _rebindPadButtons[button].Content = "Rebind Pad";
            }
        }

        private void OnResetClick(object? sender, RoutedEventArgs e)
        {
            _keyBindings.ResetToDefaults();
            _keyBindings.Save();
            _gamepadBindings.ResetToDefaults();
            _gamepadBindings.Save();
            RefreshKeyLabels();
            RefreshPadLabels();
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
