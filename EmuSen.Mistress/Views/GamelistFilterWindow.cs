using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.BigPicture.Scene;

namespace EmuSen.Mistress.Views
{
    // ES-DE's filter screen: the game name, then each filter's values taken from the list's own games, or "Nothing to filter" - see EmuSen_Settings_Reference.md §4.58.
    public sealed class GamelistFilterWindow : ToolWindow
    {
        private readonly IReadOnlyList<SceneGame> _games;
        private readonly TextBox _name = new() { Name = "FilterGameName", Watermark = "Any name", MinWidth = 320 };
        private readonly StackPanel _fields = Ui.Stack(12);
        private bool _filling;

        public GamelistFilterWindow(GameFilter current, IReadOnlyList<SceneGame> games)
        {
            _games = games;
            Filter = current;
            Title = "Filter Gamelist";
            Width = 720;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            _name.Text = current.Name;
            _name.TextChanged += (_, _) => { if (!_filling) Filter = Filter with { Name = _name.Text ?? "" }; };

            Button reset = Ui.Button("Reset All Filters", Reset);
            reset.Name = "FilterReset";
            Button done = Ui.Button("Done", Close);
            done.Name = "FilterDone";
            Control buttons = Ui.Buttons(reset, done).Margin(0, 12, 0, 0);
            DockPanel.SetDock(buttons, Dock.Bottom);
            var page = Ui.Stack(12, new FieldRow { Label = "Game Name", Hint = "Any part of the name, in any case.", Content = _name }, _fields);
            Content = new DockPanel { LastChildFill = true, Children = { buttons, new ScrollViewer { Content = page.Margin(4, 4, 4, 4), MaxHeight = 560 } } }.Margin(16);
            Fill();
        }

        // The filter as the screen now sets it.
        public GameFilter Filter { get; private set; }

        private void Fill()
        {
            _filling = true;
            _fields.Children.Clear();
            foreach (FilterField field in GamelistOptions.Fields)
            {
                IReadOnlyList<string> values = GamelistOptions.Values(_games, field);
                Control content;
                if (values.Count == 0) content = new HintText { Text = GamelistOptions.NothingToFilter, Name = $"Filter{field}Nothing" };
                else
                {
                    var wrap = new WrapPanel { Orientation = Orientation.Horizontal, Name = $"Filter{field}Values" };
                    IReadOnlySet<string> chosen = Filter.Chosen.GetValueOrDefault(field) ?? new HashSet<string>();
                    foreach (string value in values)
                    {
                        var box = new LunaSwitch { Label = value, IsChecked = chosen.Contains(value), Margin = new Avalonia.Thickness(0, 0, 16, 4) };
                        box.IsCheckedChanged += (_, _) => Toggle(field, value, box.IsChecked == true);
                        wrap.Children.Add(box);
                    }
                    content = wrap;
                }
                _fields.Children.Add(new FieldRow { Label = GamelistOptions.Label(field), Content = content });
            }
            _filling = false;
        }

        private void Toggle(FilterField field, string value, bool on)
        {
            if (_filling) return;
            var chosen = new HashSet<string>(Filter.Chosen.GetValueOrDefault(field) ?? new HashSet<string>(), StringComparer.OrdinalIgnoreCase);
            if (on) chosen.Add(value);
            else chosen.Remove(value);
            Filter = Filter.With(field, chosen);
        }

        private void Reset()
        {
            Filter = GameFilter.None;
            _filling = true;
            _name.Text = "";
            _filling = false;
            Fill();
        }
    }
}
