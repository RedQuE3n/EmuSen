using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Layout;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // One entry of the game options menu; Closes says whether choosing it puts the menu away first.
    public sealed record GameOption(string Label, Action Choose, bool Closes = true)
    {
        // A row of its own in place of a button, such as Jump To's or Sort's choice (§4.58).
        public Func<Control>? Row { get; init; }

        // The same row drawn as a big-screen menu row (§4.69).
        public Func<Control>? MenuRow { get; init; }

        // Run when the menu closes applying, not cancelled: ES-DE's B and Apply against its Back and Cancel.
        public Action? Apply { get; init; }

        // Whether choosing it opens another screen, which a big-screen menu marks with a chevron (§4.69).
        public bool Opens { get; init; }
    }

    // ES-DE's gamelist options menu, opened with Select on a game in the themed gamelist - see EmuSen_Settings_Reference.md §4.59; in a big-screen session ES-DE's look, §4.69.
    public sealed class GameOptionsWindow : ToolWindow, IPadDriven
    {
        private readonly StackPanel _entries = Ui.Stack(8);
        private readonly List<Button> _buttons = new();

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GameOptionsWindow() : this("", []) { }

        public GameOptionsWindow(string game, IReadOnlyList<GameOption> options, PadFamily? menu = null)
        {
            Title = "Options";
            Width = 560;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;
            Options = options;
            bool applies = options.Any(o => o.Apply is not null);
            if (menu is { } family) BuildMenu(game, options, applies, family);
            else BuildDesktop(game, options, applies);
            Opened += (_, _) => (_entries.Children.FirstOrDefault() as InputElement)?.Focus(NavigationMethod.Directional);
            Closing += (_, _) =>
            {
                if (_cancelled || _applied) return;
                _applied = true;
                foreach (GameOption option in Options) option.Apply?.Invoke();
            };
        }

        private void BuildDesktop(string game, IReadOnlyList<GameOption> options, bool applies)
        {
            foreach (GameOption option in options)
            {
                if (option.Row is { } row)
                {
                    _entries.Children.Add(row());
                    continue;
                }
                Button button = Entry(option);
                button.HorizontalContentAlignment = HorizontalAlignment.Left;
                _entries.Children.Add(button);
            }
            Button close = Ui.Button(applies ? "Apply" : "Close", Close);
            close.Name = "GameOptionsClose";
            Button[] buttons = applies ? [close, Ui.Button("Cancel", Cancel)] : [close];
            if (applies) buttons[1].Name = "GameOptionsCancel";
            Content = Ui.Stack(12,
                new TextBlock { Name = "GameOptionsTitle", Text = game, FontSize = 20, FontWeight = Avalonia.Media.FontWeight.SemiBold, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                _entries,
                Ui.Buttons(buttons)).Margin(16);
        }

        // Rows under a large title in a panel of its own, over the screen: no Close button, since B and Select close it; Apply and Cancel where the rows apply.
        private void BuildMenu(string game, IReadOnlyList<GameOption> options, bool applies, PadFamily family)
        {
            SheetLayer.SetChromeless(this, true);
            _entries.Spacing = 0;
            foreach (GameOption option in options)
            {
                if (option.MenuRow is { } menuRow) _entries.Children.Add(menuRow());
                else if (option.Row is { } row) _entries.Children.Add(row());
                else _entries.Children.Add(MenuRows.Apply(Entry(option), option.Opens ? MenuRowKind.Submenu : MenuRowKind.Action));
            }
            if (applies)
            {
                Button apply = MenuRows.ApplyButton(Ui.Button("Apply", Close));
                apply.Name = "GameOptionsClose";
                Button cancel = MenuRows.ApplyButton(Ui.Button("Cancel", Cancel));
                cancel.Name = "GameOptionsCancel";
                _entries.Children.Add(new StackPanel { Orientation = Orientation.Horizontal, HorizontalAlignment = HorizontalAlignment.Center, Spacing = 8, Margin = new Thickness(0, 12, 0, 0), Children = { apply, cancel } });
            }
            bool adjustable = _entries.Children.OfType<ComboBox>().Any();
            Content = new MenuPanel
            {
                Name = "GameOptionsMenu",
                Title = game,
                HintFamily = family,
                Hints = MainWindow.GameOptionsHints(applies, adjustable),
                Child = new ScrollViewer { Content = _entries, VerticalScrollBarVisibility = Avalonia.Controls.Primitives.ScrollBarVisibility.Hidden },
            };
        }

        private Button Entry(GameOption option)
        {
            Button button = Ui.Button(option.Label, () => Choose(option));
            button.Name = "GameOption_" + new string(option.Label.Where(char.IsLetterOrDigit).ToArray());
            button.HorizontalAlignment = HorizontalAlignment.Stretch;
            _buttons.Add(button);
            return button;
        }

        private bool _cancelled, _applied;

        // Closes without applying the rows' choices, as ES-DE's Back and Cancel do.
        public void Cancel()
        {
            _cancelled = true;
            Close();
        }

        public IReadOnlyList<GameOption> Options { get; }

        // The entries' own buttons, not the rows' nor Apply and Cancel.
        public IEnumerable<Button> Entries => _buttons;

        // An entry that closes the menu applies the rows first, as ES-DE applies them when leaving its menu for another screen.
        private void Choose(GameOption option)
        {
            if (option.Closes) Close();
            option.Choose();
        }

        // Select again puts the menu away without applying, as ES-DE's Back button does on its own menu.
        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Options) return false;
            Cancel();
            return true;
        }
    }
}
