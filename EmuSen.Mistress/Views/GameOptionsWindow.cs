using System;
using System.Collections.Generic;
using System.Linq;
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

        // Run when the menu closes applying, not cancelled: ES-DE's B and Apply against its Back and Cancel.
        public Action? Apply { get; init; }
    }

    // ES-DE's gamelist options menu, opened with Select on a game in the themed gamelist - see EmuSen_Settings_Reference.md §4.59.
    public sealed class GameOptionsWindow : ToolWindow, IPadDriven
    {
        private readonly StackPanel _entries = Ui.Stack(8);

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GameOptionsWindow() : this("", []) { }

        public GameOptionsWindow(string game, IReadOnlyList<GameOption> options)
        {
            Title = "Options";
            Width = 560;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;
            Options = options;
            foreach (GameOption option in options)
            {
                if (option.Row is { } row)
                {
                    _entries.Children.Add(row());
                    continue;
                }
                Button button = Ui.Button(option.Label, () => Choose(option));
                button.Name = "GameOption_" + new string(option.Label.Where(char.IsLetterOrDigit).ToArray());
                button.HorizontalAlignment = HorizontalAlignment.Stretch;
                button.HorizontalContentAlignment = HorizontalAlignment.Left;
                _entries.Children.Add(button);
            }
            bool applies = options.Any(o => o.Apply is not null);
            Button close = Ui.Button(applies ? "Apply" : "Close", Close);
            close.Name = "GameOptionsClose";
            Button[] buttons = applies ? [close, Ui.Button("Cancel", Cancel)] : [close];
            if (applies) buttons[1].Name = "GameOptionsCancel";
            Content = Ui.Stack(12,
                new TextBlock { Name = "GameOptionsTitle", Text = game, FontSize = 20, FontWeight = Avalonia.Media.FontWeight.SemiBold, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                _entries,
                Ui.Buttons(buttons)).Margin(16);
            Opened += (_, _) => (_entries.Children.FirstOrDefault() as InputElement)?.Focus(NavigationMethod.Directional);
            Closing += (_, _) =>
            {
                if (_cancelled || _applied) return;
                _applied = true;
                foreach (GameOption option in Options) option.Apply?.Invoke();
            };
        }

        private bool _cancelled, _applied;

        // Closes without applying the rows' choices, as ES-DE's Back and Cancel do.
        public void Cancel()
        {
            _cancelled = true;
            Close();
        }

        public IReadOnlyList<GameOption> Options { get; }

        public IEnumerable<Button> Entries => _entries.Children.OfType<Button>();

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
