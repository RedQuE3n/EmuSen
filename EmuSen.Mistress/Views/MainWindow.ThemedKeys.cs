using System.Collections.Generic;
using Avalonia.Input;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // ES-DE's default keyboard, with F4 for Escape's Start, steering the themed view as held pad buttons - see EmuSen_Settings_Reference.md §4.52.
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
            Key.F4 => UiButton.Menu,
            Key.F1 => UiButton.Options,
            Key.Insert => UiButton.Search,
            Key.PageUp => UiButton.PageUp,
            Key.PageDown => UiButton.PageDown,
            Key.Home => UiButton.First,
            Key.End => UiButton.Last,
            Key.F2 or Key.F3 => UiButton.Random,
            Key.Delete => UiButton.Screensaver,
            _ => null,
        };

        // True when the key was the themed view's: pressed over it with nothing else taking keys, or let go after such a press.
        private bool ThemedKey(Key key, KeyModifiers modifiers, bool pressed)
        {
            if (ThemedKeyButton(key) is not { } button) return false;
            if (!pressed) return _themedKeys.Remove(button);
            if ((modifiers & KeyModifiers.Alt) != 0) return false;
            if (OnScreenKeyboard.OpenOver(this) is not null) return false;
            if (!BigMenuOnScreen && (!ThemedLibraryShown || !LibraryView.IsVisible || OtherWindow() is not null)) return false;
            _themedKeys.Add(button);
            _keyboardSteering = true;
            PadTick();
            return true;
        }

        // A big-screen menu on screen, the pad menu or a sheet drawn as a menu, which ES-DE's keys drive as they drive the view (Q102, §4.72.7, §4.80).
        internal bool BigMenuOnScreen => _bigScreen && (_padMenuOpen || Sheets.Current is { } sheet && Sheets.DrawsMenu(sheet));

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
