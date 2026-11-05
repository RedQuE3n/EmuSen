using EmuSen.Mistress.BigPicture;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The own look's navigation sounds: one of ES-DE's seven names a press, played through PlayUiSound - see EmuSen_Interface.md §23.
    public partial class MainWindow
    {
        private string? _ownCue;

        // The big screen with no theme drawing the library, the switch on, and no game on screen or under the Pause Card.
        private bool OwnLookSounds =>
            _bigScreen && _appSettings.NavigationSounds && !ThemedLibraryShown && !GameOnScreen && !(_padMenuOpen && _padMenuInGame) && OtherWindow() is null;

        // What a press did, named outright: a launch or a favourite outranks the button's own cue.
        private void Cue(string name) => _ownCue = name;

        // A press a screen used, as its button's cue unless the press already named one.
        private void Heard(UiButton button, bool used)
        {
            if (used) _ownCue ??= CueOf(button);
        }

        private static string CueOf(UiButton button) => button switch
        {
            UiButton.Up or UiButton.Down or UiButton.Left or UiButton.Right or UiButton.First or UiButton.Last => "scroll",
            UiButton.PageUp or UiButton.PageDown => "systembrowse",
            UiButton.Back => "back",
            _ => "select",
        };

        // Called around each press: the cue it named is played once it is done, if the own look was sounding when it came.
        private void PlayOwnCue(bool sounding)
        {
            string? cue = _ownCue;
            _ownCue = null;
            if (sounding && cue is not null) PlayUiSound(NavigationSounds.Key(cue));
        }
    }
}
