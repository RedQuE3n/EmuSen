using System;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Fluent;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Library;
using EmuSen.Serenity;

namespace EmuSen.Mistress.Views
{
    // One game's own crop, in place of its console's: the explicit exception to every game of a console being drawn alike - see EmuSen_Settings_Reference.md §4.100.
    public sealed class GameCropWindow : ToolWindow
    {
        private readonly GameCrop _console;
        private readonly string _consoleName;
        private readonly Action<GameCrop?> _changed;
        private readonly HintText _status = new() { Name = "GameCropStatus", TextWrapping = TextWrapping.Wrap };
        private readonly CropFields _fields;
        private readonly Button _useConsole;
        private GameCrop? _own;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public GameCropWindow() : this("", "", PictureCrop.None, null, _ => { }) { }

        // changed is told the game's crop as it is typed, or null for its console's.
        public GameCropWindow(string game, string console, PictureCrop consoleCrop, GameCrop? own, Action<GameCrop?> changed)
        {
            _console = GameCrop.From(consoleCrop);
            _consoleName = $"the {console}'s";
            _own = own;
            _changed = changed;

            Title = "Crop This Game";
            Width = 560;
            CanResize = false;
            SizeToContent = SizeToContent.Height;
            WindowStartupLocation = WindowStartupLocation.CenterOwner;
            ClosesOnEscape = true;

            _fields = new CropFields("GameCrop", edge => Shown[edge], Typed);
            _useConsole = Ui.Button("Use the Console's Setting", UseConsole);
            _useConsole.Name = "GameCropUseConsole";
            var name = new TextBlock { Name = "GameCropGame", Text = game, TextWrapping = TextWrapping.Wrap, FontWeight = FontWeight.SemiBold };
            Control hint = Ui.Hint($"{CropFields.Hint} The rest fills the screen. A crop set here is this game's alone and replaces {_consoleName} Overscan setting while it runs.");
            Control buttons = Ui.Buttons(_useConsole, Ui.Button("Close", Close));
            buttons.Margin = new Avalonia.Thickness(0, 6, 0, 0);
            Content = Ui.Stack(10, name, hint, new FieldRow { Label = "Crop, % of the picture", Content = _fields.Row }, _status, buttons).Margin(16);
            ShowStatus();
        }

        // What the boxes show: the game's own crop, or its console's until it has one.
        private GameCrop Shown => _own ?? _console;

        public GameCrop? Own => _own;

        private void Typed(int edge, double percent)
        {
            _own = Shown.With(edge, percent);
            _changed(_own);
            ShowStatus();
        }

        private void UseConsole()
        {
            _own = null;
            _changed(null);
            _fields.Show();
            ShowStatus();
        }

        private void ShowStatus()
        {
            _status.Text = _own is null ? $"This game is drawn with {_consoleName} setting." : "This game has a crop of its own.";
            _useConsole.IsEnabled = _own is not null;
        }
    }
}
