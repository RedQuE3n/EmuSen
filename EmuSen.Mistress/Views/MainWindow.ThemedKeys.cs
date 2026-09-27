using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // ES-DE's default keyboard, less Escape, steering the themed view as held pad buttons - see EmuSen_Settings_Reference.md §4.52.
    public partial class MainWindow
    {
        private readonly HashSet<UiButton> _themedKeys = new();
        private bool _keyboardSteering;

        internal static UiButton? ThemedKeyButton(Key key) => key switch
        {
            Key.Up => UiButton.Up,
            Key.Down => UiButton.Down,
            Key.Left => UiButton.Left,
            Key.Right => UiButton.Right,
            Key.Enter => UiButton.Accept,
            Key.Back => UiButton.Back,
            Key.F1 => UiButton.Options,
            Key.Insert => UiButton.Search,
            Key.PageUp => UiButton.PageUp,
            Key.PageDown => UiButton.PageDown,
            Key.Home => UiButton.First,
            Key.End => UiButton.Last,
            Key.F2 or Key.F3 => UiButton.Random,
            _ => null,
        };

        // True when the key was the themed view's: pressed over it with nothing else taking keys, or let go after such a press.
        private bool ThemedKey(Key key, bool pressed)
        {
            if (ThemedKeyButton(key) is not { } button) return false;
            if (!pressed) return _themedKeys.Remove(button);
            if (!ThemedLibraryShown || !LibraryView.IsVisible || OtherWindow() is not null || OnScreenKeyboard.OpenOver(this) is not null) return false;
            _themedKeys.Add(button);
            _keyboardSteering = true;
            PadTick();
            return true;
        }

        private bool KeyHeld(UiButton button) => _themedKeys.Contains(button);

        // Polls once more after the last key is let go, so the navigator and the view see the release.
        private bool KeyboardSteers()
        {
            bool steering = _keyboardSteering;
            _keyboardSteering = _themedKeys.Count > 0;
            return steering;
        }
    }
}
