using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Theme;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.Media;
using Avalonia.Threading;
using SDL3;
using EmuSen.Cores;
using EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen;
using EmuSen.Mistress.Input;
using EmuSen.Nehellania.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Input;

namespace EmuSen.Mistress.Views
{
    public partial class InputSettingsWindow : Avalonia.Controls.Window
    {
        private readonly ControllerKeyBindings _keyBindings;
        private readonly GamepadBindings _gamepadBindings;
        private readonly HotkeyBindingMap _hotkeyBindings;
        private readonly GamepadManager? _gamepad; // null from the previewer/tests
        private readonly AppSettings _appSettings;

        // Key and pad listeners are separate - an event vs a poll. Both carry the
        // console, since the same PadButton means a different binding per tab.
        private (string Console, PadButton Button)? _listeningForKey;
        private HotkeyAction? _listeningForHotkey;
        private (string Console, PadButton Button)? _listeningForPad;
        private DispatcherTimer? _padPollTimer;
        private DispatcherTimer? _statusPollTimer;

        // XAML handlers fire during InitializeComponent - see EmuSen_Settings_Reference.md §4.6.
        private bool _initialized;

        private const string RebindKeyText = "Rebind Key";
        private const string RebindPadText = "Rebind Pad";
        private const string ListeningText = "Press a key...";
        private const string ListeningPadText = "Press a button...";
        private const string Unbound = "(unbound)";

        private readonly Dictionary<(string, PadButton), TextBlock> _keyLabels = new();
        private readonly Dictionary<(string, PadButton), TextBlock> _padLabels = new();
        private readonly Dictionary<(string, PadButton), Button> _rebindKeyButtons = new();
        private readonly Dictionary<(string, PadButton), Button> _rebindPadButtons = new();
        private readonly Dictionary<HotkeyAction, TextBlock> _hotkeyLabels = new();
        private readonly Dictionary<HotkeyAction, Button> _rebindHotkeyButtons = new();

        // One tab per console this build has, oldest first - see EmuSen_Input.md §5.1.
        private readonly IReadOnlyList<CoreDescriptor> _consoles;

        // For Avalonia's XAML tooling only - real code uses the full ctor.
        public InputSettingsWindow() : this(
            new ControllerKeyBindings(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console)),
            new GamepadBindings(CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console)),
            null!, new AppSettings(), new HotkeyBindingMap())
        { }

        public InputSettingsWindow(ControllerKeyBindings keyBindings, GamepadBindings gamepadBindings, GamepadManager gamepad,
            AppSettings appSettings, HotkeyBindingMap hotkeyBindings, string? selectedConsole = null)
        {
            InitializeComponent();
            _consoles = CoreCatalog.ConsolesInReleaseOrder;
            _keyBindings = keyBindings;
            _gamepadBindings = gamepadBindings;
            _hotkeyBindings = hotkeyBindings;
            _gamepad = gamepad;
            _appSettings = appSettings;

            BuildHotkeyRows();
            BuildConsoleTabs();
            SelectConsoleTab(selectedConsole);
            RefreshConflicts();

            MirrorPlayer1ToPlayer2CheckBox.IsChecked = _appSettings.MirrorPlayer1ToPlayer2;
            AnalogStickAsDpadCheckBox.IsChecked = _appSettings.AnalogStickAsDpad;
            DeadzoneSlider.Value = _appSettings.StickDeadzone;
            UpdateDeadzoneText();
            UpdateControllerStatus();

            // Tunnel, not bubbling, or the focused button eats the key - see EmuSen_Settings_Reference.md §4.2.
            AddHandler(KeyDownEvent, OnPreviewKeyDown, RoutingStrategies.Tunnel, handledEventsToo: true);

            // Notices a pad plugged in or out while the window is open.
            if (_gamepad is not null)
            {
                _statusPollTimer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
                _statusPollTimer.Tick += (_, _) => UpdateControllerStatus();
                _statusPollTimer.Start();
            }

            Closing += (_, _) =>
            {
                _padPollTimer?.Stop();
                _statusPollTimer?.Stop();
            };

            _initialized = true;
        }

        // --- Row construction ---

        // The header and every data row take their columns from here. No Auto anywhere, and see §4.6 for what each width is holding.
        private const string ButtonRowColumns = "90,130,110,68,140,130,85";

        private const string HotkeyRowColumns = "130,130,110,56";

        private void BuildConsoleTabs()
        {
            foreach (CoreDescriptor console in _consoles)
            {
                Tabs.Add(console.Console, new ScrollViewer { Content = BuildConsolePanel(console) });
            }
        }

        private Control BuildConsolePanel(CoreDescriptor console)
        {
            var panel = new StackPanel { Spacing = 4, Margin = new Avalonia.Thickness(0, 8, 0, 0) };

            panel.Children.Add(new TextBlock
            {
                Text = "Game Buttons",
                FontWeight = FontWeight.Bold,
                Margin = new Avalonia.Thickness(0, 0, 0, 2),
            });
            panel.Children.Add(Ui.Hint($"What the emulated {console.Console} controller reads. These bindings are this console's alone.")
                .Margin(0, 0, 0, 6));

            // Named so a layout test can find them; these are built in code, so there is no XAML name scope.
            panel.Children.Add(Ui.Cols(ButtonRowColumns,
                    ColumnHeader("Button"),
                    ColumnHeader("Keyboard"),
                    ColumnHeader("Gamepad").AtColumn(4))
                .Name("ButtonHeaderRow").Margin(0, 0, 0, 2));

            var rows = new StackPanel { Name = "BindingsPanel", Spacing = 6 };
            foreach (PadButton button in CoreCatalog.ButtonsFor(console.Console))
            {
                rows.Children.Add(BuildButtonRow(console.Console, button));
            }
            panel.Children.Add(rows);

            return panel;
        }

        private Control BuildButtonRow(string console, PadButton button)
        {
            var key = (console, button);

            TextBlock keyText = NewValueLabel(CurrentKeyLabel(console, button));
            _keyLabels[key] = keyText;

            Button rebindKey = Ui.Button(RebindKeyText, () => StartListeningForKey(console, button));
            _rebindKeyButtons[key] = rebindKey;

            Button clearKey = Ui.Button("Clear", () =>
            {
                _keyBindings.For(console).Unbind(button);
                _keyBindings.Save();
                RefreshKeyLabels();
            }).Margin(4, 0, 12, 0);

            TextBlock padText = NewValueLabel(CurrentPadLabel(console, button));
            _padLabels[key] = padText;

            Button rebindPad = Ui.Button(RebindPadText, () => StartListeningForPad(console, button));
            _rebindPadButtons[key] = rebindPad;

            Button clearPad = Ui.Button("Clear Pad", () =>
            {
                _gamepadBindings.For(console).Unbind(button);
                _gamepadBindings.Save();
                RefreshPadLabels();
            }).Margin(4, 0, 0, 0);

            // Columns are assigned by position, which is exactly the order the row reads in.
            return Ui.Cols(ButtonRowColumns,
                Ui.Text(button.ToString()).Center(),
                keyText, rebindKey, clearKey, padText, rebindPad, clearPad);
        }

        // Opens on the loaded console's tab, since that is the one being played.
        private void SelectConsoleTab(string? console)
        {
            if (console is null) return;

            for (int i = 0; i < _consoles.Count; i++)
            {
                if (!string.Equals(_consoles[i].Console, console, StringComparison.OrdinalIgnoreCase)) continue;
                Tabs.SelectedIndex = i + 1; // General occupies index 0
                return;
            }
        }

        private void BuildHotkeyRows()
        {
            HotkeyHeaderRow.ColumnDefinitions = new ColumnDefinitions(HotkeyRowColumns);
            HotkeysPanel.Children.Clear();
            _hotkeyLabels.Clear();
            _rebindHotkeyButtons.Clear();

            foreach (HotkeyAction action in Enum.GetValues<HotkeyAction>())
            {
                HotkeyAction captured = action;

                TextBlock keyText = NewValueLabel(CurrentHotkeyLabel(action));
                _hotkeyLabels[action] = keyText;

                Button rebind = Ui.Button(RebindKeyText, () => StartListeningForHotkey(captured));
                _rebindHotkeyButtons[action] = rebind;

                Button clear = Ui.Button("Clear", () =>
                {
                    _hotkeyBindings.Unbind(captured);
                    _hotkeyBindings.Save();
                    RefreshHotkeyLabels();
                }).Margin(4, 0, 0, 0);

                HotkeysPanel.Children.Add(Ui.Cols(HotkeyRowColumns,
                    Ui.Text(HotkeyBindingMap.DisplayName(action)).Center(),
                    keyText, rebind, clear));
            }
        }

        private static TextBlock NewValueLabel(string text) => new()
        {
            Text = text,
            VerticalAlignment = VerticalAlignment.Center,
            TextTrimming = TextTrimming.CharacterEllipsis,
        };

        private static TextBlock ColumnHeader(string text) => new() { Text = text, FontWeight = FontWeight.SemiBold };

        private string CurrentKeyLabel(string console, PadButton button) =>
            _keyBindings.For(console).ButtonToKey.TryGetValue(button, out Key k) ? k.ToString() : Unbound;

        private string CurrentPadLabel(string console, PadButton button) =>
            _gamepadBindings.For(console).ButtonToPad.TryGetValue(button, out SDL.GamepadButton p) ? PadName(p) : Unbound;

        // The connected pad's printed label where SDL3 knows it, else the
        // button's position - see EmuSen_Settings_Reference.md §4.6.
        private string PadName(SDL.GamepadButton pad) => _gamepad?.ButtonLabel(pad) ?? pad.ToString();

        private string CurrentHotkeyLabel(HotkeyAction action) =>
            _hotkeyBindings.ActionToKey.TryGetValue(action, out Key k) ? k.ToString() : Unbound;

        // --- Keyboard capture (game buttons and hotkeys share one listener) ---

        private void StartListeningForKey(string console, PadButton button)
        {
            ClearKeyListening();
            _listeningForKey = (console, button);
            _rebindKeyButtons[(console, button)].Content = ListeningText;
        }

        private void StartListeningForHotkey(HotkeyAction action)
        {
            ClearKeyListening();
            _listeningForHotkey = action;
            _rebindHotkeyButtons[action].Content = ListeningText;
        }

        private void ClearKeyListening()
        {
            if (_listeningForKey is { } b) _rebindKeyButtons[b].Content = RebindKeyText;
            if (_listeningForHotkey is HotkeyAction a) _rebindHotkeyButtons[a].Content = RebindKeyText;
            _listeningForKey = null;
            _listeningForHotkey = null;
        }

        private void OnPreviewKeyDown(object? sender, KeyEventArgs e)
        {
            if (_listeningForKey is null && _listeningForHotkey is null) return;

            // A bare modifier would be an unpressable binding.
            if (e.Key is Key.LeftCtrl or Key.RightCtrl or Key.LeftAlt or Key.RightAlt
                or Key.LeftShift or Key.RightShift or Key.LWin or Key.RWin) return;

            if (e.Key != Key.Escape)
            {
                if (_listeningForKey is { } target)
                {
                    (string console, PadButton button) = target;

                    // A hotkey is global, so it still cannot share a key with any console's button.
                    if (_hotkeyBindings.TryGetAction(e.Key, out HotkeyAction clash))
                    {
                        _hotkeyBindings.Unbind(clash);
                        _hotkeyBindings.Save();
                    }
                    _keyBindings.For(console).Rebind(button, e.Key);
                    _keyBindings.Save();
                }
                else if (_listeningForHotkey is HotkeyAction action)
                {
                    foreach (var kv in _keyBindings.ByConsole)
                    {
                        if (kv.Value.TryGetButton(e.Key, out PadButton clash)) kv.Value.Unbind(clash);
                    }
                    _keyBindings.Save();
                    _hotkeyBindings.Rebind(action, e.Key);
                    _hotkeyBindings.Save();
                }
            }

            _listeningForKey = null;
            _listeningForHotkey = null;
            RefreshKeyLabels();
            RefreshHotkeyLabels();

            // Stops the key re-arming the still-focused rebind button.
            e.Handled = true;
        }

        private void RefreshKeyLabels()
        {
            foreach (var kv in _keyLabels)
            {
                (string console, PadButton button) = kv.Key;
                kv.Value.Text = CurrentKeyLabel(console, button);
                _rebindKeyButtons[kv.Key].Content = RebindKeyText;
            }
            RefreshConflicts();
        }

        private void RefreshHotkeyLabels()
        {
            foreach (HotkeyAction action in Enum.GetValues<HotkeyAction>())
            {
                _hotkeyLabels[action].Text = CurrentHotkeyLabel(action);
                _rebindHotkeyButtons[action].Content = RebindKeyText;
            }
            RefreshConflicts();
        }

        // Per console, since two consoles sharing a key is the point of separate tabs - see EmuSen_Input.md §5.1.
        private void RefreshConflicts()
        {
            var messages = new List<string>();
            var conflicting = new HashSet<(string, PadButton)>();
            var conflictingHotkeys = new HashSet<HotkeyAction>();

            foreach (CoreDescriptor console in _consoles)
            {
                string name = console.Console;
                var buttons = CoreCatalog.ButtonsFor(name).ToHashSet();
                var seen = new Dictionary<Key, List<string>>();

                // Displayed buttons only - a button this console lacks is unreachable, so it cannot clash.
                foreach (var kv in _keyBindings.For(name).ButtonToKey.Where(kv => buttons.Contains(kv.Key)))
                {
                    if (!seen.TryGetValue(kv.Value, out var owners)) seen[kv.Value] = owners = new List<string>();
                    owners.Add(kv.Key.ToString());
                }
                foreach (var kv in _hotkeyBindings.ActionToKey)
                {
                    if (!seen.TryGetValue(kv.Value, out var owners)) seen[kv.Value] = owners = new List<string>();
                    owners.Add(HotkeyBindingMap.DisplayName(kv.Key));
                }

                foreach (var kv in seen.Where(kv => kv.Value.Count > 1))
                {
                    messages.Add($"{name}: {kv.Key} is bound to {string.Join(" and ", kv.Value)}");

                    foreach (PadButton button in buttons)
                    {
                        if (_keyBindings.For(name).ButtonToKey.TryGetValue(button, out Key k) && k == kv.Key)
                        {
                            conflicting.Add((name, button));
                        }
                    }
                    foreach (var hk in _hotkeyBindings.ActionToKey.Where(h => h.Value == kv.Key))
                    {
                        conflictingHotkeys.Add(hk.Key);
                    }
                }
            }

            foreach (var kv in _keyLabels) MarkConflict(kv.Value, conflicting.Contains(kv.Key));
            foreach (var kv in _hotkeyLabels) MarkConflict(kv.Value, conflictingHotkeys.Contains(kv.Key));

            ConflictText.Text = messages.Count == 0 ? "" : "Conflict: " + string.Join("; ", messages);
        }

        // ClearValue, not Foreground = null - see EmuSen_Settings_Reference.md §4.5.
        private static void MarkConflict(TextBlock label, bool conflicting)
        {
            if (conflicting) label.Foreground = Brushes.OrangeRed;
            else label.ClearValue(TextBlock.ForegroundProperty);
        }

        // --- Gamepad rebind: polled, since Avalonia has no pad-press event ---

        private void StartListeningForPad(string console, PadButton button)
        {
            if (_gamepad is null) return; // parameterless-ctor / previewer case

            if (_listeningForPad is { } previous) _rebindPadButtons[previous].Content = RebindPadText;

            _listeningForPad = (console, button);
            _rebindPadButtons[(console, button)].Content = ListeningPadText;

            _padPollTimer?.Stop();
            _padPollTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(50) };
            _padPollTimer.Tick += (_, _) => PollForPadButton();
            _padPollTimer.Start();
        }

        private void PollForPadButton()
        {
            // Guarded locally so the timer stops itself - see EmuSen_Settings_Reference.md §4.6.
            if (_gamepad is null || _listeningForPad is not { } target)
            {
                _padPollTimer?.Stop();
                return;
            }

            (string console, PadButton button) = target;

            SDL.GamepadButton? pressed = _gamepad.GetAnyPressedButton();
            if (pressed is not SDL.GamepadButton padButton) return;

            _gamepadBindings.For(console).Rebind(button, padButton);
            _gamepadBindings.Save();

            _padPollTimer?.Stop();
            _listeningForPad = null;
            RefreshPadLabels();
        }

        private void RefreshPadLabels()
        {
            foreach (var kv in _padLabels)
            {
                (string console, PadButton button) = kv.Key;
                kv.Value.Text = CurrentPadLabel(console, button);
                _rebindPadButtons[kv.Key].Content = RebindPadText;
            }
        }

        private void UpdateControllerStatus()
        {
            if (_gamepad is null)
            {
                ControllerStatusText.Text = "Controller: not available";
                return;
            }

            ControllerStatusText.Text = _gamepad.IsConnected
                ? $"Connected: {_gamepad.ControllerName}"
                : "No controller detected - plug one in and it will be picked up automatically.";
        }

        // --- Option handlers ---

        private void OnMirrorPlayer1ToPlayer2Changed(object? sender, RoutedEventArgs e)
        {
            if (!_initialized) return;
            _appSettings.MirrorPlayer1ToPlayer2 = MirrorPlayer1ToPlayer2CheckBox.IsChecked == true;
            _appSettings.Save();
        }

        private void OnAnalogStickAsDpadChanged(object? sender, RoutedEventArgs e)
        {
            if (!_initialized) return;
            _appSettings.AnalogStickAsDpad = AnalogStickAsDpadCheckBox.IsChecked == true;
            _appSettings.Save();
            if (_gamepad is not null) _gamepad.AnalogStickAsDpad = _appSettings.AnalogStickAsDpad;
        }

        private void OnDeadzoneChanged(object? sender, Avalonia.Controls.Primitives.RangeBaseValueChangedEventArgs e)
        {
            if (!_initialized) return;
            _appSettings.StickDeadzone = DeadzoneSlider.Value;
            _appSettings.Save();
            if (_gamepad is not null) _gamepad.StickDeadzone = _appSettings.StickDeadzone;
            UpdateDeadzoneText();
        }

        private void UpdateDeadzoneText()
        {
            if (DeadzoneText is not null) DeadzoneText.Text = $"{DeadzoneSlider.Value * 100:F0}%";
        }

        private void OnResetClick(object? sender, RoutedEventArgs e)
        {
            _keyBindings.ResetToDefaults();
            _keyBindings.Save();
            _gamepadBindings.ResetToDefaults();
            _gamepadBindings.Save();
            _hotkeyBindings.ResetToDefaults();
            _hotkeyBindings.Save();
            RefreshKeyLabels();
            RefreshPadLabels();
            RefreshHotkeyLabels();
        }

        private void OnCloseClick(object? sender, RoutedEventArgs e)
        {
            Close();
        }
    }
}
