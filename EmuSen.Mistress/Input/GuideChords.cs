namespace EmuSen.Mistress.Input
{
    // What a Guide-held chord asked for: nothing yet, a save, a load, a screenshot, or the menu, for Guide let go alone.
    public enum GuideChord { None, Save, Load, Screenshot, Menu }

    // Guide held with Up, Down or R1, read from the buttons each tick; a button already down when Guide went down is not a chord - see EmuSen_BigPicture.md §48.7.
    public sealed class GuideChordReader
    {
        private bool _guide, _used, _up, _down, _r1;

        public GuideChord Feed(bool guide, bool up, bool down, bool r1)
        {
            if (!guide)
            {
                bool alone = _guide && !_used;
                _guide = false;
                return alone ? GuideChord.Menu : GuideChord.None;
            }
            GuideChord chord = GuideChord.None;
            if (_guide)
            {
                if (up && !_up) chord = GuideChord.Save;
                else if (down && !_down) chord = GuideChord.Load;
                else if (r1 && !_r1) chord = GuideChord.Screenshot;
            }
            else _used = false;
            if (chord != GuideChord.None) _used = true;
            _guide = true;
            (_up, _down, _r1) = (up, down, r1);
            return chord;
        }

        // Forgets a Guide held across a menu, so letting it go opens nothing.
        public void Reset(bool guideHeld)
        {
            _guide = guideHeld;
            _used = true;
        }
    }
}
