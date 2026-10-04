using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Controls;
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
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Mistress.Views
{
    public partial class InputSettingsWindow : ToolWindow, IPadCapturing, IPadDriven
    {
        private readonly ControllerKeyBindings _keyBindings;
        private readonly GamepadBindings _gamepadBindings;
        private readonly HotkeyBindingMap _hotkeyBindings;
        private readonly GamepadManager? _gamepad; // null from the previewer/tests
        private readonly AppSettings _appSettings;

        // Key and pad listeners are separate (an event vs a poll) and carry the console; a region chosen on a drawing sets both - see §4.81.
        private (string Console, PadControl Button)? _listeningForKey;
        private HotkeyAction? _listeningForHotkey;
        private (string Console, PadControl Button)? _listeningForPad;
        private DispatcherTimer? _padPollTimer;
        private DispatcherTimer? _statusPollTimer;

        // The button that chose Rebind Pad is still down when listening starts, so a press counts only after all are let go - see EmuSen_Settings_Reference.md §4.45.4.
        private bool _padArmed;
        private bool _padReleaseWait;
        private readonly System.Diagnostics.Stopwatch _padListening = new();

        // How long a pad capture waits for a press before giving up, since every button it could be cancelled with is one it could bind.
        public TimeSpan PadCaptureTimeout { get; set; } = TimeSpan.FromSeconds(5);

        public PadCapture Capturing =>
            _listeningForPad is not null || _padReleaseWait ? PadCapture.PadButton
            : _listeningForKey is not null || _listeningForHotkey is not null ? PadCapture.Key
            : PadCapture.None;

        public void CancelCapture()
        {
            ClearKeyListening();
            StopListeningForPad();
        }

        private void StopListeningForPad()
        {
            _padPollTimer?.Stop();
            if (_listeningForPad is { } target) _rebindPadButtons[target].Content = RebindPadText;
            _listeningForPad = null;
            RefreshDiagramLabels();
        }

        // XAML handlers fire during InitializeComponent - see EmuSen_Settings_Reference.md §4.6.
        private bool _initialized;

        private const string RebindKeyText = "Rebind Key";
        private const string RebindPadText = "Rebind Pad";
        private const string ListeningText = "Press a key...";
        private const string ListeningPadText = "Press a button...";
        private const string Unbound = "(unbound)";

        private readonly Dictionary<(string, PadControl), TextBlock> _keyLabels = new();
        private readonly Dictionary<(string, PadControl), TextBlock> _padLabels = new();
        private readonly Dictionary<(string, PadControl), Button> _rebindKeyButtons = new();
        private readonly Dictionary<(string, PadControl), Button> _rebindPadButtons = new();
        private readonly Dictionary<HotkeyAction, TextBlock> _hotkeyLabels = new();
        private readonly Dictionary<HotkeyAction, Button> _rebindHotkeyButtons = new();

        // The player whose pad bindings each console's tab shows; 1 until chosen - see EmuSen_Input.md §8.4.
        private readonly Dictionary<string, int> _playerByConsole = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, (TextBlock Words, TextBlock KeyboardHeader)> _playerRows = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dropdown _playerChoice = new() { Name = "PlayerSelector", Width = 130 };
        private readonly Button _usePlayer1;

        public int PlayerOf(string console) => _playerByConsole.GetValueOrDefault(console, 1);

        private GamepadBindingMap PadMap(string console) => _gamepadBindings.For(console, PlayerOf(console));

        // A change to a player past the first gives it a map of its own, begun from player 1's.
        private GamepadBindingMap OwnPadMap(string console) => _gamepadBindings.Own(console, PlayerOf(console));

        // The selected player's pad, or with none seated player 1's the first connected, as before players.
        private ConnectedPad? TesterPad(string console) =>
            _gamepad?.Players.PadFor(PlayerOf(console)) ?? (PlayerOf(console) == 1 ? _gamepad?.Primary : null);

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
            _usePlayer1 = Ui.Button("Use Player 1's", UsePlayer1).HelpText("Give the player shown player 1's gamepad buttons again");
            _consoles = CoreCatalog.ConsolesInReleaseOrder;
            _keyBindings = keyBindings;
            _gamepadBindings = gamepadBindings;
            _hotkeyBindings = hotkeyBindings;
            _gamepad = gamepad;
            _appSettings = appSettings;

            BuildHotkeyRows();
            BuildConsoleTabs();
            SelectConsoleTab(selectedConsole);
            SetUpPlayerBar();
            RefreshConflicts();
            RefreshDiagramLabels();

            MirrorPlayer1ToPlayer2CheckBox.IsChecked = _appSettings.MirrorPlayer1ToPlayer2;
            AnalogStickAsDpadCheckBox.IsChecked = _appSettings.AnalogStickAsDpad;
            DeadzoneSlider.Value = _appSettings.StickDeadzone;
            UpdateDeadzoneText();
            UpdateControllerStatus();

            // Tunnel, not bubbling, or the focused button eats the key (§4.2); on the content, which a sheet takes with it (§4.45.2).
            ((Control)Content!).AddHandler(KeyDownEvent, OnPreviewKeyDown, RoutingStrategies.Tunnel, handledEventsToo: true);
            ((Control)Content!).AddHandler(KeyUpEvent, OnPreviewKeyUp, RoutingStrategies.Tunnel, handledEventsToo: true);
            // A key typed with the focus on nothing reaches the window alone.
            AddHandler(KeyDownEvent, (s, e) => { if (ReferenceEquals(e.Source, this)) OnPreviewKeyDown(s, e); }, RoutingStrategies.Tunnel, handledEventsToo: true);
            AddHandler(KeyUpEvent, (s, e) => { if (ReferenceEquals(e.Source, this)) OnPreviewKeyUp(s, e); }, RoutingStrategies.Tunnel, handledEventsToo: true);
            SetUpTester();

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
                TearDownTester();
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
                var page = new ScrollViewer();
                Control list = BuildConsolePanel(console);
                page.Content = WithDiagram(console.Console, page, list);
                Tabs.Add(console.Console, page);
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

            TextBlock keyboardHeader = ColumnHeader(KeyboardHeader(console.Console));
            if (EmuSen.Cores.ControllerPorts.ForConsole(console.Console) > 1)
            {
                var words = new TextBlock { Name = $"PlayerWords{console.Console}", TextWrapping = TextWrapping.Wrap, Margin = new Avalonia.Thickness(0, 0, 0, 6) };
                _playerRows[console.Console] = (words, keyboardHeader);
                panel.Children.Add(words);
                UpdatePlayerRow(console.Console);
            }

            // Named so a layout test can find them; these are built in code, so there is no XAML name scope.
            panel.Children.Add(Ui.Cols(ButtonRowColumns,
                    ColumnHeader("Button"),
                    keyboardHeader,
                    ColumnHeader("Gamepad").AtColumn(4))
                .Name("ButtonHeaderRow").Margin(0, 0, 0, 2));

            var rows = new StackPanel { Name = "BindingsPanel", Spacing = 6 };
            foreach (PadControl control in CoreCatalog.ControlsFor(console.Console))
            {
                rows.Children.Add(BuildButtonRow(console.Console, control));
            }
            panel.Children.Add(rows);

            return panel;
        }

        // One selector beside the tabs for the console shown, so the drawing keeps its height - see EmuSen_Input.md §8.4.
        private void SetUpPlayerBar()
        {
            _usePlayer1.Name = "UsePlayer1Button";
            Avalonia.Automation.AutomationProperties.SetName(_playerChoice, "Player");
            PlayerBar.Children.Add(Ui.Text("Player").Center());
            PlayerBar.Children.Add(_playerChoice);
            PlayerBar.Children.Add(_usePlayer1);
            _playerChoice.Chose += chosen =>
            {
                if (ShownConsole is not { } console || chosen is not string text || !int.TryParse(text.AsSpan("Player ".Length), out int player)) return;
                CancelCapture();
                _playerByConsole[console] = player;
                RefreshPadLabels();
            };
            Tabs.SelectionChanged += (_, _) => ShowPlayerBar();
            ShowPlayerBar();
        }

        private void ShowPlayerBar()
        {
            string? console = ShownConsole;
            int ports = EmuSen.Cores.ControllerPorts.ForConsole(console);
            PlayerBar.IsVisible = console is not null && ports > 1;
            if (!PlayerBar.IsVisible) return;
            _playerChoice.Fill(Enumerable.Range(1, ports).Select(PlayerPreferencesRows.Player).ToArray(), PlayerPreferencesRows.Player(Math.Min(PlayerOf(console!), ports)));
            _usePlayer1.IsEnabled = _gamepadBindings.HasOwn(console!, PlayerOf(console!));
        }

        private void UsePlayer1()
        {
            if (ShownConsole is not { } console) return;
            _gamepadBindings.Forget(console, PlayerOf(console));
            _gamepadBindings.Save();
            RefreshPadLabels();
        }

        private string KeyboardHeader(string console) =>
            _appSettings.KeyboardPlayer > 1 || PlayerOf(console) != 1 ? $"Keyboard (Player {Math.Max(1, _appSettings.KeyboardPlayer)})" : "Keyboard";

        private void UpdatePlayerRow(string console)
        {
            if (!_playerRows.TryGetValue(console, out var row)) return;
            int player = PlayerOf(console);
            bool own = _gamepadBindings.HasOwn(console, player);
            row.Words.Text = player == 1 ? "Showing player 1's gamepad buttons; every other player has these until given its own (Player, above the drawing)."
                : own ? $"Showing player {player}'s own gamepad buttons." : $"Showing player {player}, who has player 1's gamepad buttons until one is changed here.";
            row.KeyboardHeader.Text = KeyboardHeader(console);
            if (console == ShownConsole) _usePlayer1.IsEnabled = own;
        }

        private Control BuildButtonRow(string console, PadControl button)
        {
            var key = (console, button);

            TextBlock keyText = NewValueLabel(CurrentKeyLabel(console, button));
            _keyLabels[key] = keyText;

            // The caption stays for voice control and the context goes in help text - see LunaP.md §24.2.
            Button rebindKey = Ui.Button(RebindKeyText, () => StartListeningForKey(console, button))
                .HelpText($"Keyboard key for {console} {FullName(button)}");
            _rebindKeyButtons[key] = rebindKey;

            Button clearKey = Ui.Button("Clear", () =>
            {
                _keyBindings.For(console).Unbind(button);
                _keyBindings.Save();
                RefreshKeyLabels();
            }).Margin(4, 0, 12, 0).HelpText($"Clear the keyboard key for {console} {FullName(button)}");

            TextBlock padText = NewValueLabel(CurrentPadLabel(console, button));
            _padLabels[key] = padText;

            // A stick direction comes from the pad's own stick, so it has no pad button to bind - see EmuSen_Input.md §7.3.
            bool isButton = PadControls.IsButton(button, out PadButton padButton);

            Button rebindPad = Ui.Button(RebindPadText, () => StartListeningForPad(console, button))
                .HelpText($"Gamepad button for {console} {FullName(button)}");
            rebindPad.IsEnabled = isButton;
            _rebindPadButtons[key] = rebindPad;

            Button clearPad = Ui.Button("Clear Pad", () =>
            {
                OwnPadMap(console).Unbind(padButton);
                _gamepadBindings.Save();
                RefreshPadLabels();
            }).Margin(4, 0, 0, 0).HelpText($"Clear the gamepad button for {console} {FullName(button)}");
            clearPad.IsEnabled = isButton;

            // Columns are assigned by position, which is exactly the order the row reads in.
            return Ui.Cols(ButtonRowColumns,
                Ui.Text(ShortName(button)).Center(),
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

                string hotkeyName = HotkeyBindingMap.DisplayName(action);

                Button rebind = Ui.Button(RebindKeyText, () => StartListeningForHotkey(captured))
                    .HelpText($"Shortcut key for {hotkeyName}");
                _rebindHotkeyButtons[action] = rebind;

                Button clear = Ui.Button("Clear", () =>
                {
                    _hotkeyBindings.Unbind(captured);
                    _hotkeyBindings.Save();
                    RefreshHotkeyLabels();
                }).Margin(4, 0, 0, 0).HelpText($"Clear the shortcut key for {hotkeyName}");

                HotkeysPanel.Children.Add(Ui.Cols(HotkeyRowColumns,
                    Ui.Text(hotkeyName).Center(),
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

        private string CurrentKeyLabel(string console, PadControl button) =>
            _keyBindings.For(console).ButtonToKey.TryGetValue(button, out Key k) ? k.ToString() : Unbound;

        private string CurrentPadLabel(string console, PadControl control)
        {
            if (!PadControls.IsButton(control, out PadButton button)) return control <= PadControl.LeftStickRight ? "Left stick" : "Right stick";
            string? bound = PadMap(console).ButtonToPad.TryGetValue(button, out SDL.GamepadButton p) ? PadName(p) : null;
            string? trigger = TriggerFor(button);
            return bound is null ? trigger ?? Unbound : trigger is null ? bound : $"{bound} or {trigger}";
        }

        // L2 and R2 are always their trigger, past half its travel, whatever button is bound beside it - see EmuSen_Input.md §7.3 and §4.81.
        private static string? TriggerFor(PadButton button) => button switch
        {
            PadButton.L2 => "Left Trigger",
            PadButton.R2 => "Right Trigger",
            _ => null,
        };

        // Short enough for the button column; the help text carries the whole name.
        private static string ShortName(PadControl control) => control switch
        {
            PadControl.LeftStickUp => "LS Up",
            PadControl.LeftStickDown => "LS Down",
            PadControl.LeftStickLeft => "LS Left",
            PadControl.LeftStickRight => "LS Right",
            PadControl.RightStickUp => "RS Up",
            PadControl.RightStickDown => "RS Down",
            PadControl.RightStickLeft => "RS Left",
            PadControl.RightStickRight => "RS Right",
            _ => control.ToString(),
        };

        private static string FullName(PadControl control) => control switch
        {
            >= PadControl.LeftStickUp => System.Text.RegularExpressions.Regex.Replace(control.ToString(), "(?<=[a-z])(?=[A-Z])", " ").ToLowerInvariant(),
            _ => control.ToString(),
        };

        // The connected pad's printed label where SDL3 knows it, else the button's position - see EmuSen_Settings_Reference.md §4.6.
        private string PadName(SDL.GamepadButton pad) => _gamepad?.ButtonLabel(pad) ?? pad.ToString();

        private string CurrentHotkeyLabel(HotkeyAction action) =>
            _hotkeyBindings.ActionToKey.TryGetValue(action, out Key k) ? k.ToString() : Unbound;

        // --- Keyboard capture (game buttons and hotkeys share one listener) ---

        private void StartListeningForKey(string console, PadControl button)
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
            RefreshDiagramLabels();
        }

        private void OnPreviewKeyDown(object? sender, KeyEventArgs e)
        {
            if (TesterTakesKey(e)) return;
            if (_listeningForKey is null && _listeningForHotkey is null) return;

            // A bare modifier would be an unpressable binding.
            if (e.Key is Key.LeftCtrl or Key.RightCtrl or Key.LeftAlt or Key.RightAlt
                or Key.LeftShift or Key.RightShift or Key.LWin or Key.RWin) return;

            if (e.Key != Key.Escape)
            {
                if (_listeningForKey is { } target)
                {
                    (string console, PadControl button) = target;

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
                        if (kv.Value.TryGetControl(e.Key, out PadControl clash)) kv.Value.Unbind(clash);
                    }
                    _keyBindings.Save();
                    _hotkeyBindings.Rebind(action, e.Key);
                    _hotkeyBindings.Save();
                }
            }

            // A capture begun from the drawing listens for both; a key answers it, or Escape cancels it, for the pad too.
            if (_listeningForKey is { } answered && _listeningForPad == answered) StopListeningForPad();
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
                (string console, PadControl button) = kv.Key;
                kv.Value.Text = CurrentKeyLabel(console, button);
                _rebindKeyButtons[kv.Key].Content = RebindKeyText;
            }
            RefreshConflicts();
            RefreshDiagramLabels();
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
            var conflicting = new HashSet<(string, PadControl)>();
            var conflictingHotkeys = new HashSet<HotkeyAction>();

            foreach (CoreDescriptor console in _consoles)
            {
                string name = console.Console;
                var buttons = CoreCatalog.ControlsFor(name).ToHashSet();
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

                    foreach (PadControl button in buttons)
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

        private void StartListeningForPad(string console, PadControl button)
        {
            if (_gamepad is null) return; // parameterless-ctor / previewer case

            if (_listeningForPad is { } previous) _rebindPadButtons[previous].Content = RebindPadText;

            _listeningForPad = (console, button);
            _rebindPadButtons[(console, button)].Content = ListeningPadText;
            _padArmed = false;
            _padListening.Restart();

            _padPollTimer?.Stop();
            _padPollTimer = new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(50) };
            _padPollTimer.Tick += (_, _) => PollForPadButton();
            _padPollTimer.Start();
        }

        private void PollForPadButton()
        {
            // The bound button is let go before the pad is the interface's again, or binding South would press Rebind Pad once more.
            if (_padReleaseWait)
            {
                if (_gamepad?.GetAnyPressedButton() is null)
                {
                    _padReleaseWait = false;
                    _padPollTimer?.Stop();
                }
                return;
            }

            // Guarded locally so the timer stops itself - see EmuSen_Settings_Reference.md §4.6.
            if (_gamepad is null || _listeningForPad is not { } target)
            {
                _padPollTimer?.Stop();
                return;
            }

            (string console, PadControl control) = target;
            if (!PadControls.IsButton(control, out PadButton button)) return;

            SDL.GamepadButton? pressed = _gamepad.GetAnyPressedButton();
            if (!_padArmed)
            {
                _padArmed = pressed is null;
                return;
            }
            // Pulling the trigger for L2 or R2 binds it alone: the button beside it is cleared, since the trigger is always read.
            if (pressed is null && TriggerFor(button) is not null
                && _gamepad.RawAxis(button == PadButton.L2 ? SDL.GamepadAxis.LeftTrigger : SDL.GamepadAxis.RightTrigger) >= 0.5)
            {
                OwnPadMap(console).Unbind(button);
                _gamepadBindings.Save();
                _listeningForPad = null;
                if (_listeningForKey == target) ClearKeyListening();
                RefreshPadLabels();
                return;
            }
            if (pressed is not SDL.GamepadButton padButton)
            {
                if (_padListening.Elapsed >= PadCaptureTimeout) StopListeningForPad();
                return;
            }

            OwnPadMap(console).Rebind(button, padButton);
            _gamepadBindings.Save();

            _listeningForPad = null;
            _padReleaseWait = true;
            if (_listeningForKey == target) ClearKeyListening();
            RefreshPadLabels();
        }

        private void RefreshPadLabels()
        {
            foreach (var kv in _padLabels)
            {
                (string console, PadControl button) = kv.Key;
                kv.Value.Text = CurrentPadLabel(console, button);
                _rebindPadButtons[kv.Key].Content = RebindPadText;
            }
            foreach (string console in _playerRows.Keys) UpdatePlayerRow(console);
            RefreshDiagramLabels();
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
            RawDiagram.StickRing = _appSettings.StickDeadzone;
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
