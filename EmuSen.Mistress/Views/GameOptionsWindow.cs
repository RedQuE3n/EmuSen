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
    public sealed record GameOption(string Label, Action Choose, bool Closes = true);

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
                Button button = Ui.Button(option.Label, () => Choose(option));
                button.Name = "GameOption_" + new string(option.Label.Where(char.IsLetterOrDigit).ToArray());
                button.HorizontalAlignment = HorizontalAlignment.Stretch;
                button.HorizontalContentAlignment = HorizontalAlignment.Left;
                _entries.Children.Add(button);
            }
            Button close = Ui.Button("Close", Close);
            close.Name = "GameOptionsClose";
            Content = Ui.Stack(12,
                new TextBlock { Name = "GameOptionsTitle", Text = game, FontSize = 20, FontWeight = Avalonia.Media.FontWeight.SemiBold, TextWrapping = Avalonia.Media.TextWrapping.Wrap },
                _entries,
                Ui.Buttons(close)).Margin(16);
            Opened += (_, _) => (_entries.Children.FirstOrDefault() as InputElement)?.Focus(NavigationMethod.Directional);
        }

        public IReadOnlyList<GameOption> Options { get; }

        public IEnumerable<Button> Entries => _entries.Children.OfType<Button>();

        private void Choose(GameOption option)
        {
            if (option.Closes) Close();
            option.Choose();
        }

        // Select again puts the menu away, as ES-DE's Back button does on its own menu.
        public bool OnPad(UiButton button)
        {
            if (button != UiButton.Options) return false;
            Close();
            return true;
        }
    }
}
