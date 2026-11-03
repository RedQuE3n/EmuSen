using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Media;
using Avalonia.Input;
using EmuSen.Galaxia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;

namespace EmuSen.Mistress.Views
{
    // Settings' plain description drawn on LunaP's SettingsLayout, and the layout's presses handed back to it - see EmuSen_BigPicture.md §52.
    public sealed class SettingsView
    {
        private readonly Func<IImage?> _frame;
        private int _depth;

        public SettingsView(SettingsScreen model, SettingsLayout layout, Func<IImage?> frame)
        {
            Model = model;
            Layout = layout;
            _frame = frame;
            layout.SectionChosen += key => { Model.ChooseSection(key); Fill(); };
            layout.RowStepped += (key, by) => { Model.Step(key, by); Fill(); };
            layout.RowChosen += key => { Model.Choose(key); Fill(); };
            layout.BackRequested += () => Back();
            layout.SearchChanged += text => { Model.Search(text); Fill(); };
            layout.ContentMoved += (dx, dy) => MoveBinding(dx, dy);
            layout.ContentTabChosen += at =>
            {
                if (Model.Binding is not { } page) return;
                if (at == 0) Model.StepBindingPlayer();
                else Model.ChooseBindingConsole(ConsoleAt(at));
                Fill();
            };
            layout.HintInvoked += key =>
            {
                if (key == SettingsScreen.HintReset) Model.ResetBinding();
                else if (key == SettingsScreen.HintPlayer) Model.StepBindingPlayer();
                else if (key == SettingsScreen.HintTest) Model.ToggleTesting();
                else if (key == SettingsScreen.HintClear && Model.Binding is not null) Model.ClearKey();
                else if (key == SettingsScreen.HintClear && layout.Focused is { } row) Model.ClearHotkey(row);
                Fill();
            };
            Fill();
        }

        private ControllerDiagram? _diagram;

        // The drawing the bindings page shows, made once and kept while Settings is open.
        public ControllerDiagram? Diagram => _diagram;

        private string ConsoleAt(int tab) => Model.Binding is { } page ? Model.BindingConsolesFrom(page).ElementAtOrDefault(tab - 1) ?? page.Console : "";

        // Each stick's knob where the player's stick, or a key held for its direction, puts it; at rest while A listens.
        public void SetSticks(Func<PadAxis, double> value)
        {
            if (_diagram is null || Model.Binding is not { } page) return;
            foreach ((string stick, PadAxis x, PadAxis y) in StickAxes(_diagram.Layout))
                _diagram.SetStick(stick, page.Capturing ? 0 : value(x), page.Capturing ? 0 : value(y));
        }

        private static IEnumerable<(string Stick, PadAxis X, PadAxis Y)> StickAxes(ControllerLayout layout) => layout switch
        {
            ControllerLayout.Nintendo64 => [("Stick", PadAxis.LeftX, PadAxis.LeftY)],
            ControllerLayout.Gamepad or ControllerLayout.DualShock => [("LeftStick", PadAxis.LeftX, PadAxis.LeftY), ("RightStick", PadAxis.RightX, PadAxis.RightY)],
            _ => [],
        };

        // The directions move between the drawing's regions by where they are drawn; a region with no control of the console's is passed over.
        private void MoveBinding(int dx, int dy)
        {
            if (Model.Binding is not { Focus: { } focus } page || _diagram is null) return;
            NavigationDirection direction = dx < 0 ? NavigationDirection.Left : dx > 0 ? NavigationDirection.Right : dy < 0 ? NavigationDirection.Up : NavigationDirection.Down;
            IReadOnlyList<PadControl> controls = Model.ControlsOf(page.Console);
            string region = ControllerDiagrams.RegionFor(page.Console, focus);
            for (int hop = 0; hop < 32 && _diagram.Neighbour(region, direction) is { } next; hop++)
            {
                region = next;
                if (ControllerDiagrams.ControlFor(page.Console, region, controls) is { } control)
                {
                    Model.FocusBinding(control);
                    break;
                }
            }
            Fill();
        }

        // The bindings page as content: the console's controller drawn, its focused control selected, the tabs the player and the consoles.
        private void FillBindings(BindingsPage page)
        {
            if (_diagram is null)
            {
                _diagram = new ControllerDiagram { Name = "BindingsDiagram", ShowsLabels = false, ShowsKeys = false };
                _diagram.RegionInvoked += (_, e) =>
                {
                    if (Model.Binding is not { } shown || ControllerDiagrams.ControlFor(shown.Console, e.Region, Model.ControlsOf(shown.Console)) is not { } control) return;
                    Model.FocusBinding(control);
                    Model.Choose(SettingsScreen.BindingKey(control));
                    Fill();
                };
            }
            // The pad the shown player's port is set to, where the console's settings choose one; the drawing follows it.
            IReadOnlyList<string?> players = Model.Host.PadNow(page.Console).Players;
            _diagram.Layout = ControllerDiagrams.LayoutFor(page.Console, page.Player <= players.Count ? players[page.Player - 1] : null);
            Avalonia.Automation.AutomationProperties.SetName(_diagram, $"{page.Console} controller");
            foreach (PadControl control in Model.ControlsOf(page.Console))
            {
                string region = ControllerDiagrams.RegionFor(page.Console, control);
                _diagram.SetBinding(region, page.Keys ? Model.BoundKey(page, control) : null, page.Keys ? null : Model.BoundPad(page, control));
                _diagram.SetPressed(region, !page.Capturing && Model.Held.Contains(control));
            }
            _diagram.SelectedRegion = page.Focus is { } focus ? ControllerDiagrams.RegionFor(page.Console, focus) : null;
            string key = page.Focus is { } f ? SettingsScreen.BindingKey(f) : "bind:none";
            Layout.SetContent(new SettingsContent(key, _diagram)
            {
                Tabs = Model.BindingTabs(page),
                Chosen = SettingsScreen.BindingTab(page),
                Caption = page.Keys ? "Left and Right move between the drawing's regions; A rebinds the focused one to a key." : "Left and Right move between the drawing's regions; A rebinds the focused one.",
            });
        }

        public SettingsScreen Model { get; }

        public SettingsLayout Layout { get; }

        // B: a page closes back to its row; false when nothing is left to close.
        public bool Back()
        {
            if (!Model.Back()) return false;
            Fill();
            return true;
        }

        // Draws the screen from the description: a page just opened starts on its first row, a page closed returns to the row it came from.
        public void Fill()
        {
            int depth = Model.Trail.Count;
            string? focus = depth > _depth ? Model.Rows.FirstOrDefault()?.Key : depth < _depth ? Model.Returned : null;
            if (depth == 0 && Layout.SelectedSection != Model.Section) focus = Model.Rows.FirstOrDefault()?.Key;
            _depth = depth;
            Layout.SetSections(SettingsScreen.SectionList.Select(s => new AdaptiveSection(s.Key, s.Label)), Model.Section);
            Layout.SetTrail(Model.Trail);
            Layout.PaneFor = key => Pane(Model.PaneOf(key));
            Layout.HintsFor = key => Model.HintsFor(key).Select(h => new PageHint(Enum.Parse<PadGlyphButton>(h.Button), h.Text, h.Key)).ToList();
            Layout.SetRows(Model.Rows, focus);
            if (Model.Binding is { } page) FillBindings(page);
            else if (Layout.Content is not null) Layout.SetContent(null);
        }

        private SettingsPane? Pane(ScreenPane? pane) => pane is null ? null : new SettingsPane(pane.Lines)
        {
            Preview = pane.Preview switch
            {
                PanePreview.Frame f => new SettingsPicture(_frame(), f.Caption),
                PanePreview.Meter m => new SettingsMeter(m.Fraction),
                PanePreview.Variants v => new SettingsVariants(v.Dark, v.Light),
                _ => null,
            },
        };
    }
}
