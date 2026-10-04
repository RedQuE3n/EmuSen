using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Layout;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Endymion.Input;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using SDL3;

namespace EmuSen.Mistress.Views
{
    // Each console's pad drawn with its bindings on it, chosen to rebind, and lit by what is pressed - see EmuSen_Settings_Reference.md §4.81.
    public partial class InputSettingsWindow
    {
        private readonly Dictionary<string, ControllerDiagram> _diagrams = new(StringComparer.OrdinalIgnoreCase);
        private readonly Dictionary<string, TextBlock> _diagramStatus = new(StringComparer.OrdinalIgnoreCase);
        private readonly HashSet<Key> _heldKeys = new();
        private bool _testing;

        // The window's content, which a sheet takes away from the window.
        private Control _body = null!;
        private TimeSpan? _backHeldSince;
        private string? _sheetHintBefore;
        private SheetLayer? _hintedLayer;

        // How long B is held to leave the tester, since a short press of B is a button being tried.
        public static readonly TimeSpan TestExitHold = TimeSpan.FromSeconds(1);

        private static readonly Stopwatch Uptime = Stopwatch.StartNew();

        // The time the hold is measured on; a test sets its own.
        public Func<TimeSpan> Clock { get; set; } = () => Uptime.Elapsed;

        public const string PadHint = "A  Rebind      Y  Test buttons      B  Back      L1 R1  Console";
        public const string TestingPadHint = "Every press lights up      Hold B  Stop testing";

        private const string DesktopHelp = "Choose a button on the controller or its label, then press the key or pad button for it; Escape cancels. " +
            "Whatever you press lights up. A key can only do one thing on the same console, so binding it somewhere new clears it from wherever it was.";
        private const string TestingHelp = "Testing: every press lights up and nothing else happens. Press Escape, click Test Buttons, or hold B on the pad for a second to stop.";

        public bool IsTesting => _testing;

        // The console whose tab is showing, or null on General.
        public string? ShownConsole => Tabs.SelectedIndex >= 1 && Tabs.SelectedIndex <= _consoles.Count ? _consoles[Tabs.SelectedIndex - 1].Console : null;

        public ControllerDiagram? DiagramFor(string console) => _diagrams.TryGetValue(console, out ControllerDiagram? d) ? d : null;

        private void SetUpTester()
        {
            HelpText.Text = DesktopHelp;
            SetUpRawTester();
            if (_gamepad is not null) _gamepad.Polled += OnPadPolled;
            Tabs.SelectionChanged += (_, _) => UpdateTester();
            Deactivated += (_, _) => { _heldKeys.Clear(); UpdateTester(); };
            _body = (Control)Content!;
            _body.AttachedToVisualTree += (_, _) => ShowSheetHint();
            MenuLook.SetWidthFraction(this, 0.94);
            MenuLook.WhenApplied(_body, InLook);
            Closed += (_, _) => TearDownTester();
        }

        private void TearDownTester()
        {
            if (_gamepad is not null) _gamepad.Polled -= OnPadPolled;
            if (_hintedLayer is not null) _hintedLayer.Hint = _sheetHintBefore;
            _hintedLayer = null;
        }

        // On a sheet the footer names this window's buttons, as the rewind reel's does, and gives the old one back on close.
        private void ShowSheetHint()
        {
            if (SheetLayer.PresenterOf(this) is not { } layer) return;
            if (_hintedLayer is null) _sheetHintBefore = layer.Hint;
            _hintedLayer = layer;
            layer.Hint = PadHints.Face(_testing ? TestingPadHint : PadHint);
            MenuLook.SetHints(this, MenuHints());
            HelpText.IsVisible = false;
        }

        // In ES-DE's look the help bar and the labels' own prompts do what the words above and below the drawing and the Close and Test Buttons buttons did, so the drawing has the room (Q184).
        private void InLook()
        {
            HelpText.IsVisible = false;
            CloseButton.IsVisible = false;
            TestToggle.IsVisible = false;
            foreach (TextBlock status in _diagramStatus.Values) status.IsVisible = false;
            SizeColumnsToText();
            MoveResetToGeneral();
        }

        // The footer row takes height from the drawing on a sheet, so Reset to Defaults, which resets every console, goes to the end of General, and the row shows only for a conflict (Q184).
        private void MoveResetToGeneral()
        {
            if (ResetButton.Parent is ItemsControl bar && MirrorPlayer1ToPlayer2CheckBox.Parent is Panel general)
            {
                bar.Items.Remove(ResetButton);
                ResetButton.HorizontalAlignment = HorizontalAlignment.Left;
                ResetButton.Margin = new Thickness(0, 18, 0, 0);
                general.Children.Add(ResetButton);
            }
            void Footer() => FooterBar.IsVisible = !string.IsNullOrEmpty(ConflictText.Text);
            ConflictText.PropertyChanged += (_, e) => { if (e.Property == TextBlock.TextProperty) Footer(); };
            Footer();
        }

        // The look's larger capitals do not fit the desktop's fixed columns, so each table's columns take their widest cell, shared by its rows (§4.81).
        private void SizeColumnsToText()
        {
            var tables = new List<(Panel Scope, IEnumerable<Grid> Rows)>
            {
                ((Panel)HotkeysPanel.Parent!, new[] { HotkeyHeaderRow }.Concat(HotkeysPanel.Children.OfType<Grid>())),
            };
            foreach (StackPanel list in _body.GetLogicalDescendants().OfType<StackPanel>().Where(p => p.Name == "BindingsPanel"))
                if (list.Parent is Panel page && page.Children.OfType<Grid>().FirstOrDefault(g => g.Name == "ButtonHeaderRow") is { } header)
                    tables.Add((page, new[] { header }.Concat(list.Children.OfType<Grid>())));
            int n = 0;
            foreach ((Panel scope, IEnumerable<Grid> rows) in tables)
            {
                Grid.SetIsSharedSizeScope(scope, true);
                string group = "Table" + n++;
                foreach (Grid row in rows)
                {
                    int count = row.ColumnDefinitions.Count;
                    row.ColumnDefinitions.Clear();
                    for (int i = 0; i < count; i++) row.ColumnDefinitions.Add(new ColumnDefinition(GridLength.Auto) { SharedSizeGroup = $"{group}c{i}" });
                    row.ColumnSpacing = 12;
                }
            }
        }

        // The help bar of the window framed in ES-DE's look: the same words as the plain sheet's footer, with the pad's buttons drawn.
        private IReadOnlyList<HintEntry> MenuHints() => _testing
            ? new[] { new HintEntry("Hold to stop testing") { Button = MainWindow.BackGlyph } }
            : new[]
            {
                new HintEntry("Rebind") { Button = MainWindow.AcceptGlyph },
                new HintEntry("Test buttons") { Button = PadHints.Glyph(PadGlyphButton.North) },
                new HintEntry("Back") { Button = MainWindow.BackGlyph },
                new HintEntry("Console") { Button = PadGlyphButton.Shoulders },
            };

        // The drawing over the page's list; the drawing takes the height the page shows, and the list is scrolled to below it.
        private Control WithDiagram(string console, ScrollViewer page, Control list)
        {
            var diagram = new ControllerDiagram { Layout = ControllerDiagrams.LayoutFor(console), Name = "Diagram" };
            Avalonia.Automation.AutomationProperties.SetName(diagram, $"{console} controller");
            diagram.RegionInvoked += (_, e) => ChooseRegion(console, e.Region);
            // Up past the drawing's top row is the player selector, which stands outside the tab's page.
            diagram.KeyDown += (_, e) =>
            {
                if (e.Handled || e.Key != Key.Up || e.KeyModifiers != KeyModifiers.None || !PlayerBar.IsVisible) return;
                _playerChoice.Focus(NavigationMethod.Directional);
                e.Handled = true;
            };
            _diagrams[console] = diagram;

            var status = new TextBlock { Name = "DiagramStatus", TextWrapping = Avalonia.Media.TextWrapping.Wrap, Margin = new Thickness(0, 6, 0, 0) };
            _diagramStatus[console] = status;
            var host = new DockPanel { MinHeight = 320 };
            DockPanel.SetDock(status, Dock.Bottom);
            host.Children.Add(status);
            host.Children.Add(diagram);
            page.GetObservable(ScrollViewer.ViewportProperty).Subscribe(new Observer<Size>(v => host.Height = Math.Max(320, v.Height - 4)));

            var allBindings = new TextBlock { Text = "All Bindings", FontWeight = Avalonia.Media.FontWeight.Bold, Margin = new Thickness(0, 18, 0, 0) };
            return new StackPanel { Spacing = 4, Children = { host, allBindings, list } };
        }

        private sealed class Observer<T>(Action<T> next) : IObserver<T>
        {
            public void OnCompleted() { }
            public void OnError(Exception error) { }
            public void OnNext(T value) => next(value);
        }

        // Every drawing's labels from the bindings, and the one being captured asks for its key or button.
        private void RefreshDiagramLabels()
        {
            foreach ((string console, ControllerDiagram diagram) in _diagrams)
            {
                foreach (PadControl control in CoreCatalog.ControlsFor(console))
                {
                    string region = ControllerDiagrams.RegionFor(console, control);
                    bool keyWanted = _listeningForKey == (console, control), padWanted = _listeningForPad == (console, control);
                    string? key = keyWanted ? "Press a key" : Bound(CurrentKeyLabel(console, control));
                    string? pad = padWanted ? "Press a button" : Friendly(Bound(CurrentPadLabel(console, control)));
                    diagram.SetBinding(region, key, pad);
                }
            }
            UpdateStatus();
        }

        private static string? Bound(string label) => label == Unbound ? null : label;

        // SDL's names spelled as words for the drawing's labels: "DPadUp" as "D-pad Up", "LeftShoulder" as "Left Shoulder".
        internal static string? Friendly(string? pad) =>
            pad is null ? null : System.Text.RegularExpressions.Regex.Replace(pad.Replace("DPad", "D-pad "), "(?<=[a-z])(?=[A-Z])", " ");

        private void UpdateStatus()
        {
            foreach ((string console, TextBlock status) in _diagramStatus)
            {
                PadControl? wanted = _listeningForKey is { } k && k.Console == console ? k.Button
                    : _listeningForPad is { } p && p.Console == console ? p.Button : null;
                status.Text = wanted is { } control
                    ? $"Binding {RegionName(console, control)}: press a {(_listeningForPad is not null ? "key or a pad button" : "key")}. Escape cancels{(_listeningForPad is not null ? $"; the pad gives up after {PadCaptureTimeout.TotalSeconds:0} seconds" : "")}."
                    : _testing ? "Testing: press anything on the pad or the keyboard."
                    : TesterPad(console) is { } shown ? $"Lights up what {shown.Name} and the keyboard press, as player {PlayerOf(console)}."
                    : "Lights up what the keyboard presses; plug in a pad to see it here too.";
            }
        }

        private string RegionName(string console, PadControl control) =>
            DiagramFor(console)?.Regions.FirstOrDefault(r => r.Id == ControllerDiagrams.RegionFor(console, control))?.Name ?? ShortName(control);

        // A region chosen on the drawing listens for a key and, for a button, a pad button as well; whichever comes first is bound.
        private void ChooseRegion(string console, string region)
        {
            if (ControllerDiagrams.ControlFor(console, region, CoreCatalog.ControlsFor(console)) is not { } control) return;
            if (_testing) SetTesting(false);
            CancelCapture();
            StartListeningForKey(console, control);
            if (PadControls.IsButton(control, out _)) StartListeningForPad(console, control);
            RefreshDiagramLabels();
            UpdateTester();
        }

        // --- The tester ---

        private void OnPadPolled()
        {
            if (_testing && _gamepad?.Primary is { } pad && pad.IsRawPressed(_appSettings.SwapPadButtons ? SDL.GamepadButton.South : SDL.GamepadButton.East))
            {
                _backHeldSince ??= Clock();
                if (Clock() - _backHeldSince.Value >= TestExitHold) SetTesting(false);
            }
            else _backHeldSince = null;
            UpdateTester();
        }

        // Lights every region of the shown drawing from the held keys and player 1's pad, and moves its sticks; nothing is lit while capturing.
        private void UpdateTester()
        {
            UpdateRawTester();
            foreach ((string console, ControllerDiagram diagram) in _diagrams)
            {
                bool live = console == ShownConsole && Capturing == PadCapture.None;
                foreach (PadControl control in CoreCatalog.ControlsFor(console))
                    diagram.SetPressed(ControllerDiagrams.RegionFor(console, control), live && Held(console, control));
                foreach ((string stick, PadAxis x, PadAxis y) in StickAxes(diagram.Layout))
                    diagram.SetStick(stick, live ? StickValue(console, x) : 0, live ? StickValue(console, y) : 0);
            }
        }

        // The General tab's modern pad, and which of player 1's own buttons each of its regions is.
        internal static readonly IReadOnlyDictionary<string, SDL.GamepadButton> RawButtons = new Dictionary<string, SDL.GamepadButton>
        {
            ["B"] = SDL.GamepadButton.South, ["A"] = SDL.GamepadButton.East, ["Y"] = SDL.GamepadButton.West, ["X"] = SDL.GamepadButton.North,
            ["L"] = SDL.GamepadButton.LeftShoulder, ["R"] = SDL.GamepadButton.RightShoulder, ["L3"] = SDL.GamepadButton.LeftStick, ["R3"] = SDL.GamepadButton.RightStick,
            ["Select"] = SDL.GamepadButton.Back, ["Start"] = SDL.GamepadButton.Start, ["Guide"] = SDL.GamepadButton.Guide,
            ["Up"] = SDL.GamepadButton.DPadUp, ["Down"] = SDL.GamepadButton.DPadDown, ["Left"] = SDL.GamepadButton.DPadLeft, ["Right"] = SDL.GamepadButton.DPadRight,
        };

        private void SetUpRawTester()
        {
            foreach ((string region, SDL.GamepadButton button) in RawButtons) RawDiagram.SetBinding(region, null, Friendly(PadName(button)));
            RawDiagram.SetBinding("L2", null, "Left Trigger");
            RawDiagram.SetBinding("R2", null, "Right Trigger");
            RawDiagram.StickRing = _appSettings.StickDeadzone;
        }

        // Player 1's pad as SDL reads it, whatever any console's bindings say: its buttons, its triggers' travel and its sticks' positions.
        private void UpdateRawTester()
        {
            ConnectedPad? pad = _gamepad?.Primary;
            bool live = Tabs.SelectedIndex == 0 && pad is not null;
            foreach ((string region, SDL.GamepadButton button) in RawButtons)
            {
                RawDiagram.SetPressed(region, live && pad!.IsRawPressed(button));
                string? letter = pad?.ButtonLabel(button);
                if (region is "A" or "B" or "X" or "Y") RawDiagram.SetCaption(region, letter is { Length: <= 2 } ? letter : "");
            }
            double left = live ? pad!.RawAxis(SDL.GamepadAxis.LeftTrigger) : 0, right = live ? pad!.RawAxis(SDL.GamepadAxis.RightTrigger) : 0;
            RawDiagram.SetTrigger("L2", left);
            RawDiagram.SetTrigger("R2", right);
            RawDiagram.SetPressed("L2", left >= 0.5);
            RawDiagram.SetPressed("R2", right >= 0.5);
            RawDiagram.SetStick("LeftStick", live ? pad!.RawAxis(SDL.GamepadAxis.LeftX) : 0, live ? pad!.RawAxis(SDL.GamepadAxis.LeftY) : 0);
            RawDiagram.SetStick("RightStick", live ? pad!.RawAxis(SDL.GamepadAxis.RightX) : 0, live ? pad!.RawAxis(SDL.GamepadAxis.RightY) : 0);
        }

        private static IEnumerable<(string Stick, PadAxis X, PadAxis Y)> StickAxes(ControllerLayout layout) => layout switch
        {
            ControllerLayout.Nintendo64 => new[] { ("Stick", PadAxis.LeftX, PadAxis.LeftY) },
            ControllerLayout.Gamepad => new[] { ("LeftStick", PadAxis.LeftX, PadAxis.LeftY), ("RightStick", PadAxis.RightX, PadAxis.RightY) },
            _ => Array.Empty<(string, PadAxis, PadAxis)>(),
        };

        private bool KeyHeld(string console, PadControl control) =>
            _keyBindings.For(console).ButtonToKey.TryGetValue(control, out Key key) && _heldKeys.Contains(key);

        private double RawAxis(string console, PadAxis axis) => _gamepad is { IsConnected: true } pad ? pad.Axis(axis, PlayerOf(console)) : 0;

        // One axis as the game would read it: the pad's stick, pushed all the way by a held key.
        private double StickValue(string console, PadAxis axis) => PadControls.Resolve(axis, RawAxis(console, axis), c => KeyHeld(console, c));

        private bool Held(string console, PadControl control)
        {
            if (KeyHeld(console, control)) return true;
            if (_gamepad is not { IsConnected: true } gamepad || TesterPad(console) is not { } pad) return false;

            if (!PadControls.IsButton(control, out PadButton button))
            {
                PadAxis axis = control switch
                {
                    PadControl.LeftStickUp or PadControl.LeftStickDown => PadAxis.LeftY,
                    PadControl.LeftStickLeft or PadControl.LeftStickRight => PadAxis.LeftX,
                    PadControl.RightStickUp or PadControl.RightStickDown => PadAxis.RightY,
                    _ => PadAxis.RightX,
                };
                bool negative = control is PadControl.LeftStickUp or PadControl.LeftStickLeft or PadControl.RightStickUp or PadControl.RightStickLeft;
                double value = RawAxis(console, axis);
                return negative ? value <= -ControllerDiagrams.DirectionThreshold : value >= ControllerDiagrams.DirectionThreshold;
            }

            // The stick stands in for the cross only on a console that does not read it as a stick - see EmuSen_Input.md §7.3.
            bool stickIsAnalog = CoreCatalog.AxesFor(console).Contains(PadAxis.LeftX);
            if (gamepad.AnalogStickAsDpad && !stickIsAnalog && button is PadButton.Up or PadButton.Down or PadButton.Left or PadButton.Right)
            {
                double threshold = Math.Clamp(gamepad.StickDeadzone, 0.05, 0.95);
                double x = pad.RawAxis(SDL.GamepadAxis.LeftX), y = pad.RawAxis(SDL.GamepadAxis.LeftY);
                if (button switch { PadButton.Up => y < -threshold, PadButton.Down => y > threshold, PadButton.Left => x < -threshold, _ => x > threshold }) return true;
            }

            if (button is PadButton.L2 or PadButton.R2 && pad.RawAxis(button == PadButton.L2 ? SDL.GamepadAxis.LeftTrigger : SDL.GamepadAxis.RightTrigger) >= 0.5) return true;
            return PadMap(console).ButtonToPad.TryGetValue(button, out SDL.GamepadButton bound) && pad.IsRawPressed(bound);
        }

        // Keys the tester reads: every key held lights its binding, and while testing no key does anything else but Escape, which stops.
        private bool TesterTakesKey(KeyEventArgs e)
        {
            if (PadWindowRouter.Raising) return false;
            if (Capturing != PadCapture.None) return false;
            _heldKeys.Add(e.Key);
            UpdateTester();
            if (!_testing) return false;
            if (e.Key == Key.Escape) SetTesting(false);
            e.Handled = true;
            return true;
        }

        private void OnPreviewKeyUp(object? sender, KeyEventArgs e)
        {
            if (PadWindowRouter.Raising) return;
            if (_heldKeys.Remove(e.Key)) UpdateTester();
            if (_testing) e.Handled = true;
        }

        private void OnTestToggleClick(object? sender, RoutedEventArgs e) => SetTesting(TestToggle.IsChecked == true);

        public void SetTesting(bool on)
        {
            if (on) CancelCapture();
            _testing = on;
            _backHeldSince = null;
            TestToggle.IsChecked = on;
            HelpText.Text = on ? TestingHelp : DesktopHelp;
            if (_hintedLayer is not null)
            {
                _hintedLayer.Hint = PadHints.Face(on ? TestingPadHint : PadHint);
                MenuLook.SetHints(this, MenuHints());
            }
            UpdateStatus();
            UpdateTester();
        }

        // The pad on the drawing: the cross moves between its buttons by where they are drawn, A rebinds, Y tries the buttons; while trying them, the pad does nothing else.
        public bool OnPad(UiButton button)
        {
            if (_testing) return true;
            if (button == UiButton.Search)
            {
                SetTesting(true);
                return true;
            }

            object? focused = TopLevel.GetTopLevel(_body)?.FocusManager?.GetFocusedElement();
            // The selector is a stop between the drawing and the tab strip: Up goes on to the tab shown.
            if (button == UiButton.Up && ReferenceEquals(focused, _playerChoice) && Tabs.ContainerFromIndex(Tabs.SelectedIndex) is TabItem shown)
            {
                shown.Focus(NavigationMethod.Directional);
                return true;
            }
            if (_diagrams.FirstOrDefault(d => d.Value.RegionOf(focused) is not null) is not { Value: { } diagram, Key: { } console }) return false;
            switch (button)
            {
                case UiButton.Up:
                    if (diagram.MoveSelection(NavigationDirection.Up)) return true;
                    if (!PlayerBar.IsVisible) return false;
                    _playerChoice.Focus(NavigationMethod.Directional);
                    return true;
                case UiButton.Down: return diagram.MoveSelection(NavigationDirection.Down);
                case UiButton.Left: return diagram.MoveSelection(NavigationDirection.Left);
                case UiButton.Right: return diagram.MoveSelection(NavigationDirection.Right);
                case UiButton.Accept:
                    ChooseRegion(console, diagram.RegionOf(focused)!);
                    return true;
                default: return false;
            }
        }
    }
}
